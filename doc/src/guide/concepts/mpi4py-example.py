"""Example actions.py using mpi4py."""

import argparse

# ANCHOR: action
import mpi4py.MPI
import signac


def action_implementation(job):
    """Implement the action on a single job."""
    # Your code that operates on one directory goes here.


def action(*jobs):
    """Process jobs in parallel with the mpi4py package.

    The number of ranks must be equal to the number of directories.
    """
    if mpi4py.MPI.COMM_WORLD.Get_size() != len(jobs):
        message = 'Number of ranks does not match number of directories.'
        raise RuntimeError(message)

    rank = mpi4py.MPI.COMM_WORLD.Get_rank()
    action_implementation(jobs[rank])
    # ANCHOR_END: action


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
