cargo run --release -- -c 3 -- 'WAIT=$(($RANDOM / 10000)); echo $WAIT; sleep ${WAIT}s'
