//! 核心模块

use crate::prelude::*;

pub fn init(bot: Bot) {
    bot.handler(ping_pong::<FriendMessage>)
        .handler(ping_pong::<GroupMessage>)
        .handler(reload::<FriendMessage>)
        .handler(reload::<GroupMessage>);
}

/// ping-pong!
async fn ping_pong<T: Conversation>(msg: T, bot: Bot) -> Result<()> {
    if msg.as_message().to_string().trim() == "ping" {
        let resp = msg.reply("pong", &bot).await?;
        sleep(Duration::from_secs(5)).await;
        bot.request(api::recall::Request {
            message_id: resp.message_id,
        })
        .await?;
    }

    Ok(())
}

/// 给管理员加上 reload 信息
async fn reload<T: Conversation>(msg: T, bot: Bot) -> Result<()> {
    if !crate::Config::get().is_admin(*msg.sender().as_ref()) {
        return Ok(());
    }

    if msg.as_message().to_string().trim() == "reload" {
        info!("reload config");
        match crate::Config::refresh() {
            Ok(_) => {
                info!("reload 成功");
                debug!("config = {:?}", crate::Config::get());
                msg.reply("reload 成功", &bot).await?;
            }
            Err(e) => {
                error!("reload 失败：{:?}", e);
                msg.reply(format!("reload 失败：{:?}", e), &bot).await?;
            }
        }
    }
    Ok(())
}
