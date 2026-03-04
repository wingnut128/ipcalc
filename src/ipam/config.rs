use serde::Deserialize;
use std::path::PathBuf;

#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct IpamConfig {
    pub enabled: bool,
    pub auto_init: bool,
    pub sqlite: SqliteConfig,
}

impl Default for IpamConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            auto_init: true,
            sqlite: SqliteConfig::default(),
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct SqliteConfig {
    pub db_path: Option<String>,
    pub wal_mode: bool,
}

impl Default for SqliteConfig {
    fn default() -> Self {
        Self {
            db_path: None,
            wal_mode: true,
        }
    }
}

/// Resolve the SQLite database path using the following precedence:
/// 1. CLI `--db <path>` flag (passed as `cli_db`)
/// 2. `IPCALC_DB` environment variable
/// 3. `db_path` in config file (via `SqliteConfig`)
/// 4. Default: `$XDG_DATA_HOME/ipcalc/ipcalc.db` (or `~/.local/share/ipcalc/ipcalc.db`)
pub fn resolve_db_path(cli_db: Option<&str>, config: &SqliteConfig) -> String {
    let env_val = std::env::var("IPCALC_DB").ok();
    resolve_db_path_inner(cli_db, env_val.as_deref(), config)
}

/// Pure resolution logic, separated from environment access for testability.
fn resolve_db_path_inner(
    cli_db: Option<&str>,
    env_db: Option<&str>,
    config: &SqliteConfig,
) -> String {
    if let Some(path) = cli_db {
        return path.to_string();
    }

    if let Some(path) = env_db
        && !path.is_empty()
    {
        return path.to_string();
    }

    if let Some(ref path) = config.db_path {
        return path.clone();
    }

    default_db_path()
}

fn default_db_path() -> String {
    let data_dir = dirs::data_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("ipcalc");
    data_dir.join("ipcalc.db").to_string_lossy().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cli_flag_takes_precedence() {
        let config = SqliteConfig {
            db_path: Some("/config/path.db".to_string()),
            wal_mode: true,
        };
        let path = resolve_db_path_inner(Some("/tmp/test.db"), Some("/env/path.db"), &config);
        assert_eq!(path, "/tmp/test.db");
    }

    #[test]
    fn test_env_var_takes_precedence_over_config() {
        let config = SqliteConfig {
            db_path: Some("/config/path.db".to_string()),
            wal_mode: true,
        };
        let path = resolve_db_path_inner(None, Some("/env/path.db"), &config);
        assert_eq!(path, "/env/path.db");
    }

    #[test]
    fn test_empty_env_var_falls_through() {
        let config = SqliteConfig {
            db_path: Some("/config/path.db".to_string()),
            wal_mode: true,
        };
        let path = resolve_db_path_inner(None, Some(""), &config);
        assert_eq!(path, "/config/path.db");
    }

    #[test]
    fn test_config_path_used_when_no_cli_or_env() {
        let config = SqliteConfig {
            db_path: Some("/etc/ipcalc/data.db".to_string()),
            wal_mode: true,
        };
        let path = resolve_db_path_inner(None, None, &config);
        assert_eq!(path, "/etc/ipcalc/data.db");
    }

    #[test]
    fn test_default_path_fallback() {
        let config = SqliteConfig::default();
        let path = resolve_db_path_inner(None, None, &config);
        assert!(path.ends_with("ipcalc/ipcalc.db"));
    }
}
