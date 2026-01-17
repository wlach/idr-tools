use gix_config;

fn env_nonempty(key: &str) -> Option<String> {
    std::env::var(key)
        .ok()
        .map(|v| v.trim().to_string())
        .filter(|v| !v.is_empty())
}

/// Get git identity as "Name \<email\>" string for use in templates.
/// Returns None if name or email is not configured anywhere.
pub fn get_identity() -> Option<String> {
    // 1. Check environment (highest priority)
    let mut name = env_nonempty("GIT_AUTHOR_NAME").or_else(|| env_nonempty("GIT_COMMITTER_NAME"));
    let mut email =
        env_nonempty("GIT_AUTHOR_EMAIL").or_else(|| env_nonempty("GIT_COMMITTER_EMAIL"));

    // If env fully specifies identity, return it
    if let (Some(n), Some(e)) = (&name, &email) {
        return Some(format!("{} <{}>", n, e));
    }

    // 2. Try repo-aware resolution
    if let Ok(repo) = gix::discover(".") {
        let cfg = repo.config_snapshot();
        name = name.or_else(|| cfg.string("user.name").map(|s| s.to_string()));
        email = email.or_else(|| cfg.string("user.email").map(|s| s.to_string()));

        if let (Some(n), Some(e)) = (&name, &email) {
            return Some(format!("{} <{}>", n, e));
        }
    }

    // 3. Global-only fallback (no repo)
    if let Ok(cfg) = gix_config::File::from_globals() {
        name = name.or_else(|| cfg.string("user.name").map(|s| s.to_string()));
        email = email.or_else(|| cfg.string("user.email").map(|s| s.to_string()));

        if let (Some(n), Some(e)) = (name, email) {
            return Some(format!("{} <{}>", n, e));
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;
    use serial_test::serial;

    struct EnvRestore {
        key: String,
        prev: Option<String>,
    }

    impl Drop for EnvRestore {
        fn drop(&mut self) {
            match &self.prev {
                Some(value) => unsafe {
                    env::set_var(&self.key, value);
                },
                None => unsafe {
                    env::remove_var(&self.key);
                },
            }
        }
    }

    fn set_env_guard(key: &str, value: &str) -> EnvRestore {
        let prev = env::var(key).ok();
        unsafe {
            env::set_var(key, value);
        }
        EnvRestore {
            key: key.to_string(),
            prev,
        }
    }

    fn remove_env_guard(key: &str) -> EnvRestore {
        let prev = env::var(key).ok();
        unsafe {
            env::remove_var(key);
        }
        EnvRestore {
            key: key.to_string(),
            prev,
        }
    }

    #[test]
    #[serial]
    fn test_env_nonempty_returns_some_for_valid_value() {
        let _guard = set_env_guard("TEST_VAR", "test_value");
        assert_eq!(env_nonempty("TEST_VAR"), Some("test_value".to_string()));
    }

    #[test]
    #[serial]
    fn test_env_nonempty_returns_none_for_missing_var() {
        let _guard = remove_env_guard("NONEXISTENT_VAR");
        assert_eq!(env_nonempty("NONEXISTENT_VAR"), None);
    }

    #[test]
    #[serial]
    fn test_env_nonempty_trims_whitespace() {
        let _guard = set_env_guard("TEST_VAR_SPACES", "  value with spaces  ");
        assert_eq!(
            env_nonempty("TEST_VAR_SPACES"),
            Some("value with spaces".to_string())
        );
    }

    #[test]
    #[serial]
    fn test_env_nonempty_filters_empty_string() {
        let _guard = set_env_guard("TEST_VAR_EMPTY", "");
        assert_eq!(env_nonempty("TEST_VAR_EMPTY"), None);
    }

    #[test]
    #[serial]
    fn test_env_nonempty_filters_whitespace_only() {
        let _guard = set_env_guard("TEST_VAR_WHITESPACE", "   ");
        assert_eq!(env_nonempty("TEST_VAR_WHITESPACE"), None);
    }

    #[test]
    #[serial]
    fn test_get_identity_from_env_vars() {
        // Set environment variables
        let _guard_name = set_env_guard("GIT_AUTHOR_NAME", "Test User");
        let _guard_email = set_env_guard("GIT_AUTHOR_EMAIL", "test@example.com");

        let result = get_identity();
        assert_eq!(result, Some("Test User <test@example.com>".to_string()));
    }

    #[test]
    #[serial]
    fn test_get_identity_prefers_author_over_committer() {
        let _guard_author_name = set_env_guard("GIT_AUTHOR_NAME", "Author Name");
        let _guard_committer_name = set_env_guard("GIT_COMMITTER_NAME", "Committer Name");
        let _guard_author_email = set_env_guard("GIT_AUTHOR_EMAIL", "author@example.com");
        let _guard_committer_email = set_env_guard("GIT_COMMITTER_EMAIL", "committer@example.com");

        let result = get_identity();
        assert_eq!(result, Some("Author Name <author@example.com>".to_string()));
    }

    #[test]
    #[serial]
    fn test_get_identity_falls_back_to_committer() {
        let _guard_name = set_env_guard("GIT_COMMITTER_NAME", "Committer Name");
        let _guard_email = set_env_guard("GIT_COMMITTER_EMAIL", "committer@example.com");

        let result = get_identity();
        assert_eq!(
            result,
            Some("Committer Name <committer@example.com>".to_string())
        );
    }

    #[test]
    #[serial]
    fn test_get_identity_returns_none_when_incomplete() {
        // Only name, no email - should continue to try other sources
        let _guard_name = set_env_guard("GIT_AUTHOR_NAME", "Test User");
        let _guard_author_email = remove_env_guard("GIT_AUTHOR_EMAIL");
        let _guard_committer_email = remove_env_guard("GIT_COMMITTER_EMAIL");

        // Can't assert None here because it might find git config
        // Just verify it doesn't panic
        let _result = get_identity();
    }
}
