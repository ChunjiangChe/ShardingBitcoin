#!/bin/bash
for ((i=0; i<5; i++))
do
  for ((j=0; j<5; j++))
  do
    node_id=$[i*5+j]
    cd nodes
    ./start_node_$node_id.sh 2>&1 | tee ../../../../../log/manifoldchain/exper_0/iter_1/exec_log/$node_id.log &
    cd ..
  done
done