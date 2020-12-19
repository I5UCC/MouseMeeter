#SingleInstance ignore
#Persistent
#NoTrayIcon
#NoEnv
SendMode Input
SetWorkingDir %A_ScriptDir%/auto_oculus_touch

#include auto_oculus_touch/auto_oculus_touch.ahk
InitOculus()

Loop {
    Poll()
    down     := GetButtonsDown()
    pressed  := GetButtonsPressed()
    released := GetButtonsReleased()
    rightY        := GetThumbStick(RightHand, YAxis)

    if (rightY >= 0.7) && ovrRThumb & down
        SendInput {Volume_Up}
    else if (rightY <= -0.7) && ovrRThumb & down
        SendInput {Volume_Down}

    Sleep 50
    if !WinExist("Oculus")
        break
}