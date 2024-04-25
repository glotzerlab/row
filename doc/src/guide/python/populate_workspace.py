import signac

N = 10

project = signac.get_project()

for x in range(N):
    job = project.open_job({'x': x}).init()
