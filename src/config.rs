use std::env;
use std::fs;
use std::path::Path;
use clap::Parser;
use serde::{Deserialize, Serialize};
use tracing::info;
use toml;
use url::Url;

use crate::utils::errors::AppError;

/// Command-line arguments
/// Only essential flags that users commonly override
#[derive(Parser, Debug)]
#[command(
    name = "endpoint-logger",
    author = "Ioan A.",
    version = env!("CARGO_PKG_VERSION"),
    about = "Privacy-first HTTP request logger - monitors and logs all HTTP traffic",
    long_about = "Endpoint Logger acts as a reverse proxy that captures and logs all HTTP requests \
                  and responses, providing real-time feedback through a dashboard. All data stays \
                  on your local machine."
)]
pub struct CliArgs {
    /// Target application URL to proxy requests to
    ///
    /// Example: http://localhost:8080
    #[arg(
        short = 't',
        long = "target",
        env = "TARGET_URL",
        help = "Target application URL to proxy requests to"
    )]
    pub target: Option<String>,

    /// Port for proxy server to listen on
    ///
    /// Default: 3000
    #[arg(
        short = 'p',
        long = "port",
        env = "PORT",
        help = "Port for proxy server to listen on [default: 3000]"
    )]
    pub port: Option<u16>,

    /// Path to database file for storing logs
    ///
    /// Default: ./endpoint-logs.db
    #[arg(
        short = 'd',
        long = "database",
        env = "DATABASE_PATH",
        help = "Path to SQLite database file [default: ./endpoint-logs.db]"
    )]
    pub database: Option<String>,

    /// Path to TOML configuration file
    ///
    /// If specified, loads configuration from this file first,
    /// then applies CLI argument overrides
    #[arg(
        short = 'c',
        long = "config",
        help = "Path to TOML configuration file"
    )]
    pub config: Option<String>,

    /// Enable verbose logging output
    #[arg(
        short = 'v',
        long = "verbose",
        help = "Enable verbose logging output"
    )]
    pub verbose: bool,
}

/// TOML configuration file structure
/// This contains ALL possible configuration options (future: 50+ fields)
/// For now, we only implement the essential 3 fields
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TomlConfig {
    #[serde(default)]
    pub target_url: Option<String>,

    #[serde(default)]
    pub proxy_port: Option<u16>,

    #[serde(default)]
    pub database_path: Option<String>,
}

/// Main application configuration
/// This is the complete config that will be used at runtime
#[derive(Clone, Debug)]
pub struct AppConfig {
    pub target_url: String,
    pub proxy_port: u16,
    pub database_path: String,
    pub verbose: bool,
}

impl AppConfig {
    /// Load configuration with full priority chain
    /// Priority: CLI > ENV > TOML > Defaults
    ///
    /// This is the main entry point for loading configuration
    pub fn load() -> Result<Self, AppError> {
        // Parse CLI arguments first
        let cli_args = CliArgs::parse();

        // Start with defaults
        let mut config = Self::default();

        // 1. Try to load from TOML file (if --config specified or default exists)
        let toml_path = cli_args.config.as_deref()
            .unwrap_or("endpoint-logger.toml");

        if Path::new(toml_path).exists() {
            let toml_config = Self::load_from_toml(toml_path)?;
            config = config.merge_toml(toml_config);
        } else if cli_args.config.is_some() {
            // User explicitly specified a config file that doesn't exist
            return Err(AppError::ConfigMissing(format!(
                "Configuration file not found: {}\n\
                 Make sure the file exists or remove the --config flag.",
                toml_path
            )));
        }
        // If default config file doesn't exist, that's fine - just skip it

        // 2. Merge environment variables (only the 3 essential ones)
        config = config.merge_env().map_err(|e| AppError::MergeEnvError(e.to_string()))?;

        // 3. Merge CLI arguments (only the 5 essential flags)
        config = config.merge_cli(cli_args);

        // 4. Validate final configuration
        config.validate().map_err(|e| AppError::ValidateConfigError(e.to_string()))?;

        Ok(config)
    }

    /// Load configuration from environment variables only
    /// This implements EP-001-01: Basic Configuration Structure
    #[allow(dead_code)] // Used in tests
    pub fn from_env() -> Result<Self, AppError> {
        let mut config = Self::default();
        config = config.merge_env()?;
        config.validate()?;
        Ok(config)
    }

    /// Load TOML configuration from file
    fn load_from_toml(path: &str) -> Result<TomlConfig, AppError> {
        let contents = fs::read_to_string(path)
            .map_err(|e| AppError::ReadConfigTomlError(format!("Failed to read config file '{}': {}", path, e)))?;

        toml::from_str(&contents)
            .map_err(|e| AppError::ReadConfigTomlError(format!("Failed to parse TOML config file '{}': {}", path, e)))
    }

    /// Merge TOML configuration
    /// TOML values override defaults
    fn merge_toml(mut self, toml: TomlConfig) -> Self {
        if let Some(target) = toml.target_url {
            self.target_url = target;
        }
        if let Some(port) = toml.proxy_port {
            self.proxy_port = port;
        }
        if let Some(database) = toml.database_path {
            self.database_path = database;
        }
        self
    }

    /// Merge environment variables
    /// ENV values override TOML/defaults
    fn merge_env(mut self) -> Result<Self, AppError> {
        if let Ok(target) = env::var("TARGET_URL") {
            self.target_url = target;
        }

        if let Ok(port_str) = env::var("PORT") {
            self.proxy_port = port_str.parse::<u16>()
                .map_err(|_| AppError::MergeEnvError(format!(
                    "Invalid PORT environment variable: '{}'. Must be a number between 1 and 65535.",
                    port_str
                )))?;
        }

        if let Ok(database) = env::var("DATABASE_PATH") {
            self.database_path = database;
        }

        Ok(self)
    }

    /// Merge CLI arguments
    /// CLI values override everything (ENV, TOML, defaults)
    fn merge_cli(mut self, cli: CliArgs) -> Self {
        if let Some(target) = cli.target {
            self.target_url = target;
        }
        if let Some(port) = cli.port {
            self.proxy_port = port;
        }
        if let Some(database) = cli.database {
            self.database_path = database;
        }
        if cli.verbose {
            self.verbose = true;
        }
        self
    }

    /// Validate the configuration
    /// Checks that required fields are set and values are valid
    pub fn validate(&self) -> Result<(), AppError> {
        // Check that target_url is not empty (it's required)
        if self.target_url.is_empty() {
            return Err(AppError::ValidateConfigError(
                "Target URL is required.\n\
                 Provide it via:\n\
                 - CLI: endpoint-logger --target http://localhost:8080\n\
                 - ENV: export TARGET_URL=http://localhost:8080\n\
                 - TOML: target_url = \"http://localhost:8080\" in endpoint-logger.toml"
                    .to_string()
            ));
        }

        // Validate URL format
        self.validate_url().map_err(|e| AppError::ValidateConfigError(e.to_string()))?;

        // Validate port range
        self.validate_port().map_err(|e| AppError::ValidateConfigError(e.to_string()))?;

        Ok(())
    }

    /// Validate URL format (must be http:// or https://)
    fn validate_url(&self) -> Result<(), AppError> {
        match Url::parse(&self.target_url) {
            Ok(url) => {
                if url.scheme() == "http" || url.scheme() == "https" {
                    if url.host_str().is_some() {
                        Ok(())
                    } else {
                        Err(AppError::ValidateURLConfig(format!(
                            "Invalid target URL: '{}' - URL must have a valid host.\n\
                             Example: http://localhost:8080",
                            self.target_url
                        )))
                    }
                } else {
                    Err(AppError::ValidateURLConfig(format!(
                        "Invalid target URL: '{}' - URL must start with http:// or https://.\n\
                         Example: http://localhost:8080",
                        self.target_url
                    )))
                }
            }
            Err(_) => Err(AppError::ValidateURLConfig(format!(
                "Invalid target URL format: '{}'.\n\
                 URL must be valid and start with http:// or https://.\n\
                 Example: http://localhost:8080",
                self.target_url
            ))),
        }
    }

    /// Validate port range (1-65535)
    fn validate_port(&self) -> Result<(), AppError> {
        if self.proxy_port == 0 {
            return Err(AppError::ValidatePORTConfig(format!(
                "Invalid port: {}. Port must be between 1 and 65535.",
                self.proxy_port
            )));
        }
        Ok(())
    }

    pub fn print_config_used(&self) {
        let cargo_content = fs::read_to_string("./Cargo.toml")
            .map_err(|_| AppError::CargoTomlError);
        match cargo_content {
            Ok(content) => {
                if let Ok(toml_value) = toml::from_str::<toml::Value>(&content) {
                    let version = toml_value.get("package")
                        .and_then(|p| p.get("version"))
                        .and_then(|v| v.as_str())
                        .unwrap_or("unknown");
                    println!("HTTP Logger start configuration: \r
                        Semantic Version: {} \r
                        Proxy Port: {} \r
                        Target URL: {} \r
                        Ready", 
                        version, self.proxy_port, self.target_url
                    );
                    info!("HTTP Logger start configuration: : \
                        Semantic Version: {} \
                        Proxy Port: {} \
                        Target URL: {}", 
                        version, self.proxy_port, self.target_url
                    );
                } else {
                    info!("Failed to parse Cargo.toml");
                }
            }
            Err(_) => {
                info!("Failed to read Cargo.toml");
            }
        }
    }
}

impl Default for AppConfig {
    /// Provide sensible defaults for all fields
    /// This allows the app to run with minimal configuration
    fn default() -> Self {
        Self {
            target_url: String::new(), // Will be required from env/cli/toml
            proxy_port: 3000,
            database_path: "./endpoint-logs.db".to_string(),
            verbose: false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn test_validate_port_valid() {
        let config = AppConfig {
            target_url: "http://example.com".to_string(),
            proxy_port: 3000,
            database_path: "./test.db".to_string(),
            verbose: false,
        };
        assert!(config.validate_port().is_ok());
    }

    // Note: u16 type already prevents values > 65535, so no need to test that case

    #[test]
    fn test_validate_port_invalid_zero() {
        let config = AppConfig {
            target_url: "http://example.com".to_string(),
            proxy_port: 0,
            database_path: "./test.db".to_string(),
            verbose: false,
        };
        assert!(config.validate_port().is_err());
    }

    #[test]
    fn test_validate_url_valid_http() {
        let config = AppConfig {
            target_url: "http://example.com".to_string(),
            proxy_port: 3000,
            database_path: "./test.db".to_string(),
            verbose: false,
        };
        assert!(config.validate_url().is_ok());
    }

    #[test]
    fn test_validate_url_valid_https() {
        let config = AppConfig {
            target_url: "https://example.com".to_string(),
            proxy_port: 3000,
            database_path: "./test.db".to_string(),
            verbose: false,
        };
        assert!(config.validate_url().is_ok());
    }

    #[test]
    fn test_validate_url_invalid_no_scheme() {
        let config = AppConfig {
            target_url: "example.com".to_string(),
            proxy_port: 3000,
            database_path: "./test.db".to_string(),
            verbose: false,
        };
        assert!(config.validate_url().is_err());
    }

    #[test]
    fn test_validate_url_invalid_wrong_scheme() {
        let config = AppConfig {
            target_url: "ftp://example.com".to_string(),
            proxy_port: 3000,
            database_path: "./test.db".to_string(),
            verbose: false,
        };
        assert!(config.validate_url().is_err());
    }

    #[test]
    fn test_from_env_with_valid_env() {
        // Set environment variables
        unsafe {
            std::env::set_var("TARGET_URL", "http://localhost:8080");
            std::env::set_var("PORT", "5000");
            std::env::set_var("DATABASE_PATH", "./custom.db");
        }

        let config = AppConfig::from_env().expect("Config should load from env");

        assert_eq!(config.target_url, "http://localhost:8080");
        assert_eq!(config.proxy_port, 5000);
        assert_eq!(config.database_path, "./custom.db");

        // Clean up
        unsafe {
            std::env::remove_var("TARGET_URL");
            std::env::remove_var("PORT");
            std::env::remove_var("DATABASE_PATH");
        }
    }

    #[test]
    fn test_from_env_with_defaults() {
        // Clean environment first
        unsafe {
            std::env::remove_var("PORT");
            std::env::remove_var("DATABASE_PATH");
            std::env::set_var("TARGET_URL", "http://localhost:8080");
        }

        let config = AppConfig::from_env().expect("Config should load with defaults");

        assert_eq!(config.target_url, "http://localhost:8080");
        assert_eq!(config.proxy_port, 3000); // default
        assert_eq!(config.database_path, "./endpoint-logs.db"); // default

        // Clean up
        unsafe {
            std::env::remove_var("TARGET_URL");
        }
    }

    #[test]
    fn test_from_env_missing_target_url() {
        // Remove TARGET_URL to test error
        unsafe {
            std::env::remove_var("TARGET_URL");
        }

        let result = AppConfig::from_env();
        assert!(result.is_err());
    }

    #[test]
    fn test_from_env_invalid_url() {
        unsafe {
            std::env::set_var("TARGET_URL", "not-a-valid-url");
        }

        let result = AppConfig::from_env();
        assert!(result.is_err());

        // Clean up
        unsafe {
            std::env::remove_var("TARGET_URL");
        }
    }

    #[test]
    fn test_from_env_invalid_port() {
        unsafe {
            std::env::set_var("TARGET_URL", "http://localhost:8080");
            std::env::set_var("PORT", "70000"); // Invalid port
        }

        let result = AppConfig::from_env();
        assert!(result.is_err());

        // Clean up
        unsafe {
            std::env::remove_var("TARGET_URL");
            std::env::remove_var("PORT");
        }
    }

    #[test]
    fn test_default_config() {
        let config = AppConfig::default();

        assert_eq!(config.target_url, "");
        assert_eq!(config.proxy_port, 3000);
        assert_eq!(config.database_path, "./endpoint-logs.db");
        assert_eq!(config.verbose, false);
    }

    #[test]
    fn test_load_from_toml() {
        // Create a temporary TOML file
        let toml_content = r#"
target_url = "http://toml-config:9000"
proxy_port = 4000
database_path = "./toml-test.db"
"#;

        let test_file = "test-config.toml";
        let mut file = fs::File::create(test_file).expect("Failed to create test file");
        file.write_all(toml_content.as_bytes()).expect("Failed to write test file");

        let toml_config = AppConfig::load_from_toml(test_file)
            .expect("Should load TOML config");

        assert_eq!(toml_config.target_url, Some("http://toml-config:9000".to_string()));
        assert_eq!(toml_config.proxy_port, Some(4000));
        assert_eq!(toml_config.database_path, Some("./toml-test.db".to_string()));

        // Clean up
        fs::remove_file(test_file).ok();
    }

    #[test]
    fn test_merge_priority() {
        // Create TOML file with base config
        let toml_content = r#"
target_url = "http://toml-target:8080"
proxy_port = 4000
database_path = "./toml.db"
"#;

        let test_file = "test-priority.toml";
        fs::write(test_file, toml_content).expect("Failed to write test file");

        // Clean environment first, then set ENV variables (should override TOML)
        unsafe {
            std::env::remove_var("DATABASE_PATH");
            std::env::set_var("TARGET_URL", "http://env-target:8080");
            std::env::set_var("PORT", "5000");
        }
        // Don't set DATABASE_PATH - should come from TOML

        // Load TOML
        let toml = AppConfig::load_from_toml(test_file).expect("Should load TOML");

        // Start with defaults, merge TOML, then ENV
        let config = AppConfig::default()
            .merge_toml(toml)
            .merge_env()
            .expect("Should merge env");

        // ENV should override TOML for target and port
        assert_eq!(config.target_url, "http://env-target:8080");
        assert_eq!(config.proxy_port, 5000);
        // DATABASE_PATH should come from TOML (no ENV override)
        assert_eq!(config.database_path, "./toml.db");

        // Clean up
        unsafe {
            std::env::remove_var("TARGET_URL");
            std::env::remove_var("PORT");
        }
        fs::remove_file(test_file).ok();
    }

    #[test]
    fn test_print_config_used() {
        let config = AppConfig {
            target_url: "http://test-target.com".to_string(),
            proxy_port: 4000,
            database_path: "./test-db.db".to_string(),
            verbose: true,
        };
        config.print_config_used();
        // Test passes if no panic
    }
}
