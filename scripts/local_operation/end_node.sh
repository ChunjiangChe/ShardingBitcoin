#!/bin/bash

declare -i num_argu=$#

if [ $num_argu -eq 2 ]; then
  for ((i=0; i<$1; i++))
  do
    for ((j=0; j<$2; j++))
    do
      ./ask_to_log.sh $i $j
      #./get_tx_in_longest_chain.sh $i
      #echo ""
      echo ""
    done
  done
  sleep 5
  sudo pkill -f "bitcoin"
else 
  echo "the number of arguments is not valid"
fi
