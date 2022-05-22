# for testing the performance between original and optimized version
echo "********original*********"
for i in {1..10}
do
    time taskset -c 0 ./pastel-original distinct 2000 >/dev/null
done

echo -e "\n******modified*******\n"
for i in {1..10}
do
    time taskset -c 0 ./pastel-modified distinct 2000 >/dev/null
done



