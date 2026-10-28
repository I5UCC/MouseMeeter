//! Process tweaks applied once Voicemeeter runs: higher priority for Voicemeeter (`SetAffinity`)
//! and pinning `audiodg` to one core against crackling (`SetCracklingFix`).

use crate::log;
use crate::platform::process::{self, Priority};

pub fn apply(voicemeeter_pids: &[u32], raise_priority: bool, crackling_fix: bool) {
    if raise_priority {
        for &pid in voicemeeter_pids {
            match process::set_priority(pid, Priority::High) {
                Ok(()) => log!("Set process priority to High for Voicemeeter (PID: {pid})"),
                Err(e) => log!("Failed to set process priority for PID {pid}: {e}"),
            }
        }
        match process::set_own_priority(Priority::BelowNormal) {
            Ok(()) => log!("Set Mousemeeter process priority to BelowNormal"),
            Err(e) => log!("Failed to set Mousemeeter process priority: {e}"),
        }
    }

    if crackling_fix {
        let pids = process::find_pids(&["audiodg"]);
        if pids.is_empty() {
            log!("Failed to apply crackling fix: audiodg is not running");
        }
        for pid in pids {
            let result = process::pin_to_first_core(pid)
                .and_then(|()| process::set_priority(pid, Priority::High));
            match result {
                Ok(()) => log!("Applied crackling fix (audiodg PID: {pid})"),
                Err(e) => log!("Failed to apply crackling fix: {e}"),
            }
        }
    }
}
