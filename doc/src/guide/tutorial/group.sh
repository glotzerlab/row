
# ANCHOR: init
row init group-workflow
cd group-workflow/workspace

mkdir directory1 && echo '{"type": "point", "x": 0, "y": 10}' > directory1/value.json
mkdir directory2 && echo '{"type": "point", "x": 3, "y": 8}' > directory2/value.json
mkdir directory3 && echo '{"type": "point", "x": 0, "y": 4}' > directory3/value.json
mkdir directory4 && echo '{"type": "point", "x": 3, "y": 11}' > directory4/value.json
mkdir directory5 && echo '{"type": "point", "x": 0, "y": -3}' > directory5/value.json
mkdir directory6 && echo '{"type": "point", "x": 2, "y": 2}' > directory6/value.json

mkdir directory7 && echo '{"type": "letter", "letter": "alpha"}' > directory7/value.json
mkdir directory8 && echo '{"type": "letter", "letter": "beta"}' > directory8/value.json
mkdir directory9 && echo '{"type": "letter", "letter": "gamma"}' > directory9/value.json
# ANCHOR_END: init

cd ..

cp ../group-workflow2.toml workflow.toml

# ANCHOR: show_point1
row show directories --action process_point --value /type --value /x --value /y
# ANCHOR_END: show_point1

# ANCHOR: show_letter
row show directories --action process_letter --value /type --value /letter
# ANCHOR_END: show_letter

cp ../group-workflow3.toml workflow.toml

# ANCHOR: show_point2
row show directories --action process_point --value /type --value /x --value /y
# ANCHOR_END: show_point2

cp ../group-workflow4.toml workflow.toml

# ANCHOR: show_point3
row show directories --action process_point --value /type --value /x --value /y
# ANCHOR_END: show_point3

# ANCHOR: submit
row submit -a process_point
# ANCHOR_END: submit

cp ../group-workflow5.toml workflow.toml

# ANCHOR: show_point4
row show directories --action process_point --value /type --value /x --value /y
# ANCHOR_END: show_point4
