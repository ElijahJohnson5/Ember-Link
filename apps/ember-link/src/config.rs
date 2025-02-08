use envconfig::Envconfig;

#[derive(Envconfig, Clone)]
pub struct Config {
    #[envconfig(from = "WEBHOOK_URL")]
    pub webhook_url: Option<String>,

    #[envconfig(from = "ALLOW_UNAUTHORIZED", default = "false")]
    pub allow_unauthorized: bool,

    #[envconfig(from = "JWT_SIGNER_KEY")]
    pub jwt_signer_key: Option<String>,

    // Used for multi tenant operation, will be called with a tenant id that the participant connected with
    #[envconfig(from = "JWT_SIGNER_KEY_ENDPOINT")]
    pub jwt_signer_key_endpoint: Option<String>,
}
