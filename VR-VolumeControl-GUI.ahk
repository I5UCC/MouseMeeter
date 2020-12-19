#SingleInstance ignore
#Persistent
#NoTrayIcon
#NoEnv
SendMode Input

Gui, +AlwaysOnTop -MinimizeBox +ToolWindow -SysMenu
Gui, Margin, 0, 0
Gui, Font, s10
gui, add, Button, w150 h50, VolumeUP  ; Add a fairly wide edit control at the top of the window.
gui, add, Button, w150 h50, VolumeDOWN  ; Add a fairly wide edit control at the top of the window.
gui, show, X3690 Y0

WinWaitClose, Oculus
Exitapp

ButtonVolumeDOWN:
    Send, {F17}
Return

ButtonVolumeUP:
    Send, {F18}
Return

GuiClose:
ExitApp