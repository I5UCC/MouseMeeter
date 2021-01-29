#SingleInstance ignore
#Persistent
#NoTrayIcon
#NoEnv
SendMode Input

InitScript()

InitScript() {
    setHotkeyState(False)
    Voicemeeter_CMD()
}

CMD(cmd, Directory, bhide:=False) {
    If Directory not contains :
        Directory = %A_ScriptDir%%Directory%
    Run, %ComSpec% /c %cmd%, %Directory%, (bhide ? Hide : Show)
}

Voicemeeter_CMD(cmd := "reset") {
    switch cmd {
        case "reset": Send, {F13}
        case "VR": Send, {F16}
    }
}

setHotkeyState(sw) {
    If (sw) {
        Hotkey, WheelUp, On
        Hotkey, WheelDown, On
        Hotkey, LButton, On
        Hotkey, RButton, On
        Hotkey, MButton, On
    }
    Else {
        Hotkey, WheelUp, Off
        Hotkey, WheelDown, Off
        Hotkey, LButton, Off
        Hotkey, RButton, Off
        Hotkey, MButton, Off
    }
}

notImpl() {
    ;MsgBox, 64, NOT IMPLEMENTED, NOT IMPLEMENTED, 5
}

;KB-HOTKEYS
^!F4::
    WinGet, active_id, PID, A
    run, taskkill /PID %active_id% /F,,Hide
return

;Mouse-HOTKEYS
XButton1::
    state := False
    While GetKeyState("XButton1", "P")
        setHotkeyState(True)
    setHotkeyState(False)
Return

XButton2::
    state := False
    While GetKeyState("XButton2", "P")
        setHotkeyState(True)
    setHotkeyState(False)
Return

XButton1 Up::
    If (!state)
        Send, {XButton1}
Return

XButton2 Up::
    If (!state)
        Send, {XButton2}
Return

WheelUp::
    If (GetKeyState("XButton1","P") && GetKeyState("XButton2","P"))
        Send, ^{Volume_Up}
    Else If (GetKeyState("XButton1","P"))
        Send, {Volume_Up}
    Else If (GetKeyState("XButton2","P"))
        Send, +{Volume_Up}
    state := True
Return

WheelDown::
    If (GetKeyState("XButton1","P") && GetKeyState("XButton2","P"))
        Send, ^{Volume_Down}
    Else If (GetKeyState("XButton1","P"))
        Send, {Volume_Down}
    Else If (GetKeyState("XButton2","P"))
        Send, +{Volume_Down}
    state := True
Return

LButton::
    If (GetKeyState("XButton1","P") && GetKeyState("XButton2","P"))
        notImpl() ;#TODO: Implement an action
    Else If (GetKeyState("XButton1","P"))
        notImpl() ;#TODO: Implement an action
    Else If (GetKeyState("XButton2","P"))
        Send, {Media_Prev}
    state := True
Return

RButton::
    If (GetKeyState("XButton1","P") && GetKeyState("XButton2","P"))
        notImpl() ;#TODO: Implement an action
    Else If (GetKeyState("XButton1","P"))
        notImpl() ;#TODO: Implement an action
    Else If (GetKeyState("XButton2","P"))
        Send, {Media_Next}
    state := True
Return

MButton::
    If (GetKeyState("XButton1","P") && GetKeyState("XButton2","P"))
        notImpl() ;#TODO: Implement an action
    Else If (GetKeyState("XButton1","P"))
        notImpl() ;#TODO: Implement an action
    Else If (GetKeyState("XButton2","P")) 
        Send, {Media_Play_Pause}
    state := True
Return

*CtrlBreak::
     If (GetKeyState("XButton1","P") && GetKeyState("XButton2","P"))
        Send, ^{Volume_Mute}
    Else If (GetKeyState("XButton1","P"))
        Send, {Volume_Mute}
    Else If (GetKeyState("XButton2","P"))
        Send, +{Volume_Mute}
    Else
        Send, {F15}
    state := True
Return