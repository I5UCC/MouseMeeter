#SingleInstance ignore
#Persistent
;#NoTrayIcon
#NoEnv
SetWorkingDir %A_ScriptDir%
SendMode Input
#Include VMR.ahk/VMR.ahk

global voicemeeter
global state

Init()

Init() {
    setHotkeyState(False)
    voicemeeter := new Voicemeeter()
    voicemeeter.cmd("RESET")
}

;Methods
notImpl() {
    ;MsgBox, 64, NOT IMPLEMENTED, NOT IMPLEMENTED, 5
}

setHotkeyState(switch) {
    If (switch) {
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


;KB-HOTKEYS
^!F4::
    WinGet, active_id, PID, A
    run, taskkill /PID %active_id% /F,,Hide
return

^+R:: 
    KeyWait, %A_ThisHotkey%
    KeyWait, %A_ThisHotkey%, d t0.250 ;Wait for double click
    If (Errorlevel)
        voicemeeter.command.restart()
    Else
        voicemeeter.cmd("RESET")
Return

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
        voicemeeter.cmd("VOIPVolUp")
    Else If (GetKeyState("XButton1","P")) ;Main
        voicemeeter.cmd("MainVolUp")
    Else If (GetKeyState("XButton2","P")) ;Media
        voicemeeter.cmd("MediaVolUp")
    state := True
Return

WheelDown::
    If (GetKeyState("XButton1","P") && GetKeyState("XButton2","P")) ;VOIP
        voicemeeter.cmd("VOIPVolDown")
    Else If (GetKeyState("XButton1","P")) ;Main
        voicemeeter.cmd("MainVolDown")
    Else If (GetKeyState("XButton2","P")) ;Media
        voicemeeter.cmd("MediaVolDown")
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
    If (GetKeyState("XButton1","P") && GetKeyState("XButton2","P")) { ;VOIP
        Sleep, 1000
        SendMessage, 0x112, 0xF170, 2, , Program Manager ;Turn off monitors
    }
    Else If (GetKeyState("XButton1","P")) ;Main
        notImpl() ;#TODO: Implement an action
    Else If (GetKeyState("XButton2","P")) ;Media
        Send, {Media_Play_Pause}
    state := True
Return

F24::
    If (GetKeyState("XButton1","P") && GetKeyState("XButton2","P")) ;VOIP
        voicemeeter.cmd("VOIPMute")
    Else If (GetKeyState("XButton1","P")) ;Main
        voicemeeter.cmd("MainMute")
    Else If (GetKeyState("XButton2","P")) ;Media
        voicemeeter.cmd("MediaMute")
    Else {
        KeyWait, %A_ThisHotkey%
        KeyWait, %A_ThisHotkey%, d t0.250 ;Wait for double click
        If (Errorlevel)
            voicemeeter.cmd("Speakers")
        Else
            voicemeeter.cmd("Bluetooth")
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


;Classes
Class Voicemeeter {
    vm := ""
    
    __New() {
        this.vm := new VMR()
        this.vm.login()
    }

    cmd(macrolabel) {
        switch macrolabel {
            case "RESET":
                this.setMainOutput("A1", True)
                this.vm.strip[6].gain := -20
                this.vm.strip[7].gain := -20
                this.vm.strip[8].gain := -20
                this.vm.command.restart()
            Return

            ;Modes
            case "TV":
                this.setMainOutput("A4")
                this.vm.strip[6].gain := 0
                this.vm.strip[7].gain := 0
                this.vm.strip[8].mute := -1
                this.vm.command.restart()
            Return
            case "VR":
                this.setMainOutput("A3")
                this.vm.strip[6].gain := -20
                this.vm.strip[7].gain := -20
                this.vm.strip[8].gain := -25
            Return
            case "Speakers":
                If (this.vm.strip[6].A2) {
                    this.setMainOutput("A1")
                    this.vm.strip[1].mute := 0
                }
                Else {
                    this.setMainOutput("A2")
                    this.vm.strip[1].mute := -1
                }
            Return
            case "Bluetooth":
                If (this.vm.strip[6].A5) {
                    this.setMainOutput("A1")
                }
                Else {
                    this.setMainOutput("A5")
                    this.vm.command.restart()
                }
            Return

            ;Main
            case "MainVolUp": this.vm.strip[6].gain += 0.5
            case "MainVolDown": this.vm.strip[6].gain -= 0.5
            case "MainMute": this.vm.strip[6].mute--

            ;Media
            case "MediaVolUp": this.vm.strip[7].gain += 0.5
            case "MediaVolDown": this.vm.strip[7].gain -= 0.5
            case "MediaMute": this.vm.strip[7].mute--

            ;VOIP
            case "VOIPVolUp": this.vm.strip[8].gain += 0.5
            case "VOIPVolDown": this.vm.strip[8].gain -= 0.5
            case "VOIPMute": this.vm.strip[8].mute--
        }
    }
    
    setMainOutput(output, unmute := False) {
        for i, strip in this.vm.strip {
            If (i > 5) {
                strip.A1 := 0
                strip.A2 := 0
                strip.A3 := 0
                strip.A4 := 0
                strip.A5 := 0
                switch output {
                    case "A1": strip.A1 := -1
                    case "A2": strip.A2 := -1
                    case "A3": strip.A3 := -1
                    case "A4": strip.A4 := -1
                    case "A5": strip.A5 := -1
                }
            }
            If (unmute)
                strip.mute := 0
        }
    }
}