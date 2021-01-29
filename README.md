# **! WARNING !**
This script is heavily adjusted to my system, so some things could not work at all or in the way you want it to. For example it triggers on Oculus startup to automatically switch to Output A3 and change Microphones, it also sets different monitor profiles to switch between. 

### **I also put in a stripped back version in the Archive that does remove almost all of it and just leaves the Voicemeeter macros.**

### **It is Open-Source, so you can change it yourself**, maybe I will make it a bit more user friendly in the future.

# WHotkeys
An Autohotkey script to bind mouse keys to do Voicemeeter commands.
This script primarily uses the mouse side buttons to control Voicemeeter (And also some other things). 
These sidebuttons are normally used to go "Forward" or "Backward" on a page, these actions still work.

Macros:

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
| F24 (bound to mouse) | switch to output A2 |
| double F24 | switch to output A5 |
| Forward+F24 | AUX Mute |
| Backward+F24 | VAIO Mute |
| Backward+Forward+F24 | VAIO3 Mute |
| Ctrl+Shift+R | Reset to Voicemeeter predefined defaults |
| Ctrl+Alt+F4 | Force close Active window (Something like [SuperF4](https://stefansundin.github.io/superf4/)) |
| Xbox Button | Changes into "TV"-Mode |

Releasing Backward/Forward without triggering any of the above macros will just send Backward/Forward again

# Modifying the script

### **! Still in Development !**

Basically all of the communication with Voicemeeter happens in the Voicemeeter()-Method so modify that. Look into the [Documentation](https://saifaqqad.github.io/VMR.ahk/) of [VMR.ahk](https://github.com/SaifAqqad/VMR.ahk) to know how.
