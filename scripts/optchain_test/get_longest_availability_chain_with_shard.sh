#!/bin/bash

declare -i num_argu=$#

if [ $num_argu -eq 3 ]; then
  curl http://127.0.0.1:70$1$2/blockchain/availability-chain-with-shard?shard-id=$3
else 
  echo "the number of argumenst is not valid"
fi
