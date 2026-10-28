//! Input → `Action`. Pure state machines: the Win32 hooks in `crate::platform` feed them events
//! and carry out what they decide.

pub mod hotkeys;
pub mod mouse;
