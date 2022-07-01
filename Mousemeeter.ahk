;@Ahk2Exe-Let Version = 1.4
;@Ahk2Exe-IgnoreBegin
;@Ahk2Exe-IgnoreEnd
;@Ahk2Exe-SetMainIcon icon.ico
;@Ahk2Exe-SetVersion %U_Version%
;@Ahk2Exe-SetName Mousemeeter
;@Ahk2Exe-SetDescription Mousemeeter
;@Ahk2Exe-Bin Unicode 64*
;@Ahk2Exe-Obey U_au, = "%A_IsUnicode%" ? 2 : 1 ; .Bin file ANSI or Unicode?
;@Ahk2Exe-PostExec "BinMod.exe" "%A_WorkFileName%"
;@Ahk2Exe-Cont  "%U_au%2.>AUTOHOTKEY SCRIPT<. DATA              "

#SingleInstance Force
#Persistent
#NoEnv
SetBatchLines -1
SetWorkingDir %A_ScriptDir%
SendMode Input
#Include VMR.ahk/VMR.ahk

global RunAsAdmin := True
global TitleMatchMode := 3
global ResetOnStartup := True
global SetAffinity := True
global SetCracklingFix := True
global OUTPUT_1 := 6
global OUTPUT_2 := 7
global OUTPUT_3 := 8
global DEFAULT_VOLUME1 := -25
global DEFAULT_VOLUME2 := -20
global DEFAULT_VOLUME3 := -10
global VOLUME_CHANGE_AMOUNT := 0.5

global MainOutput = A1
global SecondOutput = A2
global ThirdOutput = A3
global MuteStipsOnMain = 0
global MuteStipsOnSecond = 0
global MuteStipsOnThird = 0

global voicemeeter := new Voicemeeter()
global isActivated := True
global HotkeyState := False
global DeactivateOnWindow := False

If (FileExist("config.ini"))
    ReadConfigIni()

if (!A_IsAdmin && RunAsAdmin) {
    Try {
	    Run *RunAs "%A_ScriptFullPath%"
    } catch {
        MsgBox % "Declined Elevation, if you want to start this up without Admin Rights, change 'RunAsAdmin' to 0 in config.json"
        ExitApp
    }
}

SetTitleMatchMode, %TitleMatchMode%

If (SetAffinity)
    Process, Priority,, High

If (SetCracklingFix)
    Run, powershell "$Process = Get-Process audiodg; $Process.ProcessorAffinity=1; $Process.PriorityClass=""High""",, Hide

If (ResetOnStartup)
    voicemeeter.reset()

MainLoop()

MainLoop() {
    If (DeactivateOnWindow) {
        Loop 
        {
            Loop, Parse, DeactivateOnWindow, `n,`r 
            {
                if WinActive(A_LoopField) {
                    isActivated := False
                    WinWaitNotActive % A_LoopField
                    isActivated := True
                }
            }
        Sleep, 500
        }
    }
}

ReadConfigIni() {
    IniRead, SettingSectionExist, config.ini, Settings
    If (SettingSectionExist) {
        IniRead, RunAsAdmin, config.ini, Settings, RunAsAdmin
        IniRead, TitleMatchMode, config.ini, Settings, TitleMatchMode
        IniRead, ResetOnStartup, config.ini, Settings, ResetOnStartup
        IniRead, SetAffinity, config.ini, Settings, SetAffinity
        IniRead, SetCracklingFix, config.ini, Settings, SetCracklingFix
    }
    
    IniRead, VoicemeeterSectionExist, config.ini, VoicemeeterSettings
    If (VoicemeeterSectionExist) {
        IniRead, OUTPUT_1, config.ini, VoicemeeterSettings, OUTPUT_1
        IniRead, OUTPUT_2, config.ini, VoicemeeterSettings, OUTPUT_2
        IniRead, OUTPUT_3, config.ini, VoicemeeterSettings, OUTPUT_3

        IniRead, DEFAULT_VOLUME1, config.ini, VoicemeeterSettings, DEFAULT_VOLUME1
        IniRead, DEFAULT_VOLUME2, config.ini, VoicemeeterSettings, DEFAULT_VOLUME2
        IniRead, DEFAULT_VOLUME3, config.ini, VoicemeeterSettings, DEFAULT_VOLUME3

        IniRead, VOLUME_CHANGE_AMOUNT, config.ini, VoicemeeterSettings, VOLUME_CHANGE_AMOUNT

        IniRead, MainOutput, config.ini, VoicemeeterSettings, MainOutput
        IniRead, SecondOutput, config.ini, VoicemeeterSettings, SecondOutput
        IniRead, ThirdOutput, config.ini, VoicemeeterSettings, ThirdOutput
        IniRead, MuteStipsOnMain, config.ini, VoicemeeterSettings, MuteStipsOnMain
        IniRead, MuteStipsOnSecond, config.ini, VoicemeeterSettings, MuteStipsOnSecond
        IniRead, MuteStipsOnThird, config.ini, VoicemeeterSettings, MuteStipsOnThird
    }
    
    IniRead, DeactivateOnWindow, config.ini, DeactivateOnWindow
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
        Send, {LButton}
    Else If (GetKeyState("XButton1","P"))
        Send, {LButton}
    Else If (GetKeyState("XButton2","P"))
        Send, {Media_Prev}
Return

RButton::
    If (GetKeyState("XButton1","P") && GetKeyState("XButton2","P"))
        Send, {RButton}
    Else If (GetKeyState("XButton1","P"))
        Send, {RButton}
    Else If (GetKeyState("XButton2","P"))
        Send, {Media_Next}
Return

MButton::
    If (GetKeyState("XButton1","P") && GetKeyState("XButton2","P"))
        Send, {MButton}
    Else If (GetKeyState("XButton1","P"))
        Send, {MButton}
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
        If (Errorlevel)
            voicemeeter.setMainOutput(SecondOutput)
        Else
            voicemeeter.setMainOutput(ThirdOutput)
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
                    output := MainOutput
            case "A3":
                If (this.vm.strip[OUTPUT_1].A3)
                    output := MainOutput
            case "A4":
                If (this.vm.strip[OUTPUT_1].A4)
                    output := MainOutput
            case "A5":
                If (this.vm.strip[OUTPUT_1].A5)
                    output := MainOutput
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
            strip.mute := 0
        }

        If (output == MainOutput) {
            Loop, Parse, MuteStipsOnMain, `,
            {
                this.volumeMute(A_LoopField, 1)
            }
        }
        Else If (output == SecondOutput) {
            Loop, Parse, MuteStipsOnSecond, `,
            {
                this.volumeMute(A_LoopField, 1)
            }
        }
        Else If (output == ThirdOutput) {
            Loop, Parse, MuteStipsOnThird, `,
            {
                this.volumeMute(A_LoopField, 1)
            }
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

        this.restart()
    }
}
