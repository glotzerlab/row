"""Implement actions."""

import argparse
import os

import signac


def square(*jobs):
    """Implement the square action.

    Squares the value `x` in each job's statepoint and writes the output to
    `square.out` when complete.
    """
    for job in jobs:
        # If the product already exists, there is no work to do.
        if job.isfile('square.out'):
            continue

        # Open a temporary file so that the action is not completed early or on error.
        with open(job.fn('square.out.in_progress'), 'w') as file:
            x = job.cached_statepoint['x']
            file.write(f'{x**2}')

        # Done! Rename the temporary file to the product file.
        os.rename(job.fn('square.out.in_progress'), job.fn('square.out'))


def compute_sum(*jobs):
    """Implement the compute_sum action.

    Prints the sum of `square.out` from each job directory.
    """
    total = 0
    for job in jobs:
        with open(job.fn('square.out')) as file:
            total += int(file.read())

    print(total)


if __name__ == '__main__':
    # Parse the command line arguments: python action.py --action <ACTION> [DIRECTORIES]
    parser = argparse.ArgumentParser()
    parser.add_argument('--action', required=True)
    parser.add_argument('directories', nargs='+')
    args = parser.parse_args()

    # Open the signac jobs
    project = signac.get_project()
    jobs = [project.open_job(id=directory) for directory in args.directories]

    # Call the action
    globals()[args.action](*jobs)
