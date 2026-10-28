//! Media keys (Forward + left / right / middle click).

use crate::action::MediaKey;
use crate::platform::keyboard;

pub fn press(key: MediaKey) {
    keyboard::tap_media_key(key);
}
