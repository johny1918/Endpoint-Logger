use std::env;
use crate::utils::errors::AppError;
use url::Url;

#[derive(Clone, Debug)]
pub struct AppConfig {
    pub database_url: String,
    pub port: u32,
    pub target_url: String,
}

impl AppConfig {
    pub fn from_env() -> Self {
        Self { 
            database_url: env::var("DATABASE_URL").expect("DATABSE_URL is required"),
            port: env::var("PORT").unwrap_or_else(|_| "3000".into()).parse().unwrap(),
            target_url: env::var("TARGET_URL").expect("TARGET_URL is required"),
        }
    }

    fn is_port_range(&self) -> Result<bool, AppError>{
        if self.port > 65535 {
            return Err(AppError::InvalidPortRange);
        }
        Ok(true)
    }

    fn check_target_url_format(&self) -> Result<bool, AppError> {
        match Url::parse(&self.target_url) {
            Ok(url) => {
                if url.scheme() == "http" || url.scheme() == "https" {
                    if url.host_str().is_some() {
                        Ok(true)
                    } else {
                        Err(AppError::InvalidURLFormat)
                    }
                } else {
                    Err(AppError::InvalidURLFormat)
                }
            }
            Err(_) => Err(AppError::InvalidURLFormat),
        }
    }
}

#[test]
fn test_is_port_range_valid() {
    let config = AppConfig {
        database_url: "dummy".to_string(),
        port: 3000,
        target_url: "dummy".to_string(),
    };
    assert!(config.is_port_range().is_ok());
}

#[test]
fn test_is_port_range_invalid() {
    let config = AppConfig {
        database_url: "dummy".to_string(),
        port: 70000,
        target_url: "dummy".to_string(),
    };
    assert!(config.is_port_range().is_err());
}

#[test]
fn test_check_target_url_format_valid() {
    let config = AppConfig {
        database_url: "dummy".to_string(),
        port: 3000,
        target_url: "http://example.com".to_string(),
    };
    assert!(config.check_target_url_format().is_ok());
}

#[test]
fn test_check_target_url_format_invalid() {
    let config = AppConfig {
        database_url: "dummy".to_string(),
        port: 3000,
        target_url: "example.com".to_string(),
    };
    assert!(config.check_target_url_format().is_err());
}

#[test]
fn test_from_env_with_valid_env() {
    unsafe {
        std::env::set_var("DATABASE_URL", "postgres://user:pass@localhost/db");
        std::env::set_var("PORT", "8080");
        std::env::set_var("TARGET_URL", "http://target.com");
    }

    let config = AppConfig::from_env();

    assert_eq!(config.database_url, "postgres://user:pass@localhost/db");
    assert_eq!(config.port, 8080);
    assert_eq!(config.target_url, "http://target.com");

    // Clean up
    unsafe {
        std::env::remove_var("DATABASE_URL");
        std::env::remove_var("PORT");
        std::env::remove_var("TARGET_URL");
    }
}
