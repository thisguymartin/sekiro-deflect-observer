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

/// F11 arms practice for this process only. Never capture the key from the game.
pub fn is_practice_toggle(message: u32, key: usize, flags: isize) -> bool {
    message == WM_KEYDOWN && key == 0x7a && flags & PREVIOUS_KEY_STATE == 0
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PracticeAction {
    Toggle,
    CycleSpeed,
}

/// Shift+F11 selects speed without changing the session's on/off state.
pub fn practice_action(
    message: u32,
    key: usize,
    flags: isize,
    shift: bool,
) -> Option<PracticeAction> {
    is_practice_toggle(message, key, flags).then_some(if shift {
        PracticeAction::CycleSpeed
    } else {
        PracticeAction::Toggle
    })
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

/// F10 resets persisted offsets. Like all observer hotkeys, never consumes input.
pub fn is_placement_reset(message: u32, key: usize, flags: isize) -> bool {
    message == WM_KEYDOWN && key == 0x79 && flags & PREVIOUS_KEY_STATE == 0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn placement_controls_ignore_repeat_key_up_and_combat_keys() {
        assert_eq!(placement_adjustment(WM_KEYDOWN, 0x75, 0), Some(8));
        assert_eq!(placement_adjustment(WM_KEYDOWN, 0x76, 0), Some(-8));
        assert!(is_placement_reset(WM_KEYDOWN, 0x79, 0));
        assert!(is_practice_toggle(WM_KEYDOWN, 0x7a, 0));
        for (message, flags) in [(WM_KEYDOWN, PREVIOUS_KEY_STATE), (0x0101, 0), (0x0104, 0)] {
            assert_eq!(placement_adjustment(message, 0x75, flags), None);
            assert!(!is_placement_reset(message, 0x79, flags));
            assert!(!is_diagnostics_toggle(message, 0x78, flags));
            assert!(!is_practice_toggle(message, 0x7a, flags));
        }
        for key in [0x01, 0x02, 0x20, 0x57, 0x45] {
            assert!(!is_visibility_toggle(WM_KEYDOWN, key, 0));
            assert!(!is_diagnostics_toggle(WM_KEYDOWN, key, 0));
            assert!(!is_practice_toggle(WM_KEYDOWN, key, 0));
            assert!(placement_adjustment(WM_KEYDOWN, key, 0).is_none());
            assert!(!is_placement_reset(WM_KEYDOWN, key, 0));
        }
    }

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

    #[test]
    fn practice_speed_shortcut_is_distinct_and_ignores_repeat_and_key_up() {
        assert_eq!(
            practice_action(WM_KEYDOWN, 0x7a, 0, false),
            Some(PracticeAction::Toggle)
        );
        assert_eq!(
            practice_action(WM_KEYDOWN, 0x7a, 0, true),
            Some(PracticeAction::CycleSpeed)
        );
        for shift in [false, true] {
            assert_eq!(
                practice_action(WM_KEYDOWN, 0x7a, PREVIOUS_KEY_STATE, shift),
                None
            );
            assert_eq!(practice_action(0x0101, 0x7a, 0, shift), None);
            assert_eq!(practice_action(WM_KEYDOWN, 0x7b, 0, shift), None);
        }
    }
}
