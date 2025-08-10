using System;
using System.Collections.Generic;
using System.Globalization;
using System.Linq;
using System.Text;
using System.Threading.Tasks;

namespace Mousemeeter;

public class MousemeeterConfig
{
    public bool RunAsAdmin { get; set; } = true;
    public int TitleMatchMode { get; set; } = 3;
    public bool ResetOnStartup { get; set; } = true;
    public bool SetAffinity { get; set; } = true;
    public bool SetCracklingFix { get; set; } = true;

    public int Output1 { get; set; } = 5;
    public int Output2 { get; set; } = 6;
    public int Output3 { get; set; } = 7;
    public float VolumeChangeAmount { get; set; } = 0.5f;

    public string DefaultFile { get; set; } = "default.xml";
    public string Profile1File { get; set; } = "profile1.xml";
    public string Profile2File { get; set; } = "profile2.xml";
    public string CurrentFile { get; set; } = "default.xml";

    public List<string> DeactivateOnWindow { get; set; } = new List<string>();

    public void LoadConfig(string configPath = "config.ini")
    {
        if (!File.Exists(configPath))
        {
            CreateDefaultConfig(configPath);
            return;
        }

        try
        {
            var lines = File.ReadAllLines(configPath);
            string currentSection = "";

            foreach (var line in lines)
            {
                var trimmedLine = line.Trim();
                if (trimmedLine.StartsWith("[") && trimmedLine.EndsWith("]"))
                {
                    currentSection = trimmedLine.Substring(1, trimmedLine.Length - 2);
                }
                else if (trimmedLine.Contains("="))
                {
                    var parts = trimmedLine.Split('=');
                    if (parts.Length == 2)
                    {
                        var key = parts[0].Trim();
                        var value = parts[1].Trim();

                        switch (currentSection)
                        {
                            case "Settings":
                                SetSettingsValue(key, value);
                                break;
                            case "VoicemeeterSettings":
                                SetVoicemeeterValue(key, value);
                                break;
                            case "DeactivateOnWindow":
                                DeactivateOnWindow.Add(value);
                                break;
                        }
                    }
                }
            }
        }
        catch (Exception ex)
        {
            Console.WriteLine($"Error loading config: {ex.Message}");
            CreateDefaultConfig(configPath);
        }
    }

    private void SetSettingsValue(string key, string value)
    {
        switch (key)
        {
            case "RunAsAdmin":
                RunAsAdmin = bool.Parse(value);
                break;
            case "TitleMatchMode":
                TitleMatchMode = int.Parse(value);
                break;
            case "ResetOnStartup":
                ResetOnStartup = bool.Parse(value);
                break;
            case "SetAffinity":
                SetAffinity = bool.Parse(value);
                break;
            case "SetCracklingFix":
                SetCracklingFix = bool.Parse(value);
                break;
        }
    }

    private void SetVoicemeeterValue(string key, string value)
    {
        switch (key)
        {
            case "OUTPUT_1":
                Output1 = int.Parse(value);
                break;
            case "OUTPUT_2":
                Output2 = int.Parse(value);
                break;
            case "OUTPUT_3":
                Output3 = int.Parse(value);
                break;
            case "VOLUME_CHANGE_AMOUNT":
                VolumeChangeAmount = float.Parse(value, CultureInfo.InvariantCulture);
                break;
            case "default_file":
                DefaultFile = value;
                break;
            case "profile1_file":
                Profile1File = value;
                break;
            case "profile2_file":
                Profile2File = value;
                break;
        }
    }

    private void CreateDefaultConfig(string configPath)
    {
        var defaultConfig = @"[Settings]
RunAsAdmin=True
TitleMatchMode=3
ResetOnStartup=True
SetAffinity=True
SetCracklingFix=True

[VoicemeeterSettings]
OUTPUT_1=5
OUTPUT_2=6
OUTPUT_3=7
VOLUME_CHANGE_AMOUNT=0.5
default_file=default.xml
profile1_file=profile1.xml
profile2_file=profile2.xml

[DeactivateOnWindow]
";
        File.WriteAllText(configPath, defaultConfig);
    }
}
