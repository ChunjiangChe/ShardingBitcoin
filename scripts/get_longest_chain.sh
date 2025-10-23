#!/bin/bash

declare -i num_argu=$#

if [ $num_argu -eq 1 ]; then
  curl http://127.0.0.1:700$1/blockchain/longest-chain
else 
  echo "the number of argumenst is not valid"
fi
