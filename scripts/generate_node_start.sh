#!/bin/bash
cd ../
declare -i num_argu=$#
if [ $num_argu -lt 2 ]; then
  echo "arguments not enough"
elif [ $num_argu -eq 2 ]; then 
  cargo run -- -vvv --p2p 127.0.0.1:600$1 --api 127.0.0.1:700$1 --shardId $2
elif [ $num_argu -eq 3 ]; then
  cargo run -- -vvv --p2p 127.0.0.1:600$1 --api 127.0.0.1:700$1 -c 127.0.0.1:600$2 --shardId $3
elif [ $num_argu -eq 4 ]; then
  cargo run -- -vvv --p2p 127.0.0.1:600$1 --api 127.0.0.1:700$1 -c 127.0.0.1:600$2 -c 127.0.0.1:600$3 --shardId $4
else 
  echo "too much arguments"
fi
