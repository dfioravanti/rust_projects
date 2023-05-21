import multiprocessing

from libcracker import generate_valid_string

original_string = "aaaa"
nb_zeros = 8

print(generate_valid_string(original_string, nb_zeros, multiprocessing.cpu_count()))
