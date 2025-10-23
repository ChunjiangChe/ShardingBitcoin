#!/bin/bash

osascript "new_window.scpt"
sleep 1
osascript "set_position.scpt"

function handle_sigint() {
  ./exit_all_processes.sh
  sleep 1
  osascript "close_window.scpt"
  echo "exit"
  sleep 1
  exit
}

trap handle_sigint SIGINT

while true; do
  sleep 1
done
