#SingleInstance Force
#Persistent
#NoEnv
SetWorkingDir %A_ScriptDir%
SendMode Input
#Include VMR.ahk/VMR.ahk
#Include AutoHotkey-JSON/Jxon.ahk

global OUTPUT_1
global OUTPUT_2
global OUTPUT_3
global DEFAULT_VOLUME1
global DEFAULT_VOLUME2
global DEFAULT_VOLUME3
global VOLUME_CHANGE_AMOUNT

global voicemeeter
global isActivated
global HotkeyState
global ProgramArray

Init()

notImplemented() { ;Placeholder

}

Init() {
    SetAffinity := True
    SetCracklingFix := True
    TitleMatchMode := 3
    ResetOnStartup := True

    isActivated := True
    HotkeyState := False
    ProgramArray := []
    
    Fileread, file, config.json
    config := Jxon_Load(file)
    for each, obj in config {
        switch each {
            case "Settings":
                for index, d in obj {
                    TitleMatchMode := d.TitleMatchMode
                    ResetOnStartup := d.ResetOnStartup
                    SetAffinity := d.SetAffinity
                    SetCracklingFix := d.SetCracklingFix
                    OUTPUT_1 := d.OUTPUT_1
                    OUTPUT_2 := d.OUTPUT_2
                    OUTPUT_3 := d.OUTPUT_3
                    DEFAULT_VOLUME1 := d.DEFAULT_VOLUME1
                    DEFAULT_VOLUME2 := d.DEFAULT_VOLUME2
                    DEFAULT_VOLUME3 := d.DEFAULT_VOLUME3
                    VOLUME_CHANGE_AMOUNT := d.VOLUME_CHANGE_AMOUNT
                }
            case "DeactivateOnWindow":
                for index, d in obj {
                    ProgramArray.Push(d)
                }
            default:
                MsgBox % "Error in Config file"
                ExitApp
        }
    }

    voicemeeter := new Voicemeeter()

    If (SetAffinity)
        Process, Priority,, High
    If (SetCracklingFix)
        Run, powershell "$Process = Get-Process audiodg; $Process.ProcessorAffinity=1; $Process.PriorityClass=""High""",, Hide
    If (ResetOnStartup)
        voicemeeter.reset()

    SetTitleMatchMode, %TitleMatchMode%

    MainLoop()
}

MainLoop() {
    While (True) {
        for index, element in ProgramArray
        {
            if WinActive(element) {
                isActivated := False
                WinWaitNotActive % element
                isActivated := True
            }
        }
        Sleep, 500
    }
}

;KB-HOTKEYS
^!F4::
    WinGet, active_id, PID, A
    run, taskkill /PID %active_id% /F,,Hide
return

^+R:: 
    KeyWait, R
    KeyWait, R, d t0.250
    If (Errorlevel)
        voicemeeter.restart()
    Else
        voicemeeter.reset()
Return

;Mouse-HOTKEYS
#If isActivated
XButton1::
    HotkeyState := True
Return

XButton2::
    HotkeyState := True
Return

XButton1 Up::
    HotkeyState := False
    If (A_PriorHotkey == "XButton1")
        Send, {XButton1}
Return

XButton2 Up::
    HotkeyState := False
    If (A_PriorHotkey == "XButton2")
        Send, {XButton2}
Return

#If isActivated && HotkeyState
LButton::
    If (GetKeyState("XButton1","P") && GetKeyState("XButton2","P"))
        notImplemented()
    Else If (GetKeyState("XButton1","P"))
        notImplemented()
    Else If (GetKeyState("XButton2","P"))
        Send, {Media_Prev}
Return

RButton::
    If (GetKeyState("XButton1","P") && GetKeyState("XButton2","P"))
        notImplemented()
    Else If (GetKeyState("XButton1","P"))
        notImplemented()
    Else If (GetKeyState("XButton2","P"))
        Send, {Media_Next}
Return

MButton::
    If (GetKeyState("XButton1","P") && GetKeyState("XButton2","P"))
        notImplemented()
    Else If (GetKeyState("XButton1","P"))
        notImplemented()
    Else If (GetKeyState("XButton2","P"))
        Send, {Media_Play_Pause}
Return

WheelUp::
    If (GetKeyState("XButton1","P") && GetKeyState("XButton2","P")) ;VAIO3
        voicemeeter.volumeUp(OUTPUT_3)
    Else If (GetKeyState("XButton1","P")) ;VAIO
        voicemeeter.volumeUp(OUTPUT_1)
    Else If (GetKeyState("XButton2","P")) ;AUX
        voicemeeter.volumeUp(OUTPUT_2)
Return

WheelDown::
    If (GetKeyState("XButton1","P") && GetKeyState("XButton2","P")) ;VAIO3
        voicemeeter.volumeDown(OUTPUT_3)
    Else If (GetKeyState("XButton1","P")) ;VAIO
        voicemeeter.volumeDown(OUTPUT_1)
    Else If (GetKeyState("XButton2","P")) ;AUX
        voicemeeter.volumeDown(OUTPUT_2)
Return

#If
F24::
    If (GetKeyState("XButton1","P") && GetKeyState("XButton2","P")) ;VAIO3
        voicemeeter.volumeMute(OUTPUT_3)
    Else If (GetKeyState("XButton1","P")) ;VAIO
        voicemeeter.volumeMute(OUTPUT_1)
    Else If (GetKeyState("XButton2","P")) ;AUX
        voicemeeter.volumeMute(OUTPUT_2)
    Else {
        KeyWait, %A_ThisHotkey%
        KeyWait, %A_ThisHotkey%, d t0.250
        If (Errorlevel) {
            voicemeeter.setMainOutput("A2")
            voicemeeter.volumeMute(2, 1)
            voicemeeter.volumeMute(1)
        }
        Else
            voicemeeter.setMainOutput("A3")
    }
Return

;Classes
Class Voicemeeter {
    vm := ""
    
    __New() {
        this.vm := new VMR()
        this.vm.login()
    }

    volumeUp(strip) {
        this.vm.strip[strip].gain += VOLUME_CHANGE_AMOUNT
    }

    volumeDown(strip) {
        this.vm.strip[strip].gain -= VOLUME_CHANGE_AMOUNT
    }

    volumeMute(strip, v = -1) {
        if (v != -1)
            this.vm.strip[strip].mute := v
        Else
            this.vm.strip[strip].mute--
    }
    
    setMainOutput(output, unmute := True) {
        switch output {
            case "A2": 
                If (this.vm.strip[OUTPUT_1].A2)
                    output := "A1"
            case "A3":
                If (this.vm.strip[OUTPUT_1].A3)
                    output := "A1"
            case "A4":
                If (this.vm.strip[OUTPUT_1].A4)
                    output := "A1"
            case "A5":
                If (this.vm.strip[OUTPUT_1].A5)
                    output := "A1"
        }
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

    restart() {
        voicemeeter.vm.command.restart()
    }

    reset() {
        this.setMainOutput("A1")

        this.vm.strip[OUTPUT_1].gain := DEFAULT_VOLUME1
        this.vm.strip[OUTPUT_2].gain := DEFAULT_VOLUME2
        this.vm.strip[OUTPUT_3].gain := DEFAULT_VOLUME3

        for i, strip in this.vm.strip {
            strip.mute := 0
        }

        this.volumeMute(2, 1)

        this.restart()
    }
}
