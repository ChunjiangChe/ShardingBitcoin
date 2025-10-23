tell application "Terminal"
  set allWindows to windows
  set fitstWindow to item 1 of allWindows
  set secondWindow to item 2 of allWindows
  set thirdWindow to item 3 of allWindows
  set fourthWindow to item 4 of allWindows
  set bounds of fitstWindow to {50, 50, 980, 520}
  set bounds of secondWindow to {980, 50, 1920, 520}
  set bounds of thirdWindow to {50, 530, 980, 1000}
  set bounds of fourthWindow to {980, 530, 1920, 1000}
end tell  
