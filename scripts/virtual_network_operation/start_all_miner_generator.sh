#!/bin/bash

declare -i num_argu=$#

if [ $num_argu -eq 5 ]; then
  for ((i=0; i<$1; i++))
  do
    for ((j=0; j<$2; j++))
    do
      ./start_miner.sh $i $j $3 
      ./start_tx_generator.sh $i $j $4 
      sleep 1
    done
  done
  ./view_node.sh $1 $2 $5
else 
  echo "the number of arguments is not valid"
fi
