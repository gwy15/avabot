#[macro_use]
extern crate log;

mod config;
mod plugins;
pub mod prelude {
    pub use anyhow::*;
    pub use miraie::prelude::*;
    pub use std::time::Duration;
    pub use tokio::time::sleep;
}

pub use config::Config;

use prelude::*;

#[tokio::main]
async fn main() -> Result<()> {
    dotenv::dotenv()?;
    log4rs::init_file("log4rs.yml", Default::default())?;

    let config = config::Config::get().clone();

    let (bot, con) = miraie::Bot::new(config.addr, config.verify_key, config.qq).await?;
    info!("连接已建立。");

    plugins::core::init(bot.clone());
    plugins::keyword_reply::init(bot);

    con.run().await?;
    Ok(())
}
