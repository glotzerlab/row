mkdir workspace || exit 1
cd workspace

for i in {1..100}
do
  mkdir dir$i || exit 1
  v=$((1 + RANDOM % 1000))
  echo "{\"value\": $v}" > dir$i/value.json || exit 1
done

row submit --cluster none --action=initialize -n 5 --yes || exit 1

row submit --cluster none --action=step1 -n 1 --yes || exit 1

row submit --action=step1 -n 1 --yes || exit 1

row show status || exit 1

row show directories step1 -n 3 --value="/value"
