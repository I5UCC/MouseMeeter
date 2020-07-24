﻿#SingleInstance ignore
#NoTrayIcon
#NoEnv
SendMode Input

RESET()

;FUNCTIONS
RESET() {
    Hotkey, WheelUp, Volume_Up
    Hotkey, WheelDown, Volume_Down
    Hotkey, WheelUp, Off
    Hotkey, WheelDown, Off
    If (GetKeyState("JoyInfo")) {
        TV_Mode()
    }
    Else {
        MonitorMode()
        VoicemeeterCMD()
        Run, powercfg -change -monitor-timeout-ac 0,,Hide
    }
    CMD("w32tm.exe", "C:\Windows\System32", True)
}

CMD(cmd, Directory, bhide:=False) {
    If Directory not contains :
        Directory = %A_ScriptDir%%Directory%

    Run, %ComSpec% /c %cmd%, %Directory%, (bhide ? Hide : Show)

    Sleep 200
}

BraviaCommand(cmd, ip:="192.168.178.12") {
    command = "py bravia_console.py -i %ip% -c %cmd%"
    CMD(command, "\BraviaCtrl", True)
    Sleep 2000
}

MonitorMode(mode:="PC") {
    cmd = "MonitorSwitcher.exe -load:%mode%.xml"
    CMD(cmd, "\MonitorProfileSwitcher", True)

    If (mode == "PC")
        Sleep 7000
}

VoicemeeterCMD(cmd:="reset") {
    switch cmd {
        case "reset": Send, ^{Home}
        case "TV": Send, ^{PrintScreen}
    }
}

TV_Mode() {
    MonitorMode("TV")
    BraviaCommand("hdmi3")
    CMD("easyrp.exe", "\EasyRP", True)
    MouseMove, 0, 1080
    VoicemeeterCMD("TV")
    RunWait, "steam://open/bigpicture"
    Process, Close, easyrp.exe
    MonitorMode()
    VoicemeeterCMD()
}

toggleHotkeys(sw) {
    If (sw) {
        Hotkey, WheelUp, On
        Hotkey, WheelDown, On
    }
    Else {
        Hotkey, WheelUp, Off
        Hotkey, WheelDown, Off
    }
}

toogleMonitorTimeout() {
    global tout
    If (tout) {
        Run, powercfg -change -monitor-timeout-ac 0,,Hide
        MsgBox, 64, Timeout changed, Timeout changed to Never, 2
    }
    Else {
        Run, powercfg -change -monitor-timeout-ac 1,,Hide
        MsgBox, 64, Timeout changed, Timeout changed to 1 Minute, 2
    }
    tout := !tout
}

;KB-HOTKEYS
^!F4::
    WinGet, active_id, PID, A
    run, taskkill /PID %active_id% /F,,Hide
return

^F1::
    Run, explorer.exe
return

^F2::
    Run, "C:\Program Files\Mozilla Firefox\firefox.exe", "C:\Program Files\Mozilla Firefox\"
return

^F3::
    Run, "C:\Program Files (x86)\GOG Galaxy\GalaxyClient.exe", "C:\Program Files (x86)\GOG Galaxy\"
return

^F4::
    Run, ubuntu.exe
return

;Mouse-HOTKEYS
XButton1::
    state := False
    While GetKeyState("XButton1", "P")
        toggleHotkeys(True)
    toggleHotkeys(False)
Return

XButton1 Up::
    If (!state)
        Send, {XButton1}
Return

XButton2::
    state := False
    While GetKeyState("XButton2", "P")
        toggleHotkeys(True)
    toggleHotkeys(False)
Return

XButton2 Up::
    If (!state)
        Send, {XButton2}
Return

Volume_Up:
    If (GetKeyState("XButton1","P") && GetKeyState("XButton2","P"))
        Send, ^{Volume_Up}
    Else If (GetKeyState("XButton1","P"))
        Send, {Volume_Up}
    Else If (GetKeyState("XButton2","P"))
        Send, +{Volume_Up}
    state := True
Return

Volume_Down:
    If (GetKeyState("XButton1","P") && GetKeyState("XButton2","P"))
        Send, ^{Volume_Down}
    Else If (GetKeyState("XButton1","P"))
        Send, {Volume_Down}
    Else If (GetKeyState("XButton2","P"))
        Send, +{Volume_Down}
    state := True
Return

Insert::
    KeyWait, Insert
    KeyWait, Insert, D T0.2
    If (!errorlevel)
        toogleMonitorTimeout()
    Else
        Send, !{Insert}
Return

;Controller-HOTKEYS
~$vk07::
    SysGet, mc, MonitorCount
    If (mc == 3) {
        Loop {
            Sleep, 300
        } Until (GetKeyState("vk07") == "0")
        Sleep, 1000
        If (GetKeyState("JoyInfo"))
            TV_Mode()
    }
Return