use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AppCommand {
    SplitRight,
    SplitDown,
    NewSurface,
    CloseSurface,
    FocusLeft,
    FocusRight,
    FocusUp,
    FocusDown,
    PrevSurface,
    NextSurface,
    ToggleSidebar,
    NewWorkspace,
    CloseWorkspace,
    SwitchWorkspace(usize),
    CycleFocusForward,
    CycleFocusBackward,
    Quit,
}

pub fn command_for_key(key: KeyEvent) -> Option<AppCommand> {
    if key.code == KeyCode::BackTab && !key.modifiers.contains(KeyModifiers::ALT) {
        return Some(AppCommand::CycleFocusBackward);
    }

    // Tab / Shift+Tab: focus cycle without Alt.
    if key.code == KeyCode::Tab && !key.modifiers.contains(KeyModifiers::ALT) {
        if key.modifiers.contains(KeyModifiers::SHIFT) {
            return Some(AppCommand::CycleFocusBackward);
        }
        return Some(AppCommand::CycleFocusForward);
    }

    if !key.modifiers.contains(KeyModifiers::ALT) {
        return None;
    }

    match key.code {
        KeyCode::Left => Some(AppCommand::FocusLeft),
        KeyCode::Right => Some(AppCommand::FocusRight),
        KeyCode::Up => Some(AppCommand::FocusUp),
        KeyCode::Down => Some(AppCommand::FocusDown),
        KeyCode::Char('[') => Some(AppCommand::PrevSurface),
        KeyCode::Char(']') => Some(AppCommand::NextSurface),
        KeyCode::Char(c) if c.eq_ignore_ascii_case(&'d') => Some(AppCommand::SplitRight),
        KeyCode::Char(c) if c.eq_ignore_ascii_case(&'s') => Some(AppCommand::SplitDown),
        KeyCode::Char(c) if c.eq_ignore_ascii_case(&'t') => Some(AppCommand::NewSurface),
        KeyCode::Char(c) if c.eq_ignore_ascii_case(&'w') => Some(AppCommand::CloseSurface),
        KeyCode::Char(c) if c.eq_ignore_ascii_case(&'c') => Some(AppCommand::NewWorkspace),
        KeyCode::Char(c) if c.eq_ignore_ascii_case(&'x') => Some(AppCommand::CloseWorkspace),
        KeyCode::Char(c) if c.eq_ignore_ascii_case(&'b') => Some(AppCommand::ToggleSidebar),
        KeyCode::Char(c) if c.eq_ignore_ascii_case(&'q') => Some(AppCommand::Quit),
        KeyCode::Char(c) if c.is_ascii_digit() && c != '0' => {
            Some(AppCommand::SwitchWorkspace(c as usize - '1' as usize))
        }
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

    use super::{command_for_key, AppCommand};

    #[test]
    fn tab_cycles_focus_forward() {
        assert_eq!(
            command_for_key(key(KeyCode::Tab, KeyModifiers::empty())),
            Some(AppCommand::CycleFocusForward)
        );
    }

    #[test]
    fn shift_tab_cycles_focus_backward() {
        assert_eq!(
            command_for_key(key(KeyCode::Tab, KeyModifiers::SHIFT)),
            Some(AppCommand::CycleFocusBackward)
        );
    }

    #[test]
    fn backtab_cycles_focus_backward() {
        assert_eq!(
            command_for_key(key(KeyCode::BackTab, KeyModifiers::SHIFT)),
            Some(AppCommand::CycleFocusBackward)
        );
    }

    #[test]
    fn alt_letter_commands_are_case_insensitive() {
        assert_eq!(
            command_for_key(key(
                KeyCode::Char('T'),
                KeyModifiers::ALT | KeyModifiers::SHIFT
            )),
            Some(AppCommand::NewSurface)
        );
    }

    #[test]
    fn alt_arrows_map_to_focus_commands() {
        assert_eq!(
            command_for_key(key(KeyCode::Left, KeyModifiers::ALT)),
            Some(AppCommand::FocusLeft)
        );
        assert_eq!(
            command_for_key(key(KeyCode::Down, KeyModifiers::ALT)),
            Some(AppCommand::FocusDown)
        );
    }

    #[test]
    fn alt_surface_navigation_commands() {
        assert_eq!(
            command_for_key(key(KeyCode::Char('['), KeyModifiers::ALT)),
            Some(AppCommand::PrevSurface)
        );
        assert_eq!(
            command_for_key(key(KeyCode::Char(']'), KeyModifiers::ALT)),
            Some(AppCommand::NextSurface)
        );
    }

    #[test]
    fn alt_digit_switches_workspace() {
        assert_eq!(
            command_for_key(key(KeyCode::Char('3'), KeyModifiers::ALT)),
            Some(AppCommand::SwitchWorkspace(2))
        );
    }

    #[test]
    fn non_alt_keys_are_ignored() {
        assert_eq!(
            command_for_key(key(KeyCode::Char('d'), KeyModifiers::empty())),
            None
        );
    }

    #[test]
    fn unsupported_alt_keys_are_ignored() {
        assert_eq!(command_for_key(key(KeyCode::F(5), KeyModifiers::ALT)), None);
    }

    fn key(code: KeyCode, modifiers: KeyModifiers) -> KeyEvent {
        KeyEvent::new(code, modifiers)
    }
}
