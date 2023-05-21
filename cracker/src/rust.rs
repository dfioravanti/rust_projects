use num::Integer;
use openssl::sha::sha1;
use std::sync::Arc;
use std::{
    sync::atomic::{AtomicBool, Ordering},
    thread,
    time::Instant,
};

// We need to work on HEX values but rust does not have a u4 this constant avoid having magic numbers in the code
const BITS_IN_HEX: u32 = 4;

/// Given a valid UTF8 `base_string` it tries to generate another string `output` of maximum length `padding`
/// composed of only ASCII characters such that sha1(`base_string` + `output`) has `nb_zeros` leading zeros.
/// The ASCII characters will be generated using all the numbers in `[lower_limit, upper_limit)`.
/// When a valid string is found in either this thread or another one the computation is halted and control returns to
/// The main thread.
///
/// # Arguments
///
/// * `base_string` - The base string that are given
/// * `nb_zeros` - The number of leading zeros in the hashing
/// * `padding` - The maximum number of characters we are allowed to expand `base_string`
/// * `lower_limit` - The minimum value that this thread will consider
/// * `upper_limit` - The maximum value that this thread will consider
/// * `is_found - A bool shared among threads to signal when one thread found a valid string
/// * `nb_thread` - Which thread is the current one
/// * `nb_threads` - The total number of threads
fn generate_valid_string_one_thread(
    base_string: String,
    nb_zeros: u32,
    max_padding: u32,
    lower_limit: u128,
    upper_limit: u128,
    is_found: &AtomicBool,
    nb_thread: u32,
    nb_threads: u32,
) -> Option<String> {
    // For sake of efficiency we view the string as an array of bytes.
    // After one allocation needed to add the extra padding we keep reusing this array of bytes loop after loop.
    // This allow us to avoid having rust calling memcpy as we do not need a new string each loop.
    let mut bytes = base_string.into_bytes();
    let nb_original_bytes = bytes.len();
    bytes.resize_with(nb_original_bytes + max_padding as usize, Default::default);

    // We want to know how many hash per second we compute for diagnostic reasons
    let start_time_program = Instant::now();
    let mut start_time_chunk = Instant::now();

    // Some operations are expensive and make sense to execute them only every once in a while
    let check_every = 10_000_000;
    // Enumerate uses usize and cannot be changed so we need to keep track by hand of the number of loops
    let mut i: u128 = 0;
    'main_loop: for mut value in lower_limit..upper_limit {
        // Sometimes we break mid loop, so we cannot increase i at the end
        i += 1;

        // Given the current value we consider its binary representation.
        // We break this representation in chunks of length 7 so that each chunk is 0xxxxxxx.
        // This means that each chunk is a valid ASCII value in UTF8.
        // We append the new character to the original string and we keep looping until we consumed any non 0 chunk.
        // For example 28370 = 0b110111011010010 = [0b00000001, 0b01011101, 0b01010010] = [0x01, 0x5D, 0x52]
        // Note that the characters are appended in inverse order so for 28370 we append [0x52, 0x5D, 0x01]
        let mut current_offset = 0;
        loop {
            // take the last 7 bites
            let current_char = (value & 127u128) as u8;
            if current_char == 9 || current_char == 10 || current_char == 13 || current_char == 32 {
                continue 'main_loop;
            }
            bytes[nb_original_bytes + current_offset] = current_char;
            current_offset += 1;

            // discard the last 7 bits
            value >>= 7;
            if value == 0u128 {
                break;
            }
        }

        // Compute the hash and the leading zeros for the original string plus the chars that we just added
        let meaningful_bytes = &bytes[..nb_original_bytes + current_offset];
        let hash = sha1(meaningful_bytes);
        let count_leading_zeros = hash
            .iter()
            .try_fold(0, |acc, n| {
                if *n == 0u8 {
                    Ok(acc + 8)
                } else {
                    Err(acc + n.leading_zeros())
                }
            })
            .unwrap_or_else(|e| e);

        if count_leading_zeros >= (nb_zeros * BITS_IN_HEX) as u32 {
            is_found.store(true, Ordering::Relaxed);
            println!("thread {} found something", nb_thread);

            let output = String::from_utf8(
                meaningful_bytes[nb_original_bytes..nb_original_bytes + current_offset].to_vec(),
            )
            .unwrap();
            return Some(output);
        }

        // As we are ok if two different threads find a solution it is not worth it to check
        // the is_found flag each loop. So we do that only once in a while since it is an expensive operation
        // as it requires locking the value among the threads.
        if (i - 1) % check_every == 0 {
            // On the last thread, i.e. the slowest one, print some diagnostic just to see how long we should wait
            if nb_thread == nb_threads - 1 {
                let hash_per_sec =
                    (check_every as f64 / start_time_chunk.elapsed().as_secs_f64()).round() as u32;

                // In expectation we have one collision every (2^4)^nb_zeros
                let expected_duration_sec = 16u128.pow(nb_zeros as u32) as f64
                    / (hash_per_sec as u64 * nb_threads as u64) as f64;
                println!(
                    "Processing {:?} hash/s. The program is running for {:?}s. With this speed it should take {:?}s",
                    hash_per_sec, start_time_program.elapsed().as_secs() ,expected_duration_sec
                );
                start_time_chunk = Instant::now();
            }

            if is_found.load(Ordering::Relaxed) {
                println!("some other thread found something");
                return None;
            }
        }
    }

    println!("thread {} did not find anything", nb_thread);
    return None;
}

/// Given a valid UTF8 `base_string` it tries to generate another string `output` composed of only ASCII characters
/// such that sha1(`base_string` + `output`) has `nb_zeros` leading zeros.
/// It uses `nb_threads` threads for the computation.
///
/// # Arguments
///
/// * `base_string` - The base string that are given
/// * `nb_zeros` - The number of leading zeros in the hashing
/// * `nb_threads` - The total number of threads
pub fn generate_valid_string(
    base_string: &String,
    nb_zeros: u32,
    nb_threads: u32,
) -> Option<String> {
    // We need to expand the original string. Assuming that SHA1 is uniformly distributed over the inputs on average
    // one needs (2^4) ^ nb_zeros tried before finding a collision.
    // As we are restricting to use ASCII each char in a string give us 7 bites so we need at least this number of bites
    let max_padding = ((16u128.pow(nb_zeros) as f64).ln() / 7f64.ln()).ceil() as u32;
    let max_value = 128u128.pow(max_padding);

    // We need to signal when one thread found the string so that all the others can stop.
    let is_found = Arc::new(AtomicBool::new(false));
    let mut handles = vec![];

    let start_time = Instant::now();
    for nb_thread in 0..nb_threads {
        let is_found = Arc::clone(&is_found);
        let base_string = base_string.clone();
        let handle = thread::spawn(move || {
            // We divide the interval [0, max_value) in nb_threads chunks
            // and select the correct chuck for the current thread.
            let nb_element_thread = Integer::div_floor(&max_value, &(nb_threads as u128));
            let lower_limit: u128 = nb_thread as u128 * nb_element_thread as u128;
            let upper_limit: u128;
            if nb_thread != nb_threads {
                upper_limit = (nb_thread + 1) as u128 * nb_element_thread as u128;
            } else {
                upper_limit = max_value;
            }

            return generate_valid_string_one_thread(
                base_string,
                nb_zeros,
                max_padding,
                lower_limit,
                upper_limit,
                &is_found,
                nb_thread,
                nb_threads,
            );
        });
        handles.push(handle);
    }

    // Check if there is at least one valid output.
    // If multiple output exist we take the last one returned.
    let mut output: Option<String> = None;
    for handle in handles {
        if let Some(string) = handle.join().unwrap() {
            output = Some(string);
        };
    }

    let duration = start_time.elapsed();
    println!("Time elapsed is: {:?}", duration);

    return output;
}

#[cfg(test)]
mod tests {
    use super::{generate_valid_string, BITS_IN_HEX};
    use openssl::sha::sha1;
    use rand::{distributions::Alphanumeric, thread_rng, Rng};

    #[test]
    fn it_works_on_random_input() {
        let base_string: String = thread_rng()
            .sample_iter(&Alphanumeric)
            .take(30)
            .map(char::from)
            .collect();
        let nb_zeros = thread_rng().gen_range(0..6);
        let nb_threads = 10;
        let extra_string = generate_valid_string(&base_string, nb_zeros, nb_threads).unwrap();

        let new_string = base_string.clone() + &extra_string;
        let hash = sha1(&new_string.into_bytes());
        let count_leading_zeros = hash
            .iter()
            .try_fold(0, |acc, n| {
                if *n == 0u8 {
                    Ok(acc + 8)
                } else {
                    Err(acc + n.leading_zeros())
                }
            })
            .unwrap_or_else(|e| e);

        assert!(count_leading_zeros >= (nb_zeros * BITS_IN_HEX));
    }
}
