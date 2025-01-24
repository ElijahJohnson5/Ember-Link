use envconfig::Envconfig;

#[derive(Envconfig)]
pub struct Config {
    #[envconfig(from = "WEBHOOK_URL")]
    pub webhook_url: Option<String>,
}
