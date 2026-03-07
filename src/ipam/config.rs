use serde::Deserialize;
use std::path::PathBuf;

#[derive(Debug, Clone, Default, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum Backend {
    #[default]
    Sqlite,
    Postgres,
}

impl std::fmt::Display for Backend {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Sqlite => write!(f, "sqlite"),
            Self::Postgres => write!(f, "postgres"),
        }
    }
}

impl std::str::FromStr for Backend {
    type Err = String;
    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "sqlite" => Ok(Self::Sqlite),
            "postgres" | "postgresql" => Ok(Self::Postgres),
            other => Err(format!("unknown IPAM backend: {other}")),
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct IpamConfig {
    pub enabled: bool,
    pub auto_init: bool,
    pub backend: Backend,
    pub sqlite: SqliteConfig,
    pub postgres: PostgresConfig,
}

impl Default for IpamConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            auto_init: true,
            backend: Backend::default(),
            sqlite: SqliteConfig::default(),
            postgres: PostgresConfig::default(),
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

#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct PostgresConfig {
    pub url: Option<String>,
    pub max_connections: u32,
    pub min_connections: u32,
}

impl Default for PostgresConfig {
    fn default() -> Self {
        Self {
            url: None,
            max_connections: 10,
            min_connections: 2,
        }
    }
}

/// Resolve the PostgreSQL connection URL using the following precedence:
/// 1. CLI `--ipam-db-url <url>` flag (passed as `cli_url`)
/// 2. `IPCALC_IPAM_DB_URL` environment variable
/// 3. `url` in config file (via `PostgresConfig`)
pub fn resolve_postgres_url(cli_url: Option<&str>, config: &PostgresConfig) -> Option<String> {
    let env_val = std::env::var("IPCALC_IPAM_DB_URL").ok();
    resolve_postgres_url_inner(cli_url, env_val.as_deref(), config)
}

fn resolve_postgres_url_inner(
    cli_url: Option<&str>,
    env_url: Option<&str>,
    config: &PostgresConfig,
) -> Option<String> {
    if let Some(url) = cli_url {
        return Some(url.to_string());
    }
    if let Some(url) = env_url
        && !url.is_empty()
    {
        return Some(url.to_string());
    }
    config.url.clone()
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

    #[test]
    fn test_backend_from_str() {
        assert_eq!("sqlite".parse::<Backend>().unwrap(), Backend::Sqlite);
        assert_eq!("postgres".parse::<Backend>().unwrap(), Backend::Postgres);
        assert_eq!("postgresql".parse::<Backend>().unwrap(), Backend::Postgres);
        assert!("unknown".parse::<Backend>().is_err());
    }

    #[test]
    fn test_postgres_url_cli_precedence() {
        let config = PostgresConfig {
            url: Some("postgresql://config".to_string()),
            ..Default::default()
        };
        let url =
            resolve_postgres_url_inner(Some("postgresql://cli"), Some("postgresql://env"), &config);
        assert_eq!(url, Some("postgresql://cli".to_string()));
    }

    #[test]
    fn test_postgres_url_env_precedence() {
        let config = PostgresConfig {
            url: Some("postgresql://config".to_string()),
            ..Default::default()
        };
        let url = resolve_postgres_url_inner(None, Some("postgresql://env"), &config);
        assert_eq!(url, Some("postgresql://env".to_string()));
    }

    #[test]
    fn test_postgres_url_config_fallback() {
        let config = PostgresConfig {
            url: Some("postgresql://config".to_string()),
            ..Default::default()
        };
        let url = resolve_postgres_url_inner(None, None, &config);
        assert_eq!(url, Some("postgresql://config".to_string()));
    }

    #[test]
    fn test_postgres_url_none_when_missing() {
        let config = PostgresConfig::default();
        let url = resolve_postgres_url_inner(None, None, &config);
        assert!(url.is_none());
    }
}
