use log::*;

use avabot::prelude::*;
use avabot::{plugins, Config};

async fn run() -> Result<()> {
    let config = Config::new()?;

    let (mut bot, con) = miraie::Bot::new(&config.addr, &config.verify_key, config.qq).await?;
    info!("连接已建立。");

    bot = bot.bot_data(Data::new(config));

    plugins::core::init(bot.clone());
    plugins::asoul_cnki::init(bot.clone());
    plugins::keyword_reply::init(bot.clone());
    plugins::bilibili_cover::init(bot.clone());
    plugins::asoul_weekly::init(bot.clone());
    plugins::schedule::init(bot.clone());
    plugins::shab::init(bot.clone());
    plugins::fraud::init(bot.clone());

    con.run().await?;
    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    // allow .env not found
    dotenv::dotenv().ok();
    log4rs::init_file("log4rs.yml", Default::default()).context("log4rs 初始化失败")?;

    // try boot
    let mut counter = 0;
    loop {
        counter += 1;
        match run().await {
            Err(e) => {
                if counter > 10 {
                    error!("尝试重启次数过多，停止");
                    break;
                }
                warn!("启动失败：{:?}", e);
                info!("等待 {} s.", counter);
                sleep(Duration::from_secs(counter)).await;
            }
            Ok(_) => {
                break;
            }
        }
    }

    Ok(())
}
