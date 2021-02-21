use super::rust::generate_valid_string;
use pyo3::prelude::*;

/// Python module that exposes rusts functions for fast calculation of strings with a given number of
/// leading zeros in their sha1 hashing.
#[pymodule]
pub fn libcracker(_py: Python, m: &PyModule) -> PyResult<()> {
    /// Given a valid UTF8 `base_string` it tries to generate another string `output` composed of only ASCII characters
    /// such that sha1(`base_string` + `output`) has `nb_zeros` leading zeros.
    /// It uses `nb_threads` threads for the computation.
    ///
    /// # Arguments
    ///
    /// * `base_string` - The base string that are given
    /// * `nb_zeros` - The number of leading zeros in the hashing
    /// * `nb_threads` - The total number of threads
    #[pyfn(m, "generate_valid_string")]
    #[text_signature = "(base_string, nb_zeros, nb_threads, /)"]
    fn generate_valid_string_python(
        base_string: String,
        nb_zeros: u32,
        nb_threads: u32,
    ) -> PyResult<String> {
        let result = generate_valid_string(&base_string, nb_zeros, nb_threads);

        match result {
            Some(string) => {
                println!("{:X?}", string.as_bytes());
                Ok(string)
            }
            None => Ok(String::from("")),
        }
    }

    Ok(())
}
