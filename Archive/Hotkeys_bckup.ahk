#SingleInstance ignore
#Persistent
#NoTrayIcon
#NoEnv
SetWorkingDir %A_ScriptDir%
SendMode Input

setHotkeyState(False)
setProcessPriorities()
CMD("w32tm.exe /resync", "C:\Windows\System32", True)
SysGet, mc, MonitorCount
If (GetKeyState("JoyInfo"))
    setMode("TV")
Else If (mc != 3)
    setMode("PC")
Loop {
    Process, Wait, OculusClient.exe
    Run, auto_oculus_touch/VRVolumeControl.exe, %A_ScriptDir%/auto_oculus_touch
    Voicemeeter("VR")
    Process, WaitClose, OculusClient.exe
    Process, Close, Steam.exe
    Voicemeeter("RESET")
}

;Methods
notImpl() {
    ;MsgBox, 64, NOT IMPLEMENTED, NOT IMPLEMENTED, 5
}

CMD(cmd, Directory, bhide:=False) {
    If Directory not contains :
        Directory = %A_ScriptDir%%Directory%
    Run, %ComSpec% /c %cmd%, %Directory%, (bhide ? Hide : Show)
}

Voicemeeter(macrolabel) {
    switch macrolabel {
        case "RESET": Send, ^{F13}
        
        ;Modes
        case "TV": Send, ^{F14}
        case "VR": Send, ^{F16}
        case "Speakers": Send, ^{F15}
        case "Bluetooth": Send, ^{F17}
        
        ;Main
        case "MainVolUp": Send, {Volume_Up}
        case "MainVolDown": Send, {Volume_Down}
        case "MainMute": Send, {Volume_Mute}

        ;Media
        case "MediaVolUp": Send, +{Volume_Up}
        case "MediaVolDown": Send, +{Volume_Down}
        case "MediaMute": Send, +{Volume_Mute}

        ;VOIP
        case "VOIPVolUp": Send, ^{Volume_Up}
        case "VOIPVolDown": Send, ^{Volume_Down}
        case "VOIPMute": Send, ^{Volume_Mute}
    }
}

MonitorSwitcher(mode:="PC") {
    cmd = "MonitorSwitcher.exe -load:%mode%.xml"
    CMD(cmd, "\MonitorProfileSwitcher", True)
    Sleep 3000
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

setProcessPriorities() {
    Process, Wait, voicemeeter8.exe
    Process, Wait, VoicemeeterMacroButtons.exe
    Process, Wait, Light Host.exe
    
    Process, Priority, Hotkeys.exe, H
    Process, Priority, Light Host.exe, H
    Process, Priority, voicemeeter8.exe, H
    Process, Priority, VoicemeeterMacroButtons.exe, A
}

setMode(mode) {
    If (mode == "PC") {
        MonitorSwitcher()
        Voicemeeter("RESET")
    }
    Else If (mode == "TV") {
        MsgBox, 64, Switching Monitor Mode, Switching to TV-Mode, 2
        MonitorSwitcher("TV")
        Run, "steam://open/bigpicture"
        CMD("easyrp.exe", "\EasyRP", True)
        Sleep, 5000
        Voicemeeter("TV")
        MouseMove, 0, 2160
        WinWaitClose, Steam
        Process, Close, easyrp.exe
        setMode("PC")
    }
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
    If (GetKeyState("XButton1","P") && GetKeyState("XButton2","P")) ;VOIP
        Voicemeeter("VOIPVolUp")
    Else If (GetKeyState("XButton1","P")) ;Main
        Voicemeeter("MainVolUp")
    Else If (GetKeyState("XButton2","P")) ;Media
        Voicemeeter("MediaVolUp")
    state := True
Return

WheelDown::
    If (GetKeyState("XButton1","P") && GetKeyState("XButton2","P")) ;VOIP
        Voicemeeter("VOIPVolDown")
    Else If (GetKeyState("XButton1","P")) ;Main
        Voicemeeter("MainVolDown")
    Else If (GetKeyState("XButton2","P")) ;Media
        Voicemeeter("MediaVolDown")
    state := True
Return

LButton::
    If (GetKeyState("XButton1","P") && GetKeyState("XButton2","P")) ;VOIP
        notImpl() ;#TODO: Implement an action
    Else If (GetKeyState("XButton1","P")) ;Main
        notImpl() ;#TODO: Implement an action
    Else If (GetKeyState("XButton2","P")) ;Media
        Send, {Media_Prev}
    state := True
Return

RButton::
    If (GetKeyState("XButton1","P") && GetKeyState("XButton2","P")) ;VOIP
        notImpl() ;#TODO: Implement an action
    Else If (GetKeyState("XButton1","P")) ;Main
        notImpl() ;#TODO: Implement an action
    Else If (GetKeyState("XButton2","P")) ;Media
        Send, {Media_Next}
    state := True
Return

MButton::
    If (GetKeyState("XButton1","P") && GetKeyState("XButton2","P")) ;VOIP
        notImpl() ;#TODO: Implement an action
    Else If (GetKeyState("XButton1","P")) ;Main
        notImpl() ;#TODO: Implement an action
    Else If (GetKeyState("XButton2","P")) ;Media
        Send, {Media_Play_Pause}
    state := True
Return

F24::
    If (GetKeyState("XButton1","P") && GetKeyState("XButton2","P")) ;VOIP
        Voicemeeter("VOIPMute")
    Else If (GetKeyState("XButton1","P")) ;Main
        Voicemeeter("MainMute")
    Else If (GetKeyState("XButton2","P")) ;Media
        Voicemeeter("MediaMute")
    Else {
        KeyWait, %A_ThisHotkey%
        KeyWait, %A_ThisHotkey%, d t0.250 ;Wait for double click
        If (Errorlevel)
            Voicemeeter("Speakers")
        Else
            Voicemeeter("Bluetooth")
    }
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
        If (GetKeyState("JoyInfo"))
            setMode("TV")
    }
Return