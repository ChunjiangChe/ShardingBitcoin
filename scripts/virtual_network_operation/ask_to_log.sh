#!/bin/bash

declare -i num_argu=$#

if [ $num_argu -eq 2 ]; then
  sid=$[$1+1]
  nid=$[($2+1)*2]
  curl "http://10.0.$sid.$nid:7000/blockchain/log"
else 
  echo "the number of argumenst is not valid"
fi
