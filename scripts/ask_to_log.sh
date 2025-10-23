#!/bin/bash

declare -i num_argu=$#

if [ $num_argu -eq 2 ]; then
  curl "http://127.0.0.1:70$1$2/blockchain/log"
else 
  echo "the number of argumenst is not valid"
fi
