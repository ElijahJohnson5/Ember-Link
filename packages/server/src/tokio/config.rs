use envconfig::Envconfig;

use crate::config::Config;

#[derive(Envconfig, Clone)]
pub struct TokioConfig {
    #[envconfig(from = "HOST", default = "127.0.0.1")]
    pub host: String,

    #[envconfig(from = "PORT", default = "8787")]
    pub port: String,

    #[envconfig(nested)]
    pub base_config: Config,
}
