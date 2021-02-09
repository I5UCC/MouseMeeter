#SingleInstance ignore
;#NoTrayIcon
#NoEnv
SendMode Input

#include auto_oculus_touch.ahk
InitOculus()


Loop {
    Sleep 55
    Poll()
    down                := GetButtonsDown()
    pressed             := GetButtonsPressed()
    released            := GetButtonsReleased()
    rightY              := GetThumbStick(RightHand, YAxis)
    leftY               := GetThumbStick(LeftHand, YAxis)
    leftHandTrigger     := GetAxis(AxisHandTriggerLeft)
    rightHandTrigger    := GetAxis(AxisHandTriggerRight)

    ;MsgBox % rightHandTrigger
    if (rightY >= 0.7) && (rightHandTrigger >= 0.7) && ovrRThumb && down
        Send, {Volume_Up} ;Voicemeeter("MainVolUp")
    else if (rightY <= -0.7) && (rightHandTrigger >= 0.7) && ovrRThumb && down
        Send, {Volume_Down} ;Voicemeeter("MainVolDown")
    else if (leftY >= 0.7) && (leftHandTrigger >= 0.7) && ovrLThumb && down
        Send, {Volume_Up} ;Voicemeeter("MediaVolUp")
    else if (leftY <= -0.7) && (leftHandTrigger >= 0.7) && ovrLThumb && down
        Send, {Volume_Down} ;Voicemeeter("MediaVolDown")

    Process, Exist, notepad.exe
} Until !ErrorLevel