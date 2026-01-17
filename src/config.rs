use std::env;

#[derive(Debug, Clone)]
pub struct OAuth2Config {
    pub enabled: bool,
    pub issuer: String,
    pub audience: Vec<String>,
    pub required_scope: Option<String>,
    pub secret_key: String,
}

impl Default for OAuth2Config {
    fn default() -> Self {
        Self {
            enabled: false,
            issuer: String::new(),
            audience: Vec::new(),
            required_scope: None,
            secret_key: String::new(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct TlsConfig {
    pub enabled: bool,
    pub cert_path: Option<String>,
    pub key_path: Option<String>,
}

#[derive(Debug, Clone)]
pub struct Config {
    pub host: String,
    pub port: u16,
    pub mongodb_uri: String,
    pub nrf_uri: Option<String>,
    pub nf_instance_id: String,
    pub scp_host: String,
    pub oauth2: OAuth2Config,
    pub tls: TlsConfig,
    pub cache_ttl_seconds: u64,
}

impl Config {
    pub fn from_env() -> anyhow::Result<Self> {
        let host = env::var("SCP_HOST")
            .unwrap_or_else(|_| "0.0.0.0".to_string());

        let port = env::var("SCP_PORT")
            .unwrap_or_else(|_| "7777".to_string())
            .parse()?;

        let mongodb_uri = env::var("MONGODB_URI")
            .unwrap_or_else(|_| "mongodb://localhost:27017".to_string());

        let nrf_uri = env::var("NRF_URI").ok();

        let nf_instance_id = env::var("NF_INSTANCE_ID")
            .unwrap_or_else(|_| uuid::Uuid::new_v4().to_string());

        let scp_host = env::var("SCP_ADVERTISED_HOST")
            .unwrap_or_else(|_| "127.0.0.1".to_string());

        let cache_ttl_seconds = env::var("CACHE_TTL_SECONDS")
            .unwrap_or_else(|_| "300".to_string())
            .parse()?;

        let oauth2_enabled = env::var("OAUTH2_ENABLED")
            .unwrap_or_else(|_| "false".to_string())
            .parse()
            .unwrap_or(false);

        let oauth2_issuer = env::var("OAUTH2_ISSUER")
            .unwrap_or_else(|_| "".to_string());

        let oauth2_audience = env::var("OAUTH2_AUDIENCE")
            .unwrap_or_else(|_| "".to_string())
            .split(',')
            .filter(|s| !s.is_empty())
            .map(|s| s.to_string())
            .collect();

        let oauth2_required_scope = env::var("OAUTH2_REQUIRED_SCOPE").ok();

        let jwt_secret = env::var("JWT_SECRET")
            .unwrap_or_else(|_| "".to_string());

        let oauth2 = OAuth2Config {
            enabled: oauth2_enabled,
            issuer: oauth2_issuer,
            audience: oauth2_audience,
            required_scope: oauth2_required_scope,
            secret_key: jwt_secret,
        };

        let tls_enabled = env::var("TLS_ENABLED")
            .unwrap_or_else(|_| "false".to_string())
            .parse()
            .unwrap_or(false);

        let tls_cert_path = env::var("TLS_CERT_PATH").ok();
        let tls_key_path = env::var("TLS_KEY_PATH").ok();

        let tls = TlsConfig {
            enabled: tls_enabled,
            cert_path: tls_cert_path,
            key_path: tls_key_path,
        };

        Ok(Self {
            host,
            port,
            mongodb_uri,
            nrf_uri,
            nf_instance_id,
            scp_host,
            oauth2,
            tls,
            cache_ttl_seconds,
        })
    }
}
