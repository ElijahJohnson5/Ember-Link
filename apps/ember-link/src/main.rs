use server::tokio::main as server_main;

use std::error::Error as StdError;

pub type BoxDynError = Box<dyn StdError + 'static + Send + Sync>;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    server_main().await
}
