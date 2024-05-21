"""Example actions.py using multiprocessing."""

import argparse
import multiprocessing
import os

import signac


# ANCHOR: action
def action_implementation(job):
    """Implement the action on a single job."""
    # Your code that operates on one directory goes here.


def action(*jobs):
    """Process any number of jobs in parallel with the multiprocessing package."""
    processes = os.environ.get('ACTION_THREADS_PER_PROCESS', multiprocessing.cpu_count())
    if hasattr(os, 'sched_getaffinity'):
        processes = len(os.sched_getaffinity(0))

    with multiprocessing.Pool(processes=processes) as p:
        p.map(action_implementation, jobs)


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
