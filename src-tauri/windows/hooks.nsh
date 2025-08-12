!macro NSIS_HOOK_POSTINSTALL
DetailPrint "Adding firewall rule for Tranzit..."
nsExec::ExecToLog 'netsh advfirewall firewall add rule name="Tranzit" dir=in action=allow program="$INSTDIR\\tranzit.exe" protocol=TCP localport=21212 enable=yes'
nsExec::ExecToLog 'netsh advfirewall firewall add rule name="Tranzit" dir=in action=allow program="$INSTDIR\\tranzit.exe" protocol=TCP localport=21112 enable=yes'
!macroend

!macro NSIS_HOOK_POSTUNINSTALL
DetailPrint "Removing firewall rule for Tranzit..."
nsExec::ExecToLog 'netsh advfirewall firewall delete rule name="Tranzit" program="$INSTDIR\\tranzit.exe"'
!macroend
