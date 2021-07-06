#[macro_use]
extern crate log;

use anyhow::*;
use miraie::{
    api,
    messages::{FriendMessage, Message, MessageChain},
    App, Bot,
};
use std::time::Duration;
use tokio::time::sleep;

mod config;
mod keyword_reply;

/// ping-pong!
async fn ping_pong(msg: Message, bot: Bot) -> Result<()> {
    let message_id = match msg {
        Message::Group(msg) => {
            if msg.message.to_string() != "ping" {
                return Ok(());
            }
            msg.quote_reply(MessageChain::new().text("pong"), &bot)
                .await?
                .message_id
        }
        Message::Friend(msg) => {
            if msg.message.to_string() != "ping" {
                return Ok(());
            }
            msg.reply(MessageChain::new().text("pong"), &bot)
                .await?
                .message_id
        }
        _ => return Ok(()),
    };

    sleep(Duration::from_secs(5)).await;
    bot.request(api::recall::Request { message_id }).await?;

    Ok(())
}

/// 给管理员加上 reload 信息
async fn reload(msg: FriendMessage, bot: Bot) -> Result<()> {
    if !config::Config::get().is_admin(msg.sender.id) {
        return Ok(());
    }

    if msg.message.to_string() != "reload" {
        return Ok(());
    }

    info!("reload config");

    match config::Config::refresh() {
        Ok(_) => {
            info!("reload 成功");
            debug!("config = {:?}", config::Config::get());
            msg.reply(MessageChain::new().text("reload 成功"), &bot)
                .await?;
        }
        Err(e) => {
            error!("reload 失败：{:?}", e);
            msg.reply(
                MessageChain::new().text(format!("reload 失败：{:?}", e)),
                &bot,
            )
            .await?;
        }
    }
    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    dotenv::dotenv()?;
    log4rs::init_file("log4rs.yml", Default::default())?;

    let config = config::Config::get().clone();

    let (bot, con) = miraie::Bot::new(config.addr, config.verify_key, config.qq).await?;
    info!("bot connected.");

    bot.handler(ping_pong)
        .handler(reload)
        .handler(keyword_reply::on_group_msg);

    con.run().await?;
    Ok(())
}
