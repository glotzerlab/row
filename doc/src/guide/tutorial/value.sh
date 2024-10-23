
# ANCHOR: init
row init value-workflow
cd value-workflow/workspace

mkdir directory1 && echo '{"seed": 0, "pressure": 1.5}' > directory1/value.json
mkdir directory2 && echo '{"seed": 1, "pressure": 1.5}' > directory2/value.json
mkdir directory3 && echo '{"seed": 0, "pressure": 2.1}' > directory3/value.json
mkdir directory4 && echo '{"seed": 1, "pressure": 2.1}' > directory4/value.json

# ANCHOR_END: init

cd ..

cp ../value-workflow.toml workflow.toml

# ANCHOR: submit
row submit
# ANCHOR_END: submit
