cargo run --release -- -- 'WAIT=$(($RANDOM / 10000)); echo $WAIT; sleep ${WAIT}s'
