# ANCHOR: row_init
# $ row init sum_squares --signac  # TODO: Write row init
mkdir -p sum_squares/workspace
cd sum_squares
echo 'workspace.value_file = "signac_statepoint.json"' > workflow.toml
# ANCHOR_END: row_init

cp ../populate_workspace.py .
# ANCHOR: signac_init
signac init
python populate_workspace.py
# ANCHOR_END: signac_init

cp ../signac-workflow.toml workflow.toml
cp ../actions.py .

# ANCHOR: submit_square
row submit --action square
# ANCHOR_END: submit_square

# ANCHOR: submit_sum
row submit --action sum
# ANCHOR_END: submit_sum
