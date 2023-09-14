# from tqdm import tqdm

import random
import argparse
import concurrent.futures

AMINO_ACIDS = [
    'A', 'R', 'N', 'D', 'C', 'E', 'Q', 'G', 'H', 'I',
    'L', 'K', 'M', 'F', 'P', 'S', 'T', 'W', 'Y', 'V'
]

def generate_peptide(length):
    return ''.join(random.choices(AMINO_ACIDS, k = length))

def generate_peptides(amount, min_length = 5, max_length = 50):
    return [generate_peptide(random.randint(min_length, max_length)) for _ in range(amount)]

parser = argparse.ArgumentParser()

parser.add_argument('--min-length', type = int, default = 5)
parser.add_argument('--max-length', type = int, default = 50)
parser.add_argument('--amount', type = int, default = 1_000_000)
parser.add_argument('--batch-size', type = int, default = 1_000)

args = parser.parse_args()

if __name__ == '__main__':
    amount_of_batches = args.amount // args.batch_size

    with concurrent.futures.ProcessPoolExecutor() as executor:
        futures = [
            executor.submit(generate_peptides, args.batch_size, args.min_length, args.max_length) 
            for _ in range(amount_of_batches)
        ]
        
        for future in concurrent.futures.as_completed(futures):
            print('\n'.join(future.result()))
