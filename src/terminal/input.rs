use std::time::{Duration, Instant};

use anyhow::{bail, Result};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct KeyBinding {
    raw: String,
    modifiers: KeyModifiers,
    code: KeyCode,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PrefixCommand {
    FocusPrev,
    FocusNext,
    NewPane,
    ClosePane,
    NewWorkspace,
    SwitchWorkspace(usize),
    Quit,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum InputAction {
    ForwardToPty(KeyEvent),
    EnterPrefixMode,
    PrefixCommand(PrefixCommand),
    PrefixTimedOut,
    Noop,
}

pub struct PrefixRouter {
    binding: KeyBinding,
    timeout: Duration,
    prefix_started_at: Option<Instant>,
}

impl KeyBinding {
    pub fn parse(value: &str) -> Result<Self> {
        let trimmed = value.trim();
        if trimmed.is_empty() {
            bail!("prefix key cannot be empty");
        }

        let tokens = trimmed
            .split('-')
            .map(|part| part.trim())
            .filter(|part| !part.is_empty())
            .collect::<Vec<_>>();
        if tokens.is_empty() {
            bail!("invalid prefix key `{trimmed}`");
        }

        let mut modifiers = KeyModifiers::empty();
        for token in &tokens[..tokens.len().saturating_sub(1)] {
            match token.to_ascii_lowercase().as_str() {
                "ctrl" | "control" => modifiers |= KeyModifiers::CONTROL,
                "alt" => modifiers |= KeyModifiers::ALT,
                "shift" => modifiers |= KeyModifiers::SHIFT,
                "super" | "cmd" | "command" | "meta" => modifiers |= KeyModifiers::SUPER,
                other => bail!("unsupported modifier `{other}` in prefix key `{trimmed}`"),
            }
        }

        let code = parse_key_code(tokens[tokens.len() - 1])?;
        Ok(Self {
            raw: trimmed.to_string(),
            modifiers,
            code,
        })
    }

    pub fn matches(&self, key: KeyEvent) -> bool {
        if key.modifiers != self.modifiers {
            return false;
        }

        match (self.code, key.code) {
            (KeyCode::Char(expected), KeyCode::Char(actual)) => {
                expected.eq_ignore_ascii_case(&actual)
            }
            _ => self.code == key.code,
        }
    }

    pub fn raw(&self) -> &str {
        &self.raw
    }
}

impl PrefixRouter {
    pub fn new(binding: KeyBinding, timeout: Duration) -> Self {
        Self {
            binding,
            timeout,
            prefix_started_at: None,
        }
    }

    pub fn route_key(&mut self, key: KeyEvent, now: Instant) -> InputAction {
        let _ = self.expire(now);

        if self.binding.matches(key) {
            self.prefix_started_at = Some(now);
            return InputAction::EnterPrefixMode;
        }

        if self.is_prefix_active() {
            self.prefix_started_at = None;
            return map_prefix_key(key);
        }

        InputAction::ForwardToPty(key)
    }

    pub fn expire(&mut self, now: Instant) -> Option<InputAction> {
        if self
            .prefix_started_at
            .is_some_and(|started| now.duration_since(started) >= self.timeout)
        {
            self.prefix_started_at = None;
            return Some(InputAction::PrefixTimedOut);
        }

        None
    }

    pub fn is_prefix_active(&self) -> bool {
        self.prefix_started_at.is_some()
    }

    pub fn binding_label(&self) -> &str {
        self.binding.raw()
    }

    pub fn timeout(&self) -> Duration {
        self.timeout
    }
}

fn parse_key_code(token: &str) -> Result<KeyCode> {
    let lower = token.to_ascii_lowercase();
    match lower.as_str() {
        "enter" => Ok(KeyCode::Enter),
        "esc" | "escape" => Ok(KeyCode::Esc),
        "tab" => Ok(KeyCode::Tab),
        "space" => Ok(KeyCode::Char(' ')),
        other if other.chars().count() == 1 => Ok(KeyCode::Char(other.chars().next().unwrap())),
        _ => bail!("unsupported key `{token}` in prefix binding"),
    }
}

fn map_prefix_key(key: KeyEvent) -> InputAction {
    match key.code {
        KeyCode::Left | KeyCode::Up => InputAction::PrefixCommand(PrefixCommand::FocusPrev),
        KeyCode::Right | KeyCode::Down => InputAction::PrefixCommand(PrefixCommand::FocusNext),
        KeyCode::Char(c) => match c.to_ascii_lowercase() {
            'n' => InputAction::PrefixCommand(PrefixCommand::NewPane),
            'x' => InputAction::PrefixCommand(PrefixCommand::ClosePane),
            'c' => InputAction::PrefixCommand(PrefixCommand::NewWorkspace),
            'q' => InputAction::PrefixCommand(PrefixCommand::Quit),
            digit if ('1'..='9').contains(&digit) => InputAction::PrefixCommand(
                PrefixCommand::SwitchWorkspace(digit as usize - '1' as usize),
            ),
            _ => InputAction::Noop,
        },
        _ => InputAction::Noop,
    }
}

#[cfg(test)]
mod tests {
    use std::time::{Duration, Instant};

    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

    use super::{InputAction, KeyBinding, PrefixRouter};

    #[test]
    fn parse_prefix_ctrl_a() {
        let binding = KeyBinding::parse("Ctrl-a").unwrap();
        assert_eq!(binding.modifiers, KeyModifiers::CONTROL);
        assert_eq!(binding.code, KeyCode::Char('a'));
    }

    #[test]
    fn parse_prefix_alt_shift() {
        let binding = KeyBinding::parse("Alt-Shift-x").unwrap();
        assert!(binding.modifiers.contains(KeyModifiers::ALT));
        assert!(binding.modifiers.contains(KeyModifiers::SHIFT));
        assert_eq!(binding.code, KeyCode::Char('x'));
    }

    #[test]
    fn parse_prefix_empty_fails() {
        assert!(KeyBinding::parse("").is_err());
    }

    #[test]
    fn parse_prefix_invalid_modifier() {
        assert!(KeyBinding::parse("Hyper-a").is_err());
    }

    #[test]
    fn keybinding_matches_correct_key() {
        let binding = KeyBinding::parse("Ctrl-a").unwrap();
        assert!(binding.matches(key(KeyCode::Char('a'), KeyModifiers::CONTROL)));
    }

    #[test]
    fn keybinding_rejects_wrong_modifier() {
        let binding = KeyBinding::parse("Ctrl-a").unwrap();
        assert!(!binding.matches(key(KeyCode::Char('a'), KeyModifiers::ALT)));
    }

    #[test]
    fn keybinding_case_insensitive() {
        let binding = KeyBinding::parse("Ctrl-a").unwrap();
        assert!(binding.matches(key(KeyCode::Char('A'), KeyModifiers::CONTROL)));
    }

    #[test]
    fn normal_mode_forwards_key() {
        let binding = KeyBinding::parse("Ctrl-a").unwrap();
        let mut router = PrefixRouter::new(binding, Duration::from_secs(2));
        let action = router.route_key(
            key(KeyCode::Char('z'), KeyModifiers::empty()),
            Instant::now(),
        );

        assert!(matches!(action, InputAction::ForwardToPty(_)));
    }

    #[test]
    fn prefix_mode_enter() {
        let binding = KeyBinding::parse("Ctrl-a").unwrap();
        let mut router = PrefixRouter::new(binding, Duration::from_secs(2));
        let action = router.route_key(
            key(KeyCode::Char('a'), KeyModifiers::CONTROL),
            Instant::now(),
        );

        assert_eq!(action, InputAction::EnterPrefixMode);
        assert!(router.is_prefix_active());
    }

    #[test]
    fn prefix_mode_timeout() {
        let binding = KeyBinding::parse("Ctrl-a").unwrap();
        let mut router = PrefixRouter::new(binding, Duration::from_secs(2));
        let now = Instant::now();
        let _ = router.route_key(key(KeyCode::Char('a'), KeyModifiers::CONTROL), now);
        let timed_out = router.expire(now + Duration::from_secs(2));

        assert_eq!(timed_out, Some(InputAction::PrefixTimedOut));
        assert!(!router.is_prefix_active());
    }

    fn key(code: KeyCode, modifiers: KeyModifiers) -> KeyEvent {
        KeyEvent::new(code, modifiers)
    }
}
