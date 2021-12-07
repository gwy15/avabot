use crate::prelude::*;
use crate::Config;

pub fn init(bot: Bot) {
    bot.command("啥b", on_shabi).command("shab", on_shabi);
}

async fn on_shabi(msg: GroupMessage, bot: Bot, config: Data<Config>) -> Result<()> {
    if !config.is_admin(msg.sender.id) {
        return Ok(());
    }

    msg.reply(MessageChain::from_xml("<v> shab.silk </v>"), &bot)
        .await?;

    Ok(())
}
