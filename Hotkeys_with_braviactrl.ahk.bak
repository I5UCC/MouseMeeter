#SingleInstance ignore
#Persistent
#NoTrayIcon
#NoEnv
SendMode Input

setHotkeyState(False)
MonitorMode()
toogleMonitorTimeout(False)
syncTime()

CMD(cmd, Directory, bhide:=False) {
    If Directory not contains :
        Directory = %A_ScriptDir%%Directory%
    Run, %ComSpec% /c %cmd%, %Directory%, (bhide ? Hide : Show)
}

syncTime() {
    CMD("w32tm.exe", "C:\Windows\System32", True)
}

Bravia_CMD(cmd, ip:="192.168.178.12") {
    command = "py bravia_console.py -i %ip% -c %cmd%"
    CMD(command, "\BraviaCtrl", True)
    Sleep 2000
}

MonitorSwitcher_CMD(mode:="PC") {
    cmd = "MonitorSwitcher.exe -load:%mode%.xml"
    CMD(cmd, "\MonitorProfileSwitcher", True)
}

Voicemeeter_CMD(cmd := "reset") {
    switch cmd {
        case "reset": Send, ^{Home}
        case "TV": Send, ^{PrintScreen}
    }
}

MonitorMode(mode := "PC") {
    If (mode == "PC") {
        SysGet, mcnt, MonitorCount
        MonitorSwitcher_CMD()
        If (mcnt == 1) {
            Bravia_CMD("home")
            Bravia_CMD("poweroff")
            Sleep 3000
        }
        Voicemeeter_CMD()
    }
    Else If (mode == "TV") {
        MonitorSwitcher_CMD("TV")
        Bravia_CMD("wakeup")
        Bravia_CMD("wakeup")
        Bravia_CMD("hdmi2")
        CMD("easyrp.exe", "\EasyRP", True)
        MouseMove, 0, 1080
        Sleep 2000
        Voicemeeter_CMD("TV")
        RunWait, "steam://open/bigpicture"
        Process, Close, easyrp.exe
        MonitorMode()
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

toogleMonitorTimeout(Mshow := True) {
    global tout
    tmp := (tout ? 1 : 0)

    c = powercfg -change -monitor-timeout-ac %tmp%
    CMD(c, "C:\Windows\System32")
    If (Mshow)
        MsgBox, 64, Timeout set, Timeout set to %tmp%, 1
    If (tout)
        turnOffMonitors()
    tout := !tout
}

turnOffMonitors() {
    Sleep 1000
    ;SendMessage, 0x112, 0xF170, 2,, Program Manager
    CMD("Turn_off_Screen.bat", "./")
}

notImpl() {
    MsgBox, 64, NOT IMPLEMENTED, NOT IMPLEMENTED, 5
}

;KB-HOTKEYS
CapsLock::Ctrl

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

XButton1 Up::
    If (!state)
        Send, {XButton1}
Return

XButton2::
    state := False
    While GetKeyState("XButton2", "P")
        setHotkeyState(True)
    setHotkeyState(False)
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
        notImpl()
    Else If (GetKeyState("XButton1","P"))
        notImpl()
    Else If (GetKeyState("XButton2","P"))
        Send, {Media_Prev}
    state := True
Return

RButton::
    If (GetKeyState("XButton1","P") && GetKeyState("XButton2","P"))
        notImpl()
    Else If (GetKeyState("XButton1","P"))
        notImpl()
    Else If (GetKeyState("XButton2","P"))
        Send, {Media_Next}
    state := True
Return

MButton::
    If (GetKeyState("XButton1","P") && GetKeyState("XButton2","P"))
        turnOffMonitors()
    Else If (GetKeyState("XButton1","P"))
        notImpl()
    Else If (GetKeyState("XButton2","P")) 
        Send, {Media_Play_Pause}
    state := True
Return

;Controller-HOTKEYS
~$vk07::
    SysGet, mc, MonitorCount
    If (mc > 1) {
        Loop {
            Sleep, 300
        } Until (GetKeyState("vk07") == "0")
        Sleep, 1000
        If (GetKeyState("JoyInfo")) {
            MsgBox, 64, Switching Monitor Mode, Switching to TV-Mode, 2
            MonitorMode("TV")
        }
    }
Return