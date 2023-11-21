;@Ahk2Exe-Let Version = 1.4
;@Ahk2Exe-IgnoreBegin
;@Ahk2Exe-IgnoreEnd
;@Ahk2Exe-SetMainIcon icon.ico
;@Ahk2Exe-SetVersion %U_Version%
;@Ahk2Exe-SetName Mousemeeter
;@Ahk2Exe-SetDescription Mousemeeter
;@Ahk2Exe-Bin Unicode 64*
;@Ahk2Exe-Obey U_au, = "%A_IsUnicode%" ? 2 : 1 ; .Bin file ANSI or Unicode?

#SingleInstance Force
#Persistent
#NoEnv
SetBatchLines -1
SetWorkingDir %A_ScriptDir%
SendMode Input
#Include VMR.ahk/VMR.ahk

ProcessExists(name) {
    Process, Exist, %name%
    return ErrorLevel
}

While (!ProcessExists("voicemeeter8.exe") && !ProcessExists("voicemeeter8x64.exe"))
    Sleep, 1000
Sleep 5000

global RunAsAdmin := True
global TitleMatchMode := 3
global ResetOnStartup := True
global SetAffinity := True
global SetCracklingFix := True
global OUTPUT_1 := 6
global OUTPUT_2 := 7
global OUTPUT_3 := 8
global VOLUME_CHANGE_AMOUNT := 0.5

global voicemeeter := new Voicemeeter()
global isActivated := True
global HotkeyState := False
global DeactivateOnWindow := False

global default_file := "default.xml"
global profile1_file := "profile1.xml"
global profile2_file := "profile2.xml"
global current_file := default_file

Menu, Tray, DeleteAll
Menu, Tray, NoStandard
Menu, Tray, UseErrorLevel, On
Menu, Tray, Add, Reload, ReloadHandler
Menu, Tray, Add, Refresh Config, RefreshHandler
Menu, Tray, Add,
Menu, Tray, Add, Open Config, OpenConfigHandler
Menu, Tray, Add,
Menu, Tray, Add, Exit, ExitHandler

Start()
Return

ReloadHandler:
    Reload
return

RefreshHandler:
    If (FileExist("config.ini"))
        ReadConfigIni()
return

OpenConfigHandler:
    If (FileExist("config.ini")) {
        RunWait, config.ini
        ReadConfigIni()
    }
return

ExitHandler:
    ExitApp
return

Start() {
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
        voicemeeter.load(default_file)

    MainLoop()
}

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
                Sleep, 300
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
        IniRead, VOLUME_CHANGE_AMOUNT, config.ini, VoicemeeterSettings, VOLUME_CHANGE_AMOUNT
        IniRead, default_file, config.ini, VoicemeeterSettings, default_file
        IniRead, profile1_file, config.ini, VoicemeeterSettings, profile1_file
        IniRead, profile2_file, config.ini, VoicemeeterSettings, profile2_file
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
    If (Errorlevel) {
        voicemeeter.restart()
    }
    Else {
        voicemeeter.load(default_file)
        current_file := default_file
    }
Return

;Mouse-HOTKEYS
#If isActivated
XButton1::
    While GetKeyState("XButton1", "P") {
        HotkeyState := True
        Sleep 200
    }
    HotkeyState := False
Return

XButton2::
    While GetKeyState("XButton2", "P") {
        HotkeyState := True
        Sleep 200
    }
    HotkeyState := False
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
        If (Errorlevel) {
            if (current_file == profile1_file) {
                voicemeeter.load(default_file)
                current_file := default_file
            }
            Else {
                voicemeeter.load(profile1_file)
                current_file := profile1_file
            }
        }
        Else {
            if (current_file == profile2_file) {
                voicemeeter.load(default_file)
                current_file := default_file
            }
            Else {
                voicemeeter.load(profile2_file)
                current_file := profile2_file
            }
        }
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

    restart() {
        this.vm.command.restart()
    }

    load(file) {
        this.vm.command.load(A_ScriptDir . "\" . file)
    }
}
