#!/bin/bash

declare -i num_argu=$#

if [ $num_argu -eq 2 ]; then
  sid=$[$1+1]
  nid=$[($2+1)*2]
  curl "http://127.0.0.1:70$sid$nid/blockchain/log"
else 
  echo "the number of argumenst is not valid"
fi
