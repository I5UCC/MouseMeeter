//! `config.ini`: parsing, defaults, and writing it back.

use std::fmt::Write;
use std::path::{Path, PathBuf};

use crate::log;

#[derive(Debug, Clone, PartialEq)]
pub struct Config {
    pub run_as_admin: bool,
    pub title_match_mode: u8,
    pub reset_on_startup: bool,
    pub set_affinity: bool,
    pub set_crackling_fix: bool,
    /// `None` means the default install location.
    pub voicemeeter_path: Option<PathBuf>,
    pub outputs: Outputs,
    pub volume_change_amount: f32,
    pub default_file: String,
    pub profile1_file: String,
    pub profile2_file: String,
    pub deactivate_on_window: Vec<String>,
}

/// Voicemeeter strip indices controlled by each side-button combination.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Outputs {
    /// Backward (XButton1) held.
    pub output1: i32,
    /// Forward (XButton2) held.
    pub output2: i32,
    /// Both held.
    pub output3: i32,
}

impl Outputs {
    pub fn for_buttons(&self, x1: bool, x2: bool) -> Option<i32> {
        match (x1, x2) {
            (true, true) => Some(self.output3),
            (true, false) => Some(self.output1),
            (false, true) => Some(self.output2),
            (false, false) => None,
        }
    }

    pub fn all(&self) -> [i32; 3] {
        [self.output1, self.output2, self.output3]
    }
}

impl Default for Config {
    fn default() -> Self {
        Config {
            run_as_admin: true,
            title_match_mode: 3,
            reset_on_startup: true,
            set_affinity: true,
            set_crackling_fix: true,
            voicemeeter_path: None,
            outputs: Outputs {
                output1: 5,
                output2: 6,
                output3: 7,
            },
            volume_change_amount: 0.5,
            default_file: "default.xml".into(),
            profile1_file: "profile1.xml".into(),
            profile2_file: "profile2.xml".into(),
            deactivate_on_window: Vec::new(),
        }
    }
}

impl Config {
    /// Loads the config file, writing the default one if it doesn't exist yet.
    pub fn load(path: &Path) -> Config {
        match std::fs::read_to_string(path) {
            Ok(text) => Self::parse(&text),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                let config = Config::default();
                if let Err(e) = config.save(path) {
                    log!("Failed to write default config: {e}");
                }
                config
            }
            Err(e) => {
                log!("Error loading config: {e}");
                Config::default()
            }
        }
    }

    pub fn save(&self, path: &Path) -> std::io::Result<()> {
        std::fs::write(path, self.to_ini())
    }

    pub fn to_ini(&self) -> String {
        let bool = |b: bool| if b { "True" } else { "False" };
        let path = match &self.voicemeeter_path {
            Some(path) => path.display().to_string(),
            None => "default".into(),
        };
        let mut ini = format!(
            "[Settings]\n\
             RunAsAdmin={}\n\
             TitleMatchMode={}\n\
             ResetOnStartup={}\n\
             SetAffinity={}\n\
             SetCracklingFix={}\n\
             VoicemeeterPath={path}\n\
             \n\
             [VoicemeeterSettings]\n\
             OUTPUT_1={}\n\
             OUTPUT_2={}\n\
             OUTPUT_3={}\n\
             VOLUME_CHANGE_AMOUNT={}\n\
             default_file={}\n\
             profile1_file={}\n\
             profile2_file={}\n\
             \n\
             [DeactivateOnWindow]\n",
            bool(self.run_as_admin),
            self.title_match_mode,
            bool(self.reset_on_startup),
            bool(self.set_affinity),
            bool(self.set_crackling_fix),
            self.outputs.output1,
            self.outputs.output2,
            self.outputs.output3,
            self.volume_change_amount,
            self.default_file,
            self.profile1_file,
            self.profile2_file,
        );
        for (i, title) in self.deactivate_on_window.iter().enumerate() {
            let _ = writeln!(ini, "{}={title}", i + 1);
        }
        ini
    }

    pub fn parse(text: &str) -> Config {
        let mut config = Config::default();
        let mut section = "";

        for line in text.trim_start_matches('\u{feff}').lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with(';') || line.starts_with('#') {
                continue;
            }
            if let Some(name) = line.strip_prefix('[').and_then(|l| l.strip_suffix(']')) {
                section = name.trim();
                continue;
            }

            let (key, value) = match line.split_once('=') {
                Some((k, v)) => (k.trim(), v.trim()),
                None => ("", line),
            };

            let result = match section {
                "Settings" => config.set_setting(key, value),
                "VoicemeeterSettings" => config.set_voicemeeter_setting(key, value),
                "DeactivateOnWindow" => {
                    if !value.is_empty() {
                        config.deactivate_on_window.push(value.to_string());
                    }
                    Ok(())
                }
                _ => Ok(()),
            };
            if let Err(e) = result {
                log!("Ignoring config line '{line}': {e}");
            }
        }

        config
    }

    fn set_setting(&mut self, key: &str, value: &str) -> Result<(), String> {
        match key {
            "RunAsAdmin" => self.run_as_admin = parse_bool(value)?,
            "TitleMatchMode" => {
                self.title_match_mode = match value.parse() {
                    Ok(mode @ 1..=3) => mode,
                    _ => return Err("expected 1, 2 or 3".into()),
                }
            }
            "ResetOnStartup" => self.reset_on_startup = parse_bool(value)?,
            "SetAffinity" => self.set_affinity = parse_bool(value)?,
            "SetCracklingFix" => self.set_crackling_fix = parse_bool(value)?,
            "VoicemeeterPath" => {
                self.voicemeeter_path = match value {
                    "" | "default" => None,
                    path => Some(PathBuf::from(path)),
                }
            }
            _ => {}
        }
        Ok(())
    }

    fn set_voicemeeter_setting(&mut self, key: &str, value: &str) -> Result<(), String> {
        let int = || value.parse::<i32>().map_err(|e| e.to_string());
        match key {
            "OUTPUT_1" => self.outputs.output1 = int()?,
            "OUTPUT_2" => self.outputs.output2 = int()?,
            "OUTPUT_3" => self.outputs.output3 = int()?,
            "VOLUME_CHANGE_AMOUNT" => {
                self.volume_change_amount = value.parse().map_err(|e| format!("{e}"))?
            }
            "default_file" => self.default_file = value.into(),
            "profile1_file" => self.profile1_file = value.into(),
            "profile2_file" => self.profile2_file = value.into(),
            _ => {}
        }
        Ok(())
    }
}

fn parse_bool(value: &str) -> Result<bool, String> {
    if value.eq_ignore_ascii_case("true") {
        Ok(true)
    } else if value.eq_ignore_ascii_case("false") {
        Ok(false)
    } else {
        Err(format!("expected True or False, got '{value}'"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_file_matches_the_original_format() {
        let expected = "[Settings]\nRunAsAdmin=True\nTitleMatchMode=3\nResetOnStartup=True\n\
                        SetAffinity=True\nSetCracklingFix=True\nVoicemeeterPath=default\n\n\
                        [VoicemeeterSettings]\nOUTPUT_1=5\nOUTPUT_2=6\nOUTPUT_3=7\n\
                        VOLUME_CHANGE_AMOUNT=0.5\ndefault_file=default.xml\n\
                        profile1_file=profile1.xml\nprofile2_file=profile2.xml\n\n\
                        [DeactivateOnWindow]\n";
        assert_eq!(Config::default().to_ini(), expected);
        assert_eq!(Config::parse(expected), Config::default());
    }

    #[test]
    fn saved_config_round_trips() {
        let config = Config {
            run_as_admin: false,
            title_match_mode: 1,
            reset_on_startup: false,
            set_affinity: false,
            set_crackling_fix: true,
            voicemeeter_path: Some(PathBuf::from(r"D:\VB\Voicemeeter")),
            outputs: Outputs {
                output1: 0,
                output2: 3,
                output3: 4,
            },
            volume_change_amount: 1.25,
            default_file: "home.xml".into(),
            profile1_file: "game.xml".into(),
            profile2_file: "stream.xml".into(),
            deactivate_on_window: vec!["Game".into(), "a = b".into()],
        };
        assert_eq!(Config::parse(&config.to_ini()), config);
    }

    #[test]
    fn overrides_are_applied() {
        let config = Config::parse(
            "\u{feff}[Settings]\nRunAsAdmin=false\nTitleMatchMode=2\nResetOnStartup=FALSE\n\
             VoicemeeterPath=D:\\VB\\Voicemeeter\n\
             [VoicemeeterSettings]\nOUTPUT_1 = 1\nOUTPUT_2=2\nOUTPUT_3=3\n\
             VOLUME_CHANGE_AMOUNT=1.25\ndefault_file=a.xml\nprofile1_file=b.xml\nprofile2_file=c.xml\n",
        );
        assert!(!config.run_as_admin);
        assert_eq!(config.title_match_mode, 2);
        assert!(!config.reset_on_startup);
        assert!(config.set_affinity);
        assert_eq!(
            config.voicemeeter_path,
            Some(PathBuf::from("D:\\VB\\Voicemeeter"))
        );
        assert_eq!(
            config.outputs,
            Outputs {
                output1: 1,
                output2: 2,
                output3: 3
            }
        );
        assert_eq!(config.volume_change_amount, 1.25);
        assert_eq!(
            (
                config.default_file.as_str(),
                config.profile1_file.as_str(),
                config.profile2_file.as_str()
            ),
            ("a.xml", "b.xml", "c.xml")
        );
    }

    #[test]
    fn default_voicemeeter_path_is_none() {
        let config = Config::parse("[Settings]\nVoicemeeterPath=default\n");
        assert_eq!(config.voicemeeter_path, None);
    }

    #[test]
    fn deactivate_on_window_accepts_keyed_and_bare_entries() {
        let config = Config::parse("[DeactivateOnWindow]\n1=Game One\nGame Two\n; comment\n2=\n");
        assert_eq!(config.deactivate_on_window, ["Game One", "Game Two"]);
    }

    #[test]
    fn malformed_values_keep_defaults() {
        let config = Config::parse(
            "[Settings]\nSetAffinity=maybe\nTitleMatchMode=7\n[VoicemeeterSettings]\nOUTPUT_1=x\nVOLUME_CHANGE_AMOUNT=\n",
        );
        assert_eq!(config, Config::default());
    }

    #[test]
    fn outputs_for_buttons() {
        let outputs = Config::default().outputs;
        assert_eq!(outputs.for_buttons(true, false), Some(5));
        assert_eq!(outputs.for_buttons(false, true), Some(6));
        assert_eq!(outputs.for_buttons(true, true), Some(7));
        assert_eq!(outputs.for_buttons(false, false), None);
    }
}
