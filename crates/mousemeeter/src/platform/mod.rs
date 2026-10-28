//! Win32 adapters: hooks, tray icon, processes and logging. No decisions are made here; input is
//! handed to `crate::gestures` and actions come back from `crate::use_cases`.

pub mod keyboard;
pub mod logger;
pub mod mouse;
pub mod process;
pub mod tray;
