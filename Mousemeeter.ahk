#SingleInstance Force
#Persistent
#NoEnv
SetWorkingDir %A_ScriptDir%
SendMode Input
#Include VMR.ahk/VMR.ahk
#Include Jxon.ahk

global OUTPUT_1
global OUTPUT_2
global OUTPUT_3
global VOLUME_CHANGE_AMOUNT
global DEFAULT_VOLUME

global voicemeeter
global isActivated
global HotkeyState
global ProgramArray

Process, Priority,, High
; Set audiodg.exe priority to High and set Affinity to one core to fix crackling noises
Run, powershell "$Process = Get-Process audiodg; $Process.ProcessorAffinity=1; $Process.PriorityClass=""High""",, Hide


Init()

notImplemented() { ;Placeholder

}

Init() {
    isActivated := True
    HotkeyState := False
    voicemeeter := new Voicemeeter()
    voicemeeter.reset()
    
    Fileread, file, config.json
    arr := Jxon_Load(file)
    ProgramArray := []
    for each, obj in arr {
        switch each {
            case "Settings":
                for index, d in obj {
                    OUTPUT_1 := d.OUTPUT_1
                    OUTPUT_2 := d.OUTPUT_2
                    OUTPUT_3 := d.OUTPUT_3
                    VOLUME_CHANGE_AMOUNT := d.VOLUME_CHANGE_AMOUNT
                    DEFAULT_VOLUME := d.DEFAULT_VOLUME
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
        Sleep, 200
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

#If isActivated
XButton2::
    HotkeyState := True
Return

#If isActivated
XButton1 Up::
    HotkeyState := False
    If (A_PriorHotkey == "XButton1")
        Send, {XButton1}
Return

#If isActivated
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

#If isActivated && HotkeyState
RButton::
    If (GetKeyState("XButton1","P") && GetKeyState("XButton2","P"))
        notImplemented()
    Else If (GetKeyState("XButton1","P"))
        notImplemented()
    Else If (GetKeyState("XButton2","P"))
        Send, {Media_Next}
Return

#If isActivated && HotkeyState
MButton::
    If (GetKeyState("XButton1","P") && GetKeyState("XButton2","P"))
        notImplemented()
    Else If (GetKeyState("XButton1","P"))
        notImplemented()
    Else If (GetKeyState("XButton2","P"))
        Send, {Media_Play_Pause}
Return

#If isActivated && HotkeyState
WheelUp::
    If (GetKeyState("XButton1","P") && GetKeyState("XButton2","P")) ;VAIO3
        voicemeeter.volumeUp(OUTPUT_3)
    Else If (GetKeyState("XButton1","P")) ;VAIO
        voicemeeter.volumeUp(OUTPUT_1)
    Else If (GetKeyState("XButton2","P")) ;AUX
        voicemeeter.volumeUp(OUTPUT_2)
Return

#If isActivated && HotkeyState
WheelDown::
    If (GetKeyState("XButton1","P") && GetKeyState("XButton2","P")) ;VAIO3
        voicemeeter.volumeDown(OUTPUT_3)
    Else If (GetKeyState("XButton1","P")) ;VAIO
        voicemeeter.volumeDown(OUTPUT_1)
    Else If (GetKeyState("XButton2","P")) ;AUX
        voicemeeter.volumeDown(OUTPUT_2)
Return

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

        this.vm.strip[OUTPUT_1].gain := DEFAULT_VOLUME
        this.vm.strip[OUTPUT_2].gain := DEFAULT_VOLUME
        this.vm.strip[OUTPUT_3].gain := DEFAULT_VOLUME

        for i, strip in this.vm.strip {
            strip.mute := 0
        }

        this.volumeMute(2, 1)

        this.restart()
    }
}
