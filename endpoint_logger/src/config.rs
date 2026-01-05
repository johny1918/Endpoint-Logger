use std::env;


#[derive(Clone, Debug)]
pub struct AppConfig {
    pub database_url: String,
    pub port: u16
}

impl AppConfig {
    pub fn from_env() -> Self {
        Self { 
            database_url: env::var("DATABASE_URL").expect("DATABSE_URL is required"),
            port: env::var("PORT").unwrap_or_else(|_| "3000".into()).parse().unwrap() 
        }
    }
}