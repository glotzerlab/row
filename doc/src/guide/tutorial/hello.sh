# ANCHOR: init
row init hello-workflow
cd hello-workflow
# ANCHOR_END: init

# ANCHOR: create
mkdir workspace/directory0 workspace/directory1 workspace/directory2
# ANCHOR_END: create

cp ../hello-workflow.toml workflow.toml
# ANCHOR: submit
row submit
# ANCHOR_END: submit

# ANCHOR: status1
row show status
# ANCHOR_END: status1

cp ../goodbye-workflow.toml workflow.toml
# ANCHOR: status2
row show status
# ANCHOR_END: status2

# ANCHOR: submit2
row submit directory1
# ANCHOR_END: submit2

# ANCHOR: directories_hello
row show directories hello
# ANCHOR_END: directories_hello

# ANCHOR: submit3
row submit --action goodbye
# ANCHOR_END: submit3
