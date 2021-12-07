use crate::prelude::*;
use crate::Config;

pub fn init(bot: Bot) {
    bot.handler(on_shabi);
}

async fn on_shabi(msg: GroupMessage, bot: Bot, config: Data<Config>) -> Result<()> {
    let message = &msg.message;
    // get source
    let source = message
        .0
        .iter()
        .filter_map(|b| match b {
            MessageBlock::Quote { sender_id, .. } => Some(*sender_id),
            _ => None,
        })
        .next();
    let source = match source {
        Some(raw) => raw,
        None => {
            trace!("该条消息不是引用回复消息");
            return Ok(());
        }
    };
    if config.is_admin(source) {
        debug!("不准骂我");
        return Ok(());
    }

    // get text
    let s = message
        .0
        .iter()
        .filter_map(|s| match s {
            MessageBlock::Text { text } => Some(text.to_string()),
            _ => None,
        })
        .collect::<Vec<_>>()
        .join("")
        .trim()
        .to_lowercase();

    if matches!(s.as_str(), "啥b" | "shabi" | "shab") {
        debug!("准备");
        msg.reply_unquote(MessageChain::new().at(source), &bot)
            .await?;

        msg.reply_unquote(MessageChain::from_xml("<v> shab.silk </v>"), &bot)
            .await?;
    }

    Ok(())
}
