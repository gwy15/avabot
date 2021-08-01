//! 关键字回复
use std::collections::HashMap;

use crate::prelude::*;

use rand::prelude::*;
use serde::Deserialize;

fn default_max_alias_times() -> u32 {
    3
}

#[derive(Debug, Clone, Deserialize)]
pub struct KeywordReplyConfig {
    /// 全文匹配
    #[serde(default)]
    pub full_match: HashMap<String, String>,

    /// 随机选取一个
    #[serde(default)]
    pub random: HashMap<String, Vec<String>>,

    /// 包含关键词
    #[serde(default)]
    pub contain: HashMap<String, String>,

    /// 别名
    #[serde(default)]
    pub alias: HashMap<String, String>,

    /// 最大重命名次数，全文匹配
    #[serde(default = "default_max_alias_times")]
    max_alias_times: u32,
}

impl KeywordReplyConfig {
    pub fn reply(&self, msg: &str) -> Option<MessageChain> {
        self.reply_impl(msg, 0)
    }

    fn reply_impl(&self, msg: &str, depth: u32) -> Option<MessageChain> {
        if depth > self.max_alias_times {
            return None;
        }
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
        if let Some(alias) = self.alias.get(msg) {
            return self.reply_impl(alias, depth + 1);
        }
        None
    }
}

pub fn init(bot: Bot) {
    bot.handler(on_msg::<GroupMessage>)
        .handler(on_msg::<FriendMessage>);
}

/// 关键字回复
async fn on_msg<T: Conversation>(msg: T, bot: Bot) -> Result<()> {
    let message = msg.as_message().to_string();
    let reply = crate::config::Config::get().keyword_reply.reply(&message);

    if let Some(reply) = reply {
        msg.reply(reply, &bot).await?;
        info!("关键词回复成功");
    }

    Ok(())
}

#[test]
fn test_alias() {
    let cfg: KeywordReplyConfig = serde_yaml::from_str(
        r"
full_match: 
    a: 1
random:
    b:
        - <i> a.jpg </i>
        - 2
alias:
    c: a
    d: c
    ",
    )
    .unwrap();
    assert_eq!(cfg.reply("c").unwrap(), MessageBlock::text("1").into());
    assert_eq!(cfg.reply("d").unwrap(), MessageBlock::text("1").into());
}
