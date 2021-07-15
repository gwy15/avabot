//! 关键字回复
use std::collections::HashMap;

use crate::prelude::*;

use rand::prelude::*;
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct KeywordReplyConfig {
    #[serde(default)]
    pub full_match: HashMap<String, String>,

    #[serde(default)]
    pub random: HashMap<String, Vec<String>>,

    #[serde(default)]
    pub contain: HashMap<String, String>,
}

impl KeywordReplyConfig {
    pub fn reply(&self, msg: &str) -> Option<MessageChain> {
        if let Some(xml) = self.full_match.get(msg) {
            return Some(MessageChain::from_xml(xml));
        }
        if let Some(options) = self.random.get(msg) {
            return options
                .choose(&mut thread_rng())
                .map(|xml| MessageChain::from_xml(xml));
        }
        for (k, xml) in self.contain.iter() {
            if msg.contains(k) {
                return Some(MessageChain::from_xml(xml));
            }
        }
        None
    }
}

pub fn init(bot: Bot) {
    bot.handler(on_group_msg);
}

/// 关键字回复，只在群聊中使用
async fn on_group_msg(group_msg: GroupMessage, bot: Bot) -> Result<()> {
    let message = group_msg.message.to_string();
    let reply = crate::config::Config::get().keyword_reply.reply(&message);

    if let Some(reply) = reply {
        group_msg.reply(reply, &bot).await?;
        info!("关键词回复成功");
    }

    Ok(())
}
