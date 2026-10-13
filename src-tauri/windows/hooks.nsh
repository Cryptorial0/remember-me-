; Adds a second Start-menu entry named "RM" so typing "rm" finds the app.
!macro NSIS_HOOK_POSTINSTALL
  CreateShortcut "$SMPROGRAMS\RM.lnk" "$INSTDIR\${MAINBINARYNAME}.exe"
!macroend

!macro NSIS_HOOK_POSTUNINSTALL
  Delete "$SMPROGRAMS\RM.lnk"
!macroend
