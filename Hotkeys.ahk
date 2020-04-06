#SingleInstance ignore
#NoTrayIcon
#NoEnv
SendMode Input

;RESET
If (GetKeyState("JoyInfo")) {
    TV_Mode()
}
Else {
    MonitorMode()
    VoicemeeterCMD()
}
CMD("w32tm.exe", "C:\Windows\System32", True)

;Mouse-HOTKEYS
WheelUp::
    If (GetKeyState("XButton1", "P") && GetKeyState("XButton2", "P"))
        state := "VOIPUp"
    Else If (GetKeyState("XButton1", "P") || GetKeyState("XButton2", "P"))
        state := "WheelUp"
    Else
        Send, {WheelUp}
Return

WheelDown::
    If (GetKeyState("XButton1", "P") && GetKeyState("XButton2", "P"))
        state := "VOIPDown"
    Else If (GetKeyState("XButton1", "P") || GetKeyState("XButton2", "P"))
        state := "WheelDown"
    Else
        Send, {WheelDown}
Return

MButton:: 
    If (GetKeyState("XButton2", "P"))
        state := "MButton"
    Else
        Send, {MButton}
Return

XButton1::
    While, GetKeyState("XButton1", "P")
    {
        switch state
        {
            case "WheelUp":
                Send, {Volume_Up}
                state := "done"
            case "WheelDown":
                Send, {Volume_Down}
                state := "done"
            case "VOIPUp":
                Send, ^{Volume_Up}
                state := "done"
            case "VOIPDown":
                Send, ^{Volume_Down}
                state := "done"
        }
    }
    If (!state)
        Send, {XButton1}

    state := ""
Return

XButton2::
    While, GetKeyState("XButton2", "P")
    {
        switch state
        {
            case "WheelUp":
                Send, +{Volume_Up}
                state := "done"
            case "WheelDown":
                Send, +{Volume_Down}
                state := "done"
            case "VOIPUp":
                Send, ^{Volume_Up}
                state := "done"
            case "VOIPDown":
                Send, ^{Volume_Down}
                state := "done"
	        case "MButton":
                Send, {Media_Play_Pause}
                state := "done"
        }
    }
    If (!state)
        Send, {XButton2}

    state := ""
Return

;KB-HOTKEYS
^!F4::
    WinGet, active_id, PID, A
    run, taskkill /PID %active_id% /F,,Hide
return

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

;FUNCTIONS
CMD(cmd, Directory, bhide:=False) {
    If Directory not contains :
        Directory = %A_ScriptDir%%Directory%

    Run, %ComSpec% /c %cmd%, %Directory%, (bhide ? Hide : Show)

    Sleep 200
}

BraviaCommand(cmd, ip:="192.168.178.26") {
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
    switch cmd 
    {
        case "reset": Send, {Home}
        case "TV": Send, {PrintScreen}
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