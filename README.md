
# <img src="https://user-images.githubusercontent.com/43730681/171195005-6a738083-34b6-418b-a73e-24d3cc11500c.png" width="32" height="32">  MouseMeeter
[![Github All Releases](https://img.shields.io/github/downloads/i5ucc/Mousemeeter/total.svg)](https://github.com/I5UCC/Mousemeeter/releases/latest) 
[![GitHub release (latest by date)](https://img.shields.io/github/downloads/i5ucc/Mousemeeter/latest/total?label=Latest%20version%20downloads)](https://github.com/I5UCC/Mousemeeter/releases/latest)
<a href='https://ko-fi.com/i5ucc' target='_blank'><img height='35' style='border:0px;height:25px;' src='https://az743702.vo.msecnd.net/cdn/kofi3.png?v=0' border='0' alt='Buy Me a Coffee at ko-fi.com' />

A relatively simple program binding mouse keys to do Voicemeeter Potato commands.
This program primarily uses the mouse side buttons to control Voicemeeter (And also some other things). 
These sidebuttons are normally used to go "Forward" or "Backward" on a page, these actions still work (Seen on the example picture).

Loads a "default.xml" file on start and reset. Create one with the save feature of Voicemeeter and place it in the same folder.

### *If you run the program as Admin it also automatically sets the affinity of "audiodg.exe" to only one core to fix the crackling noises that can occur in VoiceMeeter. Otherwise, elevation is not needed.*

## Macros:

| Macro | Action |
| --- | --- |
| Backward+WheelUp   | VAIO Volume increase by 0.5 |
| Backward+WheelDown | VAIO Volume decrease by 0.5 |
| Forward+WheelUp | AUX Volume increase by 0.5 |
| Forward+WheelDown | AUX Volume decrease by 0.5 |
| Backward+Forward+WheelUp | VAIO3 Volume increase by 0.5 |
| Backward+Forward+WheelDown | VAIO3 Volume decrease by 0.5 |
| Forward+LButton | Media_Prev |
| Forward+RButton | Media_Next |
| Forward+MButton | Media_Play_Pause |
| F24 (bound to mouse) | Load `profile1.xml` |
| double F24 | Load `profile2.xml` |
| Forward+F24 | AUX Mute |
| Backward+F24 | VAIO Mute |
| Backward+Forward+F24 | VAIO3 Mute |
| Ctrl+Shift+R | Restart Audio Engine |
| double Ctrl+Shift+R | Load `default.xml` |
| Ctrl+Alt+F4 | Force close Active window (Something like [SuperF4](https://stefansundin.github.io/superf4/)) |

Releasing Backward/Forward without triggering any of the above macros will just send Backward/Forward again

## Setup

Download `Mousemeeter-windows.zip` from the [latest release](https://github.com/I5UCC/Mousemeeter/releases/latest) and extract it into a folder together with your profile XML files. A default `config.ini` is created there on first start, and `log.txt` is written next to the exe.

## Configuration

Right-click the tray icon → **Settings…** (or double-click the tray icon) to change the settings in a window. Saving applies them right away, except the startup options, which apply after **Reload**. They are stored in `config.ini`:

| Key | Description |
| --- | --- |
| `RunAsAdmin` | Restart with administrator rights (UAC prompt). If declined, Mousemeeter keeps running without them. |
| `ResetOnStartup` | Load `default_file` on start instead of reading the current values from Voicemeeter. |
| `SetAffinity` | Raise Voicemeeter's process priority. |
| `SetCracklingFix` | Pin `audiodg.exe` to one core with high priority (needs admin). |
| `VoicemeeterPath` | Voicemeeter install folder, or `default`. |
| `OUTPUT_1` / `OUTPUT_2` / `OUTPUT_3` | Strip index controlled with Backward / Forward / both. |
| `VOLUME_CHANGE_AMOUNT` | dB per wheel step. |
| `default_file` / `profile1_file` / `profile2_file` | Profile XML files. |
| `[DeactivateOnWindow]` | Window titles (one per line, `key=title` or just `title`) where the mouse macros are disabled. |
| `TitleMatchMode` | How those titles match: `1` = starts with, `2` = contains, `3` = exact. |

## Building

Mousemeeter is written in Rust and only builds on Windows.

```
cargo build --release    # target/release/Mousemeeter.exe
cargo test
cargo xtask package      # dist/Mousemeeter-windows.zip
```

Pushing a tag like `v3.0.0` builds the zip on GitHub Actions and attaches it to a release.

### Example
![Example picture](https://i.imgur.com/xqwWpx9.png)
