pub fn is_autobindable(name: &str) -> bool {
    name.chars().next().is_some_and(char::is_lowercase)
}
