#!/bin/bash
declare -i num_argu=$#

if [ $num_argu -eq 3 ]; then
  sid=$[$1+1]
  nid=$[($2+1)*2]
  curl http://10.0.$sid.$nid:7000/tx-generator/start?theta=$3
else 
  echo "the number of arguments is not valid"
fi
