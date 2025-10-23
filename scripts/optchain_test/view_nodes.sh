#!/bin/bash

declare -i num_argu=$#

if [ $num_argu -eq 2 ]; then
  while true; do
    printf "\033c"
    for ((i=0; i<$1; i++))
    do
      for ((j=0; j<$2; j++))
      do
        echo "node $j in shard $i:"
        echo "proposer chain"  
        ./get_longest_proposer_chain.sh $i $j
        echo ""
        echo "availability chains"
        for ((k=0; k<$1; k++))
        do
          ./get_longest_availability_chain_with_shard.sh $i $j $k
          echo ""
        done
 #      ./view_mempool.sh $i
 #      echo ""
      done
     #./get_tx_in_longest_chain.sh $i
     #echo ""
     echo ""
    done
    sleep 1
  done
else 
  echo "the number of arguments is not valid"
fi