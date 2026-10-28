//! Loading Voicemeeter profiles (F24 / double F24 / double Ctrl+Shift+R) and restarting the
//! audio engine (Ctrl+Shift+R).

use super::{Mixer, MixerSession};
use crate::action::Profile;
use crate::log;

impl<M: Mixer> MixerSession<M> {
    pub(super) fn switch_profile(&mut self, profile: Profile) {
        let files = &self.settings.profiles;
        let target = match profile {
            Profile::Default => files.default.as_str(),
            Profile::Profile1 => toggle(&self.current_profile, &files.profile1, &files.default),
            Profile::Profile2 => toggle(&self.current_profile, &files.profile2, &files.default),
        };
        self.load_profile(target.to_string());
    }

    pub(super) fn load_profile(&mut self, name: String) {
        let path = self.settings.profiles.dir.join(&name);
        match self.mixer.load_profile(&path) {
            Ok(()) => log!("Loaded profile: {}", path.display()),
            Err(e) => log!("Error loading profile {}: {e}", path.display()),
        }
        self.current_profile = name;
        self.sync_strips();
    }

    pub(super) fn restart_engine(&mut self) {
        // Only the audio engine restarts; strip values stay the same.
        match self.mixer.restart_engine() {
            Ok(()) => log!("Voicemeeter audio engine restarted"),
            Err(e) => log!("Error restarting the Voicemeeter audio engine: {e}"),
        }
    }
}

/// Pressing a profile's key while it is loaded goes back to the default profile.
fn toggle<'a>(current: &str, profile: &'a str, default: &'a str) -> &'a str {
    if current == profile { default } else { profile }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::super::fake::{FakeMixer, settings};
    use super::*;
    use crate::action::MixerAction;

    fn mixer_with_profiles() -> FakeMixer {
        let mut mixer = FakeMixer::default();
        for (name, gain) in [
            ("default.xml", 0.0),
            ("profile1.xml", -10.0),
            ("profile2.xml", -20.0),
        ] {
            mixer
                .profiles
                .insert(name.into(), vec![(5, gain, false), (6, gain, true)]);
        }
        mixer
    }

    fn switch(s: &mut MixerSession<FakeMixer>, profile: Profile) {
        s.run(MixerAction::SwitchProfile(profile));
    }

    fn loaded(s: &MixerSession<FakeMixer>) -> Vec<String> {
        s.mixer
            .loaded
            .iter()
            .map(|p| p.file_name().unwrap().to_string_lossy().into_owned())
            .collect()
    }

    #[test]
    fn profile_keys_toggle_back_to_default() {
        let mut s = MixerSession::new(mixer_with_profiles(), settings());
        switch(&mut s, Profile::Profile1);
        switch(&mut s, Profile::Profile1);
        switch(&mut s, Profile::Profile2);
        switch(&mut s, Profile::Profile1);
        switch(&mut s, Profile::Default);
        switch(&mut s, Profile::Default);
        assert_eq!(
            loaded(&s),
            [
                "profile1.xml",
                "default.xml",
                "profile2.xml",
                "profile1.xml",
                "default.xml",
                "default.xml"
            ]
        );
    }

    #[test]
    fn profiles_load_from_the_profile_dir() {
        let mut s = MixerSession::new(mixer_with_profiles(), settings());
        switch(&mut s, Profile::Profile2);
        assert_eq!(s.mixer.loaded, [PathBuf::from("profiles/profile2.xml")]);
    }

    #[test]
    fn loading_resyncs_strip_values() {
        let mut s = MixerSession::new(mixer_with_profiles(), settings());
        s.run(MixerAction::AdjustVolume { strip: 5, up: true });
        switch(&mut s, Profile::Profile1);
        // Builds on the profile's -10 dB, not the earlier +0.5 dB.
        s.run(MixerAction::AdjustVolume { strip: 5, up: true });
        assert_eq!(s.mixer.settled_gain(5), Some(-9.5));
    }

    #[test]
    fn missing_profile_still_counts_as_current() {
        let mut s = MixerSession::new(FakeMixer::default(), settings());
        switch(&mut s, Profile::Profile1);
        assert!(s.mixer.loaded.is_empty());
        assert_eq!(s.current_profile, "profile1.xml");
    }

    #[test]
    fn start_loads_default_or_syncs() {
        let mut s = MixerSession::new(mixer_with_profiles(), settings());
        s.start(true);
        assert_eq!(loaded(&s), ["default.xml"]);

        let mut mixer = FakeMixer::default();
        mixer.mutes.insert(6, true);
        let mut s = MixerSession::new(mixer, settings());
        s.start(false);
        assert!(s.mixer.loaded.is_empty());
        s.run(MixerAction::ToggleMute { strip: 6 });
        assert_eq!(s.mixer.settled_mute(6), Some(false));
    }

    #[test]
    fn restart_engine() {
        let mut s = MixerSession::new(FakeMixer::default(), settings());
        s.run(MixerAction::RestartEngine);
        assert_eq!(s.mixer.restarts, 1);
    }
}
