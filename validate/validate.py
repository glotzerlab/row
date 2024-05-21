"""Validate that row jobs correctly submit to supported clusters.

Create a row project directory that validates cluster job submissions.

To test a built-in cluster, run:
* `python validate.py init` (add `--account=<your-account>` if needed).
* `row submit`
* Wait for jobs to complete....
* `cat *.out`
* `cat <cluster-name>/output/*.out`

The submitted jobs check serial, threaded, MPI, MPI+threads, GPU, and
MPI+GPU jobs to ensure that they run successfully and are scheduled to the
selected resources. Check `*.out` for any error messages. Then check
`<cluster-name>/output/*.out` for `ERROR`, `WARN`, and `PASSED` lines.
`validate.py` prints: `ERROR` when the launched job has a more restrictive
binding than requested; `WARN` when the binding is less restrictive; and
'PASSED' when there are at least enough  available resources to execute.

To test a non-built-in cluster:
* Configure your cluster in `cluster.toml`.
* If the default `srun` launcher is not sufficient, configure your MPI
  launcher in `launchers.toml.
* Add a key to the `CLUSTERS` dictionary in `validate.py` that describes
  your cluster.
* Then follow the steps above.
"""

import argparse
import collections
import os
import re
import socket
import subprocess
import textwrap
from pathlib import Path

# import numpy to ensure that it does not improperly modify the cpuset
# https://stackoverflow.com/questions/15639779/
import numpy  # noqa: F401

# Set the number of cpus and gpus per node in the *default* partitions that row selects.
# Testing non-default partitions is beyond the scope of this script. Set to 0 to prevent
# CPU and/or GPU jobs from executing.
Cluster = collections.namedtuple(
    'Cluster',
    ('cpus_per_node', 'gpus_per_node', 'gpu_arch', 'has_shared'),
    defaults=(None, None, 'nvidia', True),
)
CLUSTERS = {
    'andes': Cluster(cpus_per_node=32, gpus_per_node=0, gpu_arch='none', has_shared=False),
    'anvil': Cluster(cpus_per_node=128, gpus_per_node=0, gpu_arch='nvidia'),
    'delta': Cluster(cpus_per_node=128, gpus_per_node=4, gpu_arch='nvidia'),
    'frontier': Cluster(cpus_per_node=0, gpus_per_node=8, gpu_arch='amd', has_shared=False),
    'greatlakes': Cluster(cpus_per_node=36, gpus_per_node=2, gpu_arch='nvidia'),
}

N_THREADS = 4
N_GPUS = 2
N_PROCESSES = 4
N_NODES = 2


def get_cluster_name():
    """Get the current cluster name."""
    result = subprocess.run(
        ['row', 'show', 'cluster', '--name'], capture_output=True, check=True, text=True
    )
    return result.stdout.strip()


def get_nvidia_gpus():
    """Get the assigned NVIDIA GPUs."""
    result = subprocess.run(
        ['nvidia-smi', '--list-gpus'], capture_output=True, check=True, text=True
    )

    gpus = []
    migs = []
    pattern = re.compile(r'.*\(UUID: (GPU|MIG)-(.*)\)')

    for line in result.stdout.splitlines():
        match = pattern.match(line)
        if not match:
            message = f'Unexpected output from nvidia_smi: {line}.'
            raise RuntimeError(message)

        if match.group(1) == 'GPU':
            gpus.append(match.group(2))
        elif match.group(1) == 'MIG':
            migs.append(match.group(2))
        else:
            message = 'Unexpected match {match.group(1)}.'
            raise RuntimeError(message)

    if len(migs):
        return migs

    return gpus


def get_amd_gpus():
    """Get the assigned AMD GPUs."""
    result = subprocess.run(
        ['rocm-smi', '--showuniqueid'], capture_output=True, check=True, text=True
    )

    gpus = []
    pattern = re.compile(r'.*Unique ID: (.*)$')

    for line in result.stdout.splitlines():
        print(line)
        match = pattern.match(line)

        if match:
            gpus.append(match.group(1))

    return gpus


def init(account, setup):
    """Initialize the project."""
    cluster_name = get_cluster_name()
    if cluster_name not in CLUSTERS:
        message = f'Unsupported cluster {cluster_name}.'
        raise RuntimeError(message)
    cluster = CLUSTERS.get(cluster_name)

    # Create the workspace
    workspace = Path(cluster_name)
    workspace.mkdir(exist_ok=True)
    output = workspace / Path('output')
    output.mkdir(exist_ok=True)

    # Create workflow.toml
    with open(file='workflow.toml', mode='w', encoding='utf-8') as workflow:
        workflow.write(
            textwrap.dedent(f"""\
            [workspace]
            path = "{cluster_name}"

            [default.action.submit_options.{cluster_name}]
            """)
        )

        if account is not None:
            workflow.write(
                textwrap.dedent(f"""\
                account = "{account}"
                """)
            )

        if setup is not None:
            workflow.write(
                textwrap.dedent(f"""\
                setup = "{setup}"
                """)
            )

        if cluster.cpus_per_node >= 1 and cluster.has_shared:
            workflow.write(
                textwrap.dedent("""
                [[action]]
                name = "serial"
                command = "python validate.py execute serial {directory}"
                products = ["serial.out"]
                [action.resources]
                processes.per_submission = 1
                walltime.per_submission = "00:05:00"
                """)
            )

        if cluster.cpus_per_node >= N_THREADS and cluster.has_shared:
            workflow.write(
                textwrap.dedent(f"""
                [[action]]
                name = "threads"
                command = "python validate.py execute threads {{directory}}"
                products = ["threads.out"]
                [action.resources]
                processes.per_submission = 1
                threads_per_process = {N_THREADS}
                walltime.per_submission = "00:05:00"
                """)
            )

        if cluster.cpus_per_node >= N_PROCESSES and cluster.has_shared:
            workflow.write(
                textwrap.dedent(f"""
                [[action]]
                name = "mpi_subnode"
                command = "python validate.py execute mpi_subnode {{directory}}"
                products = ["mpi_subnode.out"]
                launchers = ["mpi"]
                [action.resources]
                processes.per_submission = {N_PROCESSES}
                walltime.per_submission = "00:05:00"
                """)
            )

        if cluster.cpus_per_node >= N_PROCESSES * N_THREADS and cluster.has_shared:
            workflow.write(
                textwrap.dedent(f"""
                [[action]]
                name = "mpi_threads_subnode"
                command = "python validate.py execute mpi_threads_subnode {{directory}}"
                products = ["mpi_threads_subnode.out"]
                launchers = ["mpi"]
                [action.resources]
                processes.per_submission = {N_PROCESSES}
                threads_per_process = {N_THREADS}
                walltime.per_submission = "00:05:00"
                """)
            )

        if cluster.cpus_per_node >= 1:
            workflow.write(
                textwrap.dedent(f"""
                [[action]]
                name = "mpi_multinode"
                command = "python validate.py execute mpi_multinode {{directory}}"
                products = ["mpi_multinode.out"]
                launchers = ["mpi"]
                [action.resources]
                processes.per_submission = {N_NODES * cluster.cpus_per_node}
                walltime.per_submission = "00:05:00"
                """)
            )

        if cluster.cpus_per_node >= 1 and (cluster.cpus_per_node % N_THREADS) == 0:
            workflow.write(
                textwrap.dedent(f"""
                [[action]]
                name = "mpi_threads_multinode"
                command = "python validate.py execute mpi_threads_multinode {{directory}}"
                products = ["mpi_threads_multinode.out"]
                launchers = ["mpi"]
                [action.resources]
                processes.per_submission = {N_NODES * cluster.cpus_per_node // N_THREADS}
                threads_per_process = {N_THREADS}
                walltime.per_submission = "00:05:00"
                """)
            )

        if cluster.gpus_per_node >= 1 and cluster.gpu_arch == 'nvidia' and cluster.has_shared:
            workflow.write(
                textwrap.dedent("""
                [[action]]
                name = "nvidia_gpu"
                command = "python validate.py execute nvidia_gpu {directory}"
                products = ["nvidia_gpu.out"]
                [action.resources]
                processes.per_submission = 1
                gpus_per_process = 1
                walltime.per_submission = "00:05:00"
                """)
            )

        if cluster.gpus_per_node >= N_GPUS and cluster.gpu_arch == 'nvidia' and cluster.has_shared:
            workflow.write(
                textwrap.dedent(f"""
                [[action]]
                name = "nvidia_gpus"
                command = "python validate.py execute nvidia_gpus {{directory}}"
                products = ["nvidia_gpus.out"]
                [action.resources]
                processes.per_submission = 1
                gpus_per_process = {N_GPUS}
                walltime.per_submission = "00:05:00"
                """)
            )

        if cluster.gpus_per_node >= 1 and cluster.gpu_arch == 'nvidia' and cluster.has_shared:
            workflow.write(
                textwrap.dedent(f"""
                [[action]]
                name = "mpi_nvidia_gpus"
                command = "python validate.py execute mpi_nvidia_gpus {{directory}}"
                products = ["mpi_nvidia_gpus.out"]
                launchers = ["mpi"]
                [action.resources]
                processes.per_submission = {N_PROCESSES}
                gpus_per_process = 1
                walltime.per_submission = "00:05:00"
                """)
            )

        if cluster.gpus_per_node >= 1 and cluster.gpu_arch == 'amd':
            workflow.write(
                textwrap.dedent(f"""
                [[action]]
                name = "mpi_wholenode_amd_gpus"
                command = "python validate.py execute mpi_wholenode_amd_gpus {{directory}}"
                products = ["mpi_wholenode_amd_gpus.out"]
                launchers = ["mpi"]
                [action.resources]
                processes.per_submission = {cluster.gpus_per_node}
                gpus_per_process = 1
                walltime.per_submission = "00:05:00"
                """)
            )


def serial(directory):
    """Validate serial jobs."""
    action_cluster = os.environ['ACTION_CLUSTER']

    output_path = Path(action_cluster) / Path(directory) / Path('serial.out')
    with output_path.open(mode='w', encoding='utf-8') as output:
        row_cluster = get_cluster_name()
        if action_cluster != row_cluster:
            print(
                'ERROR: `row cluster --name` does not match at submission '
                f'({action_cluster}) and execution ({row_cluster})',
                file=output,
            )

        cpuset = os.sched_getaffinity(0)
        if len(cpuset) > 1:
            print(
                f'WARN: Allowed to run on more cpus than requested: {cpuset}.',
                file=output,
            )
        elif len(cpuset) == 1:
            print(f'PASSED: {cpuset}', file=output)
        else:
            print('ERROR: unknown.', file=output)


def threads(directory):
    """Validate threaded jobs."""
    action_cluster = os.environ['ACTION_CLUSTER']

    output_path = Path(action_cluster) / Path(directory) / Path('threads.out')
    with output_path.open(mode='w', encoding='utf-8') as output:
        cpuset = os.sched_getaffinity(0)
        if len(cpuset) > N_THREADS:
            print(
                f'WARN: Allowed to run on more cpus than requested: {cpuset}.',
                file=output,
            )

        if len(cpuset) < N_THREADS:
            print(f'ERROR: Not allowed to run on requested cpus: {cpuset}.', file=output)
        elif len(cpuset) == N_THREADS:
            print(f'PASSED: {cpuset}', file=output)


def check_mpi(directory, n_processes, n_threads, n_hosts, name, n_gpus=0, gpu_arch='nvidia'):
    """Validate that MPI jobs run on less than a whole node.

    Ensure that each process has n_threads threads.
    """
    from mpi4py import MPI

    action_cluster = os.environ['ACTION_CLUSTER']

    comm = MPI.COMM_WORLD

    if comm.Get_size() != n_processes:
        message = f'ERROR: incorrect number of processes {comm.Get_size()}.'
        raise RuntimeError(message)

    cpusets = comm.gather(os.sched_getaffinity(0), root=0)
    hostnames = comm.gather(socket.gethostname(), root=0)
    gpus = []
    if n_gpus > 0 and gpu_arch == 'nvidia':
        gpus = comm.gather(get_nvidia_gpus(), root=0)
    if n_gpus > 0 and gpu_arch == 'amd':
        gpus = comm.gather(get_amd_gpus(), root=0)

    if comm.Get_rank() == 0:
        cpuset_sizes = [len(s) for s in cpusets]
        gpu_sizes = [len(g) for g in gpus]

        output_path = Path(action_cluster) / Path(directory) / Path(name + '.out')
        with output_path.open(mode='w', encoding='utf-8') as output:
            if len(set(hostnames)) > n_hosts:
                print(
                    f'WARN: Executing on more than {n_hosts} host(s): {set(hostnames)}.',
                    file=output,
                )

            if len(set(cpuset_sizes)) != 1:
                print(f'WARN: cpusets have different sizes: {cpusets}.', file=output)

            if max(cpuset_sizes) > n_threads:
                print(
                    f'WARN: Allowed to run on more cpus than requested: {cpusets}.',
                    file=output,
                )

            if n_gpus > 0:
                if len(set(gpu_sizes)) != 1:
                    print(f'WARN: gpus have different sizes: {gpus}.', file=output)

                if max(gpu_sizes) > n_gpus:
                    print(
                        f'WARN: Allowed to run on more GPUs than requested: {gpus}.',
                        file=output,
                    )

            if min(cpuset_sizes) < n_threads:
                print(
                    f'ERROR: Not allowed to run on requested cpus: {cpusets}.',
                    file=output,
                )
            elif len(set(hostnames)) < n_hosts:
                print(
                    f'ERROR: Executing on fewer than {n_hosts} hosts: {set(hostnames)}.',
                    file=output,
                )
            elif n_gpus > 0 and min(gpu_sizes) < n_gpus:
                print(
                    f'ERROR: Not allowed to run on requested GPUs: {gpus}.',
                    file=output,
                )
            else:
                print(f'PASSED: {set(hostnames)} {cpusets} {gpus}', file=output)


def mpi_subnode(directory):
    """Check that MPI allocates processes correctly on one node."""
    check_mpi(directory, n_processes=N_PROCESSES, n_threads=1, n_hosts=1, name='mpi_subnode')


def mpi_threads_subnode(directory):
    """Check that MPI allocates processes and threads correctly on one node."""
    check_mpi(
        directory,
        n_processes=N_PROCESSES,
        n_threads=N_THREADS,
        n_hosts=1,
        name='mpi_threads_subnode',
    )


def mpi_nvidia_gpus(directory):
    """Check that MPI allocates GPUs correctly."""
    check_mpi(
        directory,
        n_processes=N_PROCESSES,
        n_threads=1,
        n_hosts=1,
        name='mpi_nvidia_gpus',
        n_gpus=1,
        gpu_arch='nvidia',
    )


def mpi_multinode(directory):
    """Check that MPI allocates processes correctly on multiple nodes."""
    cluster_name = get_cluster_name()
    cluster = CLUSTERS.get(cluster_name)

    check_mpi(
        directory,
        n_processes=cluster.cpus_per_node * N_NODES,
        n_threads=1,
        n_hosts=N_NODES,
        name='mpi_multinode',
    )


def mpi_threads_multinode(directory):
    """Check that MPI allocates processes and threads correctly on multiple nodes."""
    cluster_name = get_cluster_name()
    cluster = CLUSTERS.get(cluster_name)

    check_mpi(
        directory,
        n_processes=cluster.cpus_per_node * N_NODES // N_THREADS,
        n_threads=N_THREADS,
        n_hosts=N_NODES,
        name='mpi_threads_multinode',
    )


def check_nvidia_gpu(directory, n_gpus, name):
    """Validate threaded GPU jobs."""
    action_cluster = os.environ['ACTION_CLUSTER']

    output_path = Path(action_cluster) / Path(directory) / Path(name + '.out')
    with output_path.open(mode='w', encoding='utf-8') as output:
        gpus = get_nvidia_gpus()
        if len(gpus) > n_gpus:
            print(
                f'WARN: Allowed to run on more GPUs than requested: {gpus}.',
                file=output,
            )

        if len(gpus) < n_gpus:
            print(f'ERROR: Not allowed to run on requested GPUs: {gpus}.', file=output)
        elif len(gpus) == n_gpus:
            print(f'PASSED: {gpus}', file=output)


def nvidia_gpu(directory):
    """Validate single GPU jobs."""
    check_nvidia_gpu(directory, n_gpus=1, name='nvidia_gpu')


def nvidia_gpus(directory):
    """Validate multi-GPU jobs."""
    check_nvidia_gpu(directory, n_gpus=N_GPUS, name='nvidia_gpus')


def mpi_wholenode_amd_gpus(directory):
    """Check that MPI allocates processes correctly to all AMD GPUs on one node."""
    cluster_name = get_cluster_name()
    cluster = CLUSTERS.get(cluster_name)

    check_mpi(
        directory,
        n_processes=cluster.gpus_per_node,
        n_threads=1,
        n_hosts=1,
        name='mpi_wholenode_amd_gpus',
        n_gpus=1,
        gpu_arch='amd',
    )


if __name__ == '__main__':
    # Parse the command line arguments:
    # * `python execute <ACTION> [DIRECTORIES]`
    # * `python init --account <ACCOUNT>`
    parser = argparse.ArgumentParser()
    subparsers = parser.add_subparsers(dest='subparser_name', required=True)

    execute_parser = subparsers.add_parser('execute')
    execute_parser.add_argument('action')
    execute_parser.add_argument('directories', nargs='+')

    init_parser = subparsers.add_parser('init')
    init_parser.add_argument('--account')
    init_parser.add_argument('--setup')
    args = parser.parse_args()

    if args.subparser_name == 'init':
        init(account=args.account, setup=args.setup)
    elif args.subparser_name == 'execute':
        globals()[args.action](*args.directories)
    else:
        message = f'Unknown subcommand {args.subparser_name}'
        raise ValueError(message)
