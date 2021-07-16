#[macro_use]
extern crate log;

mod config;
mod plugins;
pub mod prelude {
    pub use anyhow::*;
    pub use miraie::prelude::*;
    pub use serde::{Deserialize, Serialize};
    pub use std::time::Duration;
    pub use tokio::time::sleep;
}

pub use config::Config;

use prelude::*;

async fn run() -> Result<()> {
    let config = config::Config::get().clone();

    let (bot, con) = miraie::Bot::new(config.addr, config.verify_key, config.qq).await?;
    info!("连接已建立。");

    plugins::core::init(bot.clone());
    plugins::asoul_cnki::init(bot.clone());
    plugins::keyword_reply::init(bot.clone());
    plugins::bilibili_cover::init(bot.clone());

    con.run().await?;
    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    // allow .env not found
    dotenv::dotenv().ok();
    log4rs::init_file("log4rs.yml", Default::default()).context("log4rs 启动失败")?;

    // try boot
    let mut counter = 0;
    loop {
        counter += 1;
        if let Err(e) = run().await {
            warn!("启动失败：{:?}", e);
            info!("等待 {} s.", counter);
            sleep(Duration::from_secs(counter)).await;
        }
        if counter > 10 {
            error!("尝试重启次数过多，停止");
            break;
        }
    }

    Ok(())
}
