#SingleInstance Force
Persistent
SetWorkingDir(A_ScriptDir)
SendMode("Input")
#Include "VMR.ahk/VMR.ahk"

ProcessExists(name) {
    ErrorLevel := ProcessExist(name)
    return ErrorLevel
}

While (!ProcessExists("voicemeeter8.exe") && !ProcessExists("voicemeeter8x64.exe"))
    Sleep(1000)
Sleep(5000)

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

Tray:= A_TrayMenu
Tray.Delete()
Tray.Delete() ; V1toV2: not 100% replacement of NoStandard, Only if NoStandard is used at the beginning
Tray.UseErrorLevel("On")
Tray.Add("Reload", ReloadHandler)
Tray.Add("Refresh Config", RefreshHandler)
Tray.Add("")
Tray.Add("Open Config", OpenConfigHandler)
Tray.Add("")
Tray.Add("Exit", ExitHandler)

Start()
Return

ReloadHandler(A_ThisMenuItem, A_ThisMenuItemPos, MyMenu)
{
    Reload()
    return
}

RefreshHandler(A_ThisMenuItem, A_ThisMenuItemPos, MyMenu)
{
    If (FileExist("config.ini"))
        ReadConfigIni()
    return
}

OpenConfigHandler(A_ThisMenuItem, A_ThisMenuItemPos, MyMenu)
{
    If (FileExist("config.ini")) {
        RunWait("config.ini")
        ReadConfigIni()
    }
    return
}

ExitHandler(A_ThisMenuItem, A_ThisMenuItemPos, MyMenu)
{
    ExitApp()
    return
}

Start() {
    If (FileExist("config.ini"))
        ReadConfigIni()

    if (!A_IsAdmin && RunAsAdmin) {
        Try {
	        Run("*RunAs `"" A_ScriptFullPath "`"")
        } catch {
            MsgBox("Declined Elevation, if you want to start this up without Admin Rights, change 'RunAsAdmin' to 0 in config.json")
            ExitApp()
        }
    }

    SetTitleMatchMode(TitleMatchMode)

    If (SetAffinity)
        ErrorLevel := ProcessSetPriority("High")

    If (SetCracklingFix)
        Run("powershell `"$Process = Get-Process audiodg; $Process.ProcessorAffinity=1; $Process.PriorityClass=``"High``"`",, Hide")

    If (ResetOnStartup)
        voicemeeter.load(default_file)

    MainLoop()
}

MainLoop() {
    If (DeactivateOnWindow) {
        Loop
        {
            Loop Parse, DeactivateOnWindow, "`n", "`r"
            {
                if WinActive(A_LoopField) {
                    isActivated := False
                    ErrorLevel := !WinWaitNotActive(A_LoopField)
                    isActivated := True
                }
                Sleep(300)
            }
        Sleep(500)
        }
    }
}

ReadConfigIni() {
    SettingSectionExist := IniRead("config.ini", "Settings")
    If (SettingSectionExist) {
        RunAsAdmin := IniRead("config.ini", "Settings", "RunAsAdmin")
        TitleMatchMode := IniRead("config.ini", "Settings", "TitleMatchMode")
        ResetOnStartup := IniRead("config.ini", "Settings", "ResetOnStartup")
        SetAffinity := IniRead("config.ini", "Settings", "SetAffinity")
        SetCracklingFix := IniRead("config.ini", "Settings", "SetCracklingFix")
    }
    
    VoicemeeterSectionExist := IniRead("config.ini", "VoicemeeterSettings")
    If (VoicemeeterSectionExist) {
        OUTPUT_1 := IniRead("config.ini", "VoicemeeterSettings", "OUTPUT_1")
        OUTPUT_2 := IniRead("config.ini", "VoicemeeterSettings", "OUTPUT_2")
        OUTPUT_3 := IniRead("config.ini", "VoicemeeterSettings", "OUTPUT_3")
        VOLUME_CHANGE_AMOUNT := IniRead("config.ini", "VoicemeeterSettings", "VOLUME_CHANGE_AMOUNT")
        default_file := IniRead("config.ini", "VoicemeeterSettings", "default_file")
        profile1_file := IniRead("config.ini", "VoicemeeterSettings", "profile1_file")
        profile2_file := IniRead("config.ini", "VoicemeeterSettings", "profile2_file")
    }
    
    DeactivateOnWindow := IniRead("config.ini", "DeactivateOnWindow")
}

;KB-HOTKEYS
^!F4::
{
    active_id := WinGetPID("A")
    Run("taskkill /PID " active_id " /F", , "Hide")
    return
}

^+R:: 
{
    ErrorLevel := !KeyWait("R")
    ErrorLevel := !KeyWait("R", "d t0.250")
    If (Errorlevel) {
        voicemeeter.restart()
    }
    Else {
        voicemeeter.load(default_file)
        current_file := default_file
    }
    return
}

;Mouse-HOTKEYS
#HotIf isActivated
XButton1::
{
    While GetKeyState("XButton1", "P") {
        HotkeyState := True
        Sleep(200)
    }
    HotkeyState := False
    return
}

XButton2::
{
    While GetKeyState("XButton2", "P") {
        HotkeyState := True
        Sleep(200)
    }
    HotkeyState := False
    return
}

XButton1 Up::
{
    HotkeyState := False
    If (A_PriorHotkey == "XButton1")
        Send("{XButton1}")
    return
}

XButton2 Up::
{
    HotkeyState := False
    If (A_PriorHotkey == "XButton2")
        Send("{XButton2}")
    return
}

#HotIf isActivated && HotkeyState
LButton::
{
    If (GetKeyState("XButton1","P") && GetKeyState("XButton2","P"))
        Send("{LButton}")
    Else If (GetKeyState("XButton1","P"))
        Send("{LButton}")
    Else If (GetKeyState("XButton2","P"))
        Send("{Media_Prev}")
    return
}

RButton::
{
    If (GetKeyState("XButton1","P") && GetKeyState("XButton2","P"))
        Send("{RButton}")
    Else If (GetKeyState("XButton1","P"))
        Send("{RButton}")
    Else If (GetKeyState("XButton2","P"))
        Send("{Media_Next}")
    return
}

MButton::
{
    If (GetKeyState("XButton1","P") && GetKeyState("XButton2","P"))
        Send("{MButton}")
    Else If (GetKeyState("XButton1","P"))
        Send("{MButton}")
    Else If (GetKeyState("XButton2","P"))
        Send("{Media_Play_Pause}")
    return
}

WheelUp::
{
    If (GetKeyState("XButton1","P") && GetKeyState("XButton2","P")) ;VAIO3
        voicemeeter.volumeUp(OUTPUT_3)
    Else If (GetKeyState("XButton1","P")) ;VAIO
        voicemeeter.volumeUp(OUTPUT_1)
    Else If (GetKeyState("XButton2","P")) ;AUX
        voicemeeter.volumeUp(OUTPUT_2)
    return
}

WheelDown::
{
    If (GetKeyState("XButton1","P") && GetKeyState("XButton2","P")) ;VAIO3
        voicemeeter.volumeDown(OUTPUT_3)
    Else If (GetKeyState("XButton1","P")) ;VAIO
        voicemeeter.volumeDown(OUTPUT_1)
    Else If (GetKeyState("XButton2","P")) ;AUX
        voicemeeter.volumeDown(OUTPUT_2)
    return
}

#HotIf
F24::
{
    If (GetKeyState("XButton1","P") && GetKeyState("XButton2","P")) ;VAIO3
        voicemeeter.volumeMute(OUTPUT_3)
    Else If (GetKeyState("XButton1","P")) ;VAIO
        voicemeeter.volumeMute(OUTPUT_1)
    Else If (GetKeyState("XButton2","P")) ;AUX
        voicemeeter.volumeMute(OUTPUT_2)
    Else {
        ErrorLevel := !KeyWait(A_ThisHotkey)
        ErrorLevel := !KeyWait(A_ThisHotkey, "d t0.250")
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
    return
}

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

    volumeMute(strip, v := -1) {
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
