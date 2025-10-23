#!/bin/bash

declare -i num_argu=$#

if [ $num_argu -eq 3 ]; then
  c=0
  while [ $c -lt $3 ]; do
    c=$[$c+1]
    printf "\033c"
    echo "progress: $c seconds"
    #for ((i=0; i<$1; i++))
    #do
    #  for ((j=0; j<$2; j++))
    #  do
    #    echo "node $j in $i:"
    #    for ((k=0; k<$1; k++))
    #    do
    #      ./get_longest_chain_with_shard.sh $i $j $k
    #      echo ""
    #    done
 #  #    ./view_mempool.sh $i
 #  #    echo ""
    #  done
    # #./get_tx_in_longest_chain.sh $i
    # #echo ""
    # echo ""
    #done
    sleep 1
  done
  ./end_node.sh $1 $2
else 
  echo "the number of arguments is not valid"
fi
