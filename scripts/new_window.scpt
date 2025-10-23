set command_1 to "cd Documents/Documents-Macmini/HKUST_Paper/source-code/Manifoldchain_Impl/scripts;./generate_node_start.sh 0 0;"
set command_2 to "cd Documents/Documents-Macmini/HKUST_Paper/source-code/Manifoldchain_Impl/scripts;./generate_node_start.sh 1 0 0;"
set command_3 to "cd Documents/Documents-Macmini/HKUST_Paper/source-code/Manifoldchain_Impl/scripts;./generate_node_start.sh 2 1 1;"
set command_4 to "cd Documents/Documents-Macmini/HKUST_Paper/source-code/Manifoldchain_Impl/scripts;./generate_node_start.sh 3 2 0 1;"
set commands to {command_1, command_2, command_3, command_4}
tell application "Terminal"
    activate
    repeat with cmd in commands
        set new_tab to do script cmd
        delay 4
    end repeat
end tell
