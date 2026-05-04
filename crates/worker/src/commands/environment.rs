use std::env;

const ENV_VARS: &[EnvVar] = &[
    EnvVar::plain("IBKR_BASE_URL"),
    EnvVar::secret("IBKR_CONSUMER_KEY"),
    EnvVar::plain("IBKR_REALM"),
    EnvVar::secret("IBKR_ACCESS_TOKEN"),
    EnvVar::secret("IBKR_ACCESS_TOKEN_SECRET"),
    EnvVar::plain("IBKR_SIGNATURE_KEY_PATH"),
    EnvVar::plain("IBKR_ENCRYPTION_KEY_PATH"),
    EnvVar::plain("IBKR_DH_PARAM_PATH"),
    EnvVar::plain("IBKR_TIMEOUT_SECONDS"),
    EnvVar::secret("IBKR_DATABASE"),
    EnvVar::plain("IBKR_LST_CACHE_MODE"),
    EnvVar::secret("IBKR_REDIS_URL"),
    EnvVar::plain("IBKR_REDIS_KEY_PREFIX"),
    EnvVar::plain("IBKR_LST_REFRESH_SKEW_SECONDS"),
    EnvVar::plain("IBKR_LST_LOCK_TTL_SECONDS"),
];

#[derive(Debug, Clone, Copy)]
struct EnvVar {
    key: &'static str,
    secret: bool,
}

impl EnvVar {
    const fn plain(key: &'static str) -> Self {
        Self { key, secret: false }
    }

    const fn secret(key: &'static str) -> Self {
        Self { key, secret: true }
    }
}

pub fn render() -> String {
    let mut lines = ENV_VARS
        .iter()
        .map(|var| {
            let value = env::var(var.key)
                .map(|value| display_value(&value, var.secret))
                .unwrap_or_else(|_| "<unset>".to_string());
            format!("{}={}", var.key, value)
        })
        .collect::<Vec<_>>()
        .join("\n");
    lines.push('\n');
    lines
}

fn display_value(value: &str, secret: bool) -> String {
    if secret {
        truncate_secret(value)
    } else {
        value.to_string()
    }
}

fn truncate_secret(value: &str) -> String {
    let chars = value.chars().collect::<Vec<_>>();
    if chars.len() <= 8 {
        return "...".to_string();
    }

    let prefix = chars.iter().take(4).collect::<String>();
    let suffix = chars
        .iter()
        .skip(chars.len().saturating_sub(4))
        .collect::<String>();
    format!("{prefix}...{suffix}")
}

#[cfg(test)]
mod tests {
    use super::{display_value, truncate_secret};

    #[test]
    fn truncates_secret_values() {
        assert_eq!(truncate_secret("abcdefghijkl"), "abcd...ijkl");
        assert_eq!(truncate_secret("abcdefgh"), "...");
        assert_eq!(truncate_secret("abc"), "...");
    }

    #[test]
    fn leaves_plain_values_visible() {
        assert_eq!(display_value("memory", false), "memory");
    }

    #[test]
    fn masks_secret_values() {
        assert_eq!(display_value("secret-token", true), "secr...oken");
    }
}
