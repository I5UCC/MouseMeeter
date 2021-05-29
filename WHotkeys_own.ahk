#SingleInstance ignore
#Persistent
;#NoTrayIcon
#NoEnv
SetWorkingDir %A_ScriptDir%
SendMode Input
#Include VMR.ahk/VMR.ahk

global voicemeeter
global state

setHotkeyState(False)
voicemeeter := new Voicemeeter()
voicemeeter.cmd("RESET")

CMD("w32tm.exe /resync", "C:\Windows\System32", True)
CMD("powercfg.exe /SETACTIVE 381b4222-f694-41f0-9685-ff5bb260df2e", "C:\Windows\System32", True)

Loop {
    Process, Wait, OculusClient.exe

    If (MessageBox("Switching to VR-Mode...", 5))
        setMode("VR")
    Else
        Process, WaitClose, OculusClient.exe
}


;Methods
notImpl() {
    ;MsgBox, 64, NOT IMPLEMENTED, NOT IMPLEMENTED, 5
}

MessageBox(Message, Timeout) {
    MsgBox, 4097,, %Message%, %Timeout%
    IfMsgBox, Cancel
        Return False
    Return True
}

CMD(cmd, dir, bhide := False, wait := False) {
    If dir not contains :
        dir = %A_ScriptDir%%dir%
    
    If (!wait && !bhide)
        Run, %ComSpec% /c %cmd%, %dir%, Show
    Else If (wait && !bhide)
        RunWait, %ComSpec% /c %cmd%, %dir%, Show
    Else If (!wait && bhide)
        Run, %ComSpec% /c %cmd%, %dir%, Hide
    Else If (wait && bhide)
        RunWait, %ComSpec% /c %cmd%, %dir%, Hide
    
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

setLighthouseState(state, mac) {
    switch := state ? " on " : " off "
    command := "python ./lighthouse-v2-manager.py"
    command = %command%%switch%%mac%
    i := 0
    Loop {
        CMD(command, "\lighthouse-v2-manager", True, True)
        If (Errorlevel != 15)
            i++
    } Until (i > 2)
}

setMonitorProfile(profile) {
    cmd = "MonitorSwitcher.exe -load:%profile%.xml"
    CMD(cmd, "\MonitorProfileSwitcher", True, True)
    Sleep 5000
}

setMode(mode) {
    If (mode == "RESET") {
        setMonitorProfile("RESET")
        voicemeeter.cmd("RESET")
    }
    Else If (mode == "VR") {
        Run, "steam://rungameid/250820"
        setMonitorProfile("VR")
        voicemeeter.cmd("VR")
        setLighthouseState(True, "C2:38:4C:C7:85:A4 EB:61:A4:E7:CF:FD")
        RunWait, %A_WorkingDir%/VOTVolumeControl/VOTVolumeControl_SteamVR.ahk, %A_WorkingDir%/VOTVolumeControl
        setMode("RESET")
        WinClose, Oculus
        WinWaitClose, Oculus
        setLighthouseState(False, "C2:38:4C:C7:85:A4 EB:61:A4:E7:CF:FD")
    }
    Else If (mode == "TV") {
        setMonitorProfile("TV")
        voicemeeter.cmd("TV")
        MouseMove, 0, 2160
        CMD("easyrp.exe", "\EasyRP", True)
        Run, "steam://open/bigpicture"
        Sleep, 10000
        Process, Wait, steam.exe
        Process, WaitClose, steam.exe
        Process, Close, easyrp.exe
        setMode("RESET")
    }
}

;KB-HOTKEYS
^!F4::
    WinGet, active_id, PID, A
    run, taskkill /PID %active_id% /F,,Hide
return

^+R:: 
    KeyWait, R
    KeyWait, R, d t0.250 ;Wait for double click
    If (Errorlevel)
        voicemeeter.vm.command.restart()
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
    If (WinActive("DeadByDaylight")) ;FIXME: thank you DBD
        Send, {Space}
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
    If (mc > 1)
        If (MessageBox("Switching to TV-Mode...", 5))
            setMode("TV")
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

                this.vm.strip[1].Color_x := -0.23
                this.vm.strip[1].Color_y := +0.37
                this.vm.strip[2].Color_x := -0.23

                this.vm.strip[6].gain := -20
                this.vm.strip[7].gain := -20
                this.vm.strip[8].gain := -20

                this.vm.strip[1].mute := 0
                this.vm.strip[2].mute := 1
                this.vm.command.restart()

            Return

            ;Modes
            case "TV":
                this.setMainOutput("A5")

                this.vm.strip[6].gain := 0
                this.vm.strip[7].gain := 0
                this.vm.strip[8].mute := 1

                this.vm.command.restart()
            Return
            case "VR":
                this.setMainOutput("A4", True)

                this.vm.strip[6].gain := -20
                this.vm.strip[7].gain := -20
                this.vm.strip[8].gain := -25

                this.vm.strip[1].mute := 1
                this.vm.strip[2].mute := 0

                this.vm.command.restart()
            Return
            case "Speakers":
                If (this.vm.strip[6].A2) {
                    this.setMainOutput("A1")
                    this.vm.strip[1].mute := 0
                }
                Else {
                    this.setMainOutput("A2")
                    this.vm.strip[1].mute := 1
                }
            Return
            case "Bluetooth":
                If (this.vm.strip[6].A5) {
                    this.setMainOutput("A1")
                }
                Else {
                    this.setMainOutput("A3")
                    this.vm.command.restart()
                }
                this.vm.strip[1].mute := 0
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
                    case "A1": strip.A1 := 1
                    case "A2": strip.A2 := 1
                    case "A3": strip.A3 := 1
                    case "A4": strip.A4 := 1
                    case "A5": strip.A5 := 1
                }
            }
            If (unmute)
                strip.mute := 0
        }
    }
}
