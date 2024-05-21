"""Example actions.py using mpi4py."""

import argparse
import os

import hoomd
import signac


# ANCHOR: action
def action_implementation(job, communicator):
    """Implement the action on a single job."""
    # Your HOOMD-blue simulation goes here. Use the given communicator. For example:
    # cpu = hoomd.device.CPU(communicator=communicator)
    # simulation = hoomd.Simulation(devices=cpu)


def action(*jobs):
    """Execute actions on directories in parallel using HOOMD-blue."""
    processes_per_directory = os.environ['ACTION_PROCESSES_PER_DIRECTORY']
    communicator = hoomd.communicator.Communicator(ranks_per_partition=processes_per_directory)
    action_implementation(jobs[communicator.partition])


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
