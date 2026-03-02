use envconfig::{Envconfig, Error};

#[derive(Debug, Envconfig)]
pub struct Cors {
    #[envconfig(from = "CORS_ALLOWED_ORIGINS")]
    pub allowed_origins: Option<String>,
}

#[derive(Debug, Envconfig)]
pub struct Http {
    #[envconfig(from = "HTTP_ADDRESS", default = "127.0.0.1:3000")]
    pub address: String,
}

#[derive(Debug, Envconfig)]
pub struct OpenApi {
    #[envconfig(nested)]
    pub info: OpenApiInfo,
    #[envconfig(from = "OPENAPI_SERVER_URLS", default = "/")]
    pub server_urls: String,
}

#[derive(Debug, Envconfig)]
pub struct OpenApiInfo {
    #[envconfig(from = "OPENAPI_INFO_TITLE", default = "")]
    pub title: String,
    #[envconfig(from = "OPENAPI_INFO_VERSION", default = "")]
    pub version: String,
    #[envconfig(from = "OPENAPI_INFO_DESCRIPTION")]
    pub description: Option<String>,
}

#[derive(Debug, Envconfig)]
pub struct Config {
    #[envconfig(nested)]
    pub cors: Cors,
    #[envconfig(nested)]
    pub http: Http,
    #[envconfig(nested)]
    pub openapi: OpenApi,
}

impl Config {
    pub fn init() -> Result<Self, Error> {
        Config::init_from_env()
    }
}
