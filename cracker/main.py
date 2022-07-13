

from libcracker import generate_valid_string

original_string = "aaaa"
nb_zeros = 5
nb_threads = 10

print(generate_valid_string(original_string, nb_zeros, nb_threads))
