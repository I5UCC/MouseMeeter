;@Ahk2Exe-Let Version = 3.0
;@Ahk2Exe-IgnoreBegin
;@Ahk2Exe-IgnoreEnd
;@Ahk2Exe-SetMainIcon icon.ico
;@Ahk2Exe-SetVersion %U_Version%
;@Ahk2Exe-SetName Mousemeeter
;@Ahk2Exe-SetDescription Mousemeeter
;@Ahk2Exe-Bin Unicode 64*
;@Ahk2Exe-Obey U_au, = "%A_IsUnicode%" ? 2 : 1 ; .Bin file ANSI or Unicode?

#SingleInstance Force
Persistent
SetWorkingDir(A_ScriptDir)
SendMode "Input"
#Include "VMR.ahk"

;#REGION: Global Variables
global RunAsAdmin := true
global TitleMatchMode := 3
global ResetOnStartup := true
global SetAffinity := true
global SetCracklingFix := true
global OUTPUT_1 := 6
global OUTPUT_2 := 7
global OUTPUT_3 := 8
global VOLUME_CHANGE_AMOUNT := 0.5

global vm := VMR()
global isActivated := true
global HotkeyState := false
global DeactivateOnWindow := false

global default_file := "default.xml"
global profile1_file := "profile1.xml"
global profile2_file := "profile2.xml"
global current_file := default_file
;#ENDREGION

;#REGION: VM-Functions
volumeUp(s) {
    global vm

    vm.strip[s].gain += VOLUME_CHANGE_AMOUNT
}

volumeDown(s) {
    global vm

    vm.strip[s].gain -= VOLUME_CHANGE_AMOUNT
}

volumeMute(strip, v := -1) {
    global vm

    if (v != -1)
        vm.strip[strip].mute := v
    Else
        vm.strip[strip].mute--
}

restart() {
    global vm

    vm.command.restart()
}

load(file) {
    global vm, current_file

    vm.command.load(A_ScriptDir . "\" . file)
    current_file := file
}
;#ENDREGION

;#REGION: Tray-Menu
Tray := A_TrayMenu
Tray.Delete() 
Tray.Add("Reload", ReloadHandler)
Tray.Add("Refresh Config", RefreshHandler)
Tray.Add("")
Tray.Add("Open Config", OpenConfigHandler)
Tray.Add("")
Tray.Add("Exit", ExitHandler)

ProcessExists(name) {
    ErrorLevel := ProcessExist(name)
    return ErrorLevel
}

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
;#ENDREGION

;#REGION: Mouse-HOTKEYS
#HotIf isActivated
XButton1::
{ 
    global HotkeyState
    While GetKeyState("XButton1", "P") {
        HotkeyState := true
        Sleep(200)
    }
    HotkeyState := false
    return
} 

XButton2::
{ 
    global HotkeyState
    While GetKeyState("XButton2", "P") {
        HotkeyState := true
        Sleep(200)
    }
    HotkeyState := false
    return
} 

XButton1 Up::
{ 
    global HotkeyState
    HotkeyState := false
    If (A_PriorHotkey == "XButton1")
        Send("{XButton1}")
    return
} 

XButton2 Up::
{ 
    global HotkeyState
    HotkeyState := false
    If (A_PriorHotkey == "XButton2")
        Send("{XButton2}")
    return
} 

#HotIf isActivated && HotkeyState
LButton::
{ 
    If (GetKeyState("XButton1", "P") && GetKeyState("XButton2", "P"))
        Send("{LButton}") ; TODO: Can add custom function here
    Else If (GetKeyState("XButton1", "P"))
        Send("{LButton}") ; TODO: Can add custom function here
    Else If (GetKeyState("XButton2", "P"))
        Send("{Media_Prev}")
    return
} 

RButton::
{ 
    If (GetKeyState("XButton1", "P") && GetKeyState("XButton2", "P"))
        Send("{RButton}") ; TODO: Can add custom function here
    Else If (GetKeyState("XButton1", "P"))
        Send("{RButton}") ; TODO: Can add custom function here
    Else If (GetKeyState("XButton2", "P"))
        Send("{Media_Next}")
    return
} 

MButton::
{  
    If (GetKeyState("XButton1", "P") && GetKeyState("XButton2", "P"))
        Send("{MButton}") ; TODO: Can add custom function here
    Else If (GetKeyState("XButton1", "P"))
        Send("{MButton}") ; TODO: Can add custom function here
    Else If (GetKeyState("XButton2", "P"))
        Send("{Media_Play_Pause}")
    return
} 

WheelUp::
{ 
    global OUTPUT_1, OUTPUT_2, OUTPUT_3

    If (GetKeyState("XButton1", "P") && GetKeyState("XButton2", "P")) ;VAIO3
        volumeUp(OUTPUT_3)
    Else If (GetKeyState("XButton1", "P")) ;VAIO
        volumeUp(OUTPUT_1)
    Else If (GetKeyState("XButton2", "P")) ;AUX
        volumeUp(OUTPUT_2)
    return
} 

WheelDown::
{
    global OUTPUT_1, OUTPUT_2, OUTPUT_3

    If (GetKeyState("XButton1", "P") && GetKeyState("XButton2", "P")) ;VAIO3
        volumeDown(OUTPUT_3)
    Else If (GetKeyState("XButton1", "P")) ;VAIO
        volumeDown(OUTPUT_1)
    Else If (GetKeyState("XButton2", "P")) ;AUX
        volumeDown(OUTPUT_2)
    return
} 
#HotIf
;#ENDREGION

;#REGION: Keyboard-HOTKEYS
F24::
{
    global default_file, profile1_file, profile2_file, OUTPUT_1, OUTPUT_2, OUTPUT_3

    If (GetKeyState("XButton1", "P") && GetKeyState("XButton2", "P")) ;VAIO3
        volumeMute(OUTPUT_3)
    Else If (GetKeyState("XButton1", "P")) ;VAIO
        volumeMute(OUTPUT_1)
    Else If (GetKeyState("XButton2", "P")) ;AUX
        volumeMute(OUTPUT_2)
    Else {
        ErrorLevel := !KeyWait(A_ThisHotkey)
        ErrorLevel := !KeyWait(A_ThisHotkey, "d t0.250")
        If (Errorlevel) {
            if (current_file == profile1_file) {
                load(default_file)
            }
            Else {
                load(profile1_file)
            }
        }
        Else {
            if (current_file == profile2_file) {
                load(default_file)
            }
            Else {
                load(profile2_file)
            }
        }
    }
    return
}

Volume_Up::
{
    global OUTPUT_1
    volumeUp(OUTPUT_1)
}

Volume_Down::
{
    global OUTPUT_1
    volumeDown(OUTPUT_1)
}

+Volume_Up::
{
    global OUTPUT_2
    volumeUp(OUTPUT_2)
}

+Volume_Down::
{
    global OUTPUT_2
    volumeDown(OUTPUT_2)
}

^+Volume_Up::
{
    global OUTPUT_3
    volumeUp(OUTPUT_3)
}

^+Volume_Down::
{
    global OUTPUT_3
    volumeDown(OUTPUT_3)
}

^!F4::
{ 
    active_id := WinGetPID("A")
    Run("taskkill /PID " active_id " /F", , "Hide")
    return
} 

^+R::
{ 
    global default_file
    ErrorLevel := !KeyWait("R")
    ErrorLevel := !KeyWait("R", "d t0.250")
    If (Errorlevel) {
        restart()
    }
    Else {
        load(default_file)
    }
    return
} 
;#ENDREGION

Start() {
    global vm

    vm.Login()

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
        ProcessSetPriority("High")

    If (SetCracklingFix)
        Run("powershell `"$Process = Get-Process audiodg; $Process.ProcessorAffinity=1; $Process.PriorityClass=`"`"High`"`"`"", , "Hide")

    If (ResetOnStartup)
        load(default_file)

    MainLoop()
}

ReadConfigIni() {
    global RunAsAdmin, TitleMatchMode, ResetOnStartup, SetAffinity, SetCracklingFix, OUTPUT_1, OUTPUT_2, OUTPUT_3, VOLUME_CHANGE_AMOUNT, default_file, profile1_file, profile2_file, DeactivateOnWindow
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

MainLoop() {
    global isActivated, DeactivateOnWindow

    If (DeactivateOnWindow) {
        Loop
        {
            Loop Parse, DeactivateOnWindow, "`n", "`r"
            {
                if WinActive(A_LoopField) {
                    isActivated := false
                    WinWaitNotActive(A_LoopField)
                    isActivated := true
                }
                Sleep(300)
            }
            Sleep(1000)
        }
    }
}

While (!ProcessExists("voicemeeter8.exe") && !ProcessExists("voicemeeter8x64.exe"))
    Sleep(1000)
Sleep(5000)

Start()
