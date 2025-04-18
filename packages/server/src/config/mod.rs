use envconfig::Envconfig;

#[derive(Envconfig, Clone, Debug)]
pub struct Config {
    #[cfg(feature = "webhook")]
    #[envconfig(from = "WEBHOOK_URL")]
    pub webhook_url: String,

    // Used for signing webhook payloads
    #[cfg(feature = "webhook")]
    #[envconfig(from = "WEBHOOK_SECRET_KEY")]
    pub webhook_secret_key: String,

    #[envconfig(from = "ALLOW_UNAUTHORIZED", default = "false")]
    pub allow_unauthorized: bool,

    #[envconfig(from = "JWT_SIGNER_KEY")]
    pub jwt_signer_key: Option<String>,

    // Used for multi tenant operation, will be called with a tenant id that the participant connected with
    #[cfg(feature = "multi-tenant")]
    #[envconfig(from = "JWT_SIGNER_KEY_ENDPOINT")]
    pub jwt_signer_key_endpoint: Option<String>,

    #[envconfig(from = "STORAGE_ENDPOINT")]
    pub storage_endpoint: Option<String>,
}

pub fn config_keys() -> Vec<&'static str> {
    vec![
        "WEBHOOK_URL",
        "WEBHOOK_SECRET_KEY",
        "ALLOW_UNAUTHORIZED",
        "JWT_SIGNER_KEY",
        "JWT_SIGNER_KEY_ENDPOINT",
        "STORAGE_ENDPOINT",
    ]
}
