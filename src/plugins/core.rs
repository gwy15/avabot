//! 核心模块

use crate::prelude::*;

pub fn init(bot: Bot) {
    bot.handler(ping_pong).handler(reload);
}

/// ping-pong!
async fn ping_pong(msg: Message, bot: Bot) -> Result<()> {
    let message_id = match msg {
        Message::Group(msg) => {
            if msg.message.to_string() != "ping" {
                return Ok(());
            }
            msg.reply("pong", &bot).await?.message_id
        }
        Message::Friend(msg) => {
            if msg.message.to_string() != "ping" {
                return Ok(());
            }
            msg.reply("pong", &bot).await?.message_id
        }
        _ => return Ok(()),
    };

    sleep(Duration::from_secs(5)).await;
    bot.request(api::recall::Request { message_id }).await?;

    Ok(())
}

/// 给管理员加上 reload 信息
async fn reload(msg: FriendMessage, bot: Bot) -> Result<()> {
    if !crate::Config::get().is_admin(msg.sender.id) {
        return Ok(());
    }

    if msg.message.to_string() != "reload" {
        return Ok(());
    }

    info!("reload config");

    match crate::Config::refresh() {
        Ok(_) => {
            info!("reload 成功");
            debug!("config = {:?}", crate::Config::get());
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
