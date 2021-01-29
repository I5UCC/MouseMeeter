#SingleInstance ignore
#NoTrayIcon
#NoEnv
SendMode Input

#include auto_oculus_touch.ahk
InitOculus()

Loop {
    Sleep 55
    Poll()
    down     := GetButtonsDown()
    pressed  := GetButtonsPressed()
    released := GetButtonsReleased()
    rightY        := GetThumbStick(RightHand, YAxis)

    if (rightY >= 0.7) && ovrRThumb & down
        SendInput {Volume_Up}
    else if (rightY <= -0.7) && ovrRThumb & down
        SendInput {Volume_Down}

    Process, Exist, OculusClient.exe
} Until !ErrorLevel