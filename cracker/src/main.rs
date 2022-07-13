use cracker::rust::generate_valid_string;
use openssl::sha::sha1;
fn main() {
    let original_string = String::from("aaaa");
    let nb_zeros = 5;
    let nb_threads = 10;

    let result = generate_valid_string(&original_string, nb_zeros, nb_threads);

    match result {
        Some(string) => {
            let total_output = format!("{}{}", original_string, string);
            println!("{}", total_output);
            println!("{:X?}", sha1(total_output.as_bytes()));
        }
        None => println!("Nothing found"),
    }
}
