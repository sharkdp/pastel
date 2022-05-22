# export RAYON_NUM_THREADS=1 taskset -c 0-3
# time ./target/release/grex --file test.txt >/dev/null
echo "********original*********"
for i in {1..10}
do
    # time taskset -c 0 ./search_and_replace-original  adadfadfadfasd s /home/psu/yyang/rust_app/svgbob/10000.txt >/dev/null
    time taskset -c 0 ./pastel-original distinct 2000 >/dev/null
done

echo -e "\n******modified*******\n"
for i in {1..10}
do
    # time taskset -c 0 ./search_and_replace-m3  adadfadfadfasd s /home/psu/yyang/rust_app/svgbob/10000.txt >/dev/null
    time taskset -c 0 ./pastel-m1 distinct 2000 >/dev/null
done



