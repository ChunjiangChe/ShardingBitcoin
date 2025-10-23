#!/bin/bash
declare -i num_argu=$#

if [ $num_argu -eq 1 ]; then
  curl http://127.0.0.1:700$1/mempool/get_txs
else 
  echo "the number of arguments is not valid"
fi
