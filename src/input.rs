const WM_KEYDOWN: u32 = 0x0100;
const VK_F8: usize = 0x77;
const PREVIOUS_KEY_STATE: isize = 1 << 30;

/// Accepts only the first F8 key-down event, never Windows auto-repeat events.
pub fn is_visibility_toggle(message: u32, key: usize, flags: isize) -> bool {
    message == WM_KEYDOWN && key == VK_F8 && flags & PREVIOUS_KEY_STATE == 0
}

pub fn is_diagnostics_toggle(message: u32, key: usize, flags: isize) -> bool {
    message == WM_KEYDOWN && key == 0x78 && flags & PREVIOUS_KEY_STATE == 0
}

pub fn placement_adjustment(message: u32, key: usize, flags: isize) -> Option<i32> {
    if message != WM_KEYDOWN || flags & PREVIOUS_KEY_STATE != 0 {
        return None;
    }
    match key {
        0x75 => Some(8),
        0x76 => Some(-8),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn one_press_followed_by_repeats_toggles_once() {
        let events = [1, PREVIOUS_KEY_STATE | 1, PREVIOUS_KEY_STATE | 3];
        assert_eq!(
            events
                .into_iter()
                .filter(|flags| is_visibility_toggle(WM_KEYDOWN, VK_F8, *flags))
                .count(),
            1
        );
    }

    #[test]
    fn ignores_key_up_other_keys_and_system_key_messages() {
        assert!(!is_visibility_toggle(0x0101, VK_F8, 0));
        assert!(!is_visibility_toggle(WM_KEYDOWN, 0x20, 0));
        assert!(!is_visibility_toggle(0x0104, VK_F8, 0));
    }

    #[test]
    fn subsequent_fresh_presses_are_accepted() {
        assert!(is_visibility_toggle(WM_KEYDOWN, VK_F8, 1));
        assert!(is_visibility_toggle(WM_KEYDOWN, VK_F8, 1));
    }
}
