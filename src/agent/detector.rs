#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AgentState {
    Idle,
    Working,
    Dead,
}

pub fn detect_state(output: &str, idle_pattern: Option<&str>, exited: bool) -> AgentState {
    if exited {
        return AgentState::Dead;
    }

    match idle_pattern {
        Some("") => AgentState::Idle,
        Some(pattern) if output.contains(pattern) => AgentState::Idle,
        _ => AgentState::Working,
    }
}

#[cfg(test)]
mod tests {
    use super::{detect_state, AgentState};

    #[test]
    fn detect_idle_by_prompt() {
        assert_eq!(
            detect_state("user@host:~$ ", Some("$ "), false),
            AgentState::Idle
        );
    }

    #[test]
    fn detect_working_no_match() {
        assert_eq!(
            detect_state("Thinking...", Some("$ "), false),
            AgentState::Working
        );
    }

    #[test]
    fn detect_dead_on_empty() {
        assert_eq!(detect_state("", Some("$ "), true), AgentState::Dead);
    }

    #[test]
    fn detect_custom_pattern() {
        assert_eq!(detect_state(" ", Some(""), false), AgentState::Idle);
    }
}
