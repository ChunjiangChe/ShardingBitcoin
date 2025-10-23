#!/bin/bash
declare -i num_argu=$#

if [ $num_argu -eq 3 ]; then
  sid=$[$1+1]
  nid=$[($2+1)*2]
  curl http://127.0.0.1:70$sid$nid/tx-generator/start?theta=$3
else 
  echo "the number of arguments is not valid"
fi
