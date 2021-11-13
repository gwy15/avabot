//! 核心模块

use crate::prelude::*;
use crate::Config;

pub fn init(bot: Bot) {
    bot.command("ping", ping_pong::<FriendMessage>)
        .command("ping", ping_pong::<GroupMessage>)
        .command("reload", reload::<FriendMessage>)
        .command("reload", reload::<GroupMessage>);
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
async fn reload<T: Conversation>(msg: T, bot: Bot, config: Data<Config>) -> Result<Option<String>> {
    if !config.is_admin(*msg.sender().as_ref()) {
        return Ok(None);
    }

    info!("reload config");
    match crate::Config::new() {
        Ok(new_config) => {
            bot.bot_data(new_config);
            info!("reload 成功");
            debug!("config = {:?}", config);
            Ok(Some("reload 成功".to_string()))
        }
        Err(e) => {
            error!("reload 失败：{:?}", e);
            Ok(Some(format!("reload 失败：{:?}", e)))
        }
    }
}
