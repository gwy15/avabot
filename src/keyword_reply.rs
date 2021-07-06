use anyhow::*;
use miraie::{
    messages::{MessageBlock, MessageChain},
    prelude::*,
};
use regex::Regex;
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct ReplyRule {
    pub r#match: String,
    pub mode: MatchMode,
    pub reply: Vec<ReplyBlock>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MatchMode {
    Full,
    Regex,
}

/// 都是路径
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ReplyBlock {
    Text(String),
    Voice(String),
    Image(String),
}

impl ReplyRule {
    pub fn matches(&self, msg: &str) -> bool {
        match self.mode {
            MatchMode::Full => self.r#match == msg,
            MatchMode::Regex => Regex::new(&self.r#match).unwrap().is_match(&msg),
        }
    }

    pub fn reply(&self) -> MessageChain {
        let blocks = self
            .reply
            .iter()
            .map(|r| match r {
                ReplyBlock::Text(s) => MessageBlock::text(s),
                ReplyBlock::Image(path) => MessageBlock::image_path(path),
                ReplyBlock::Voice(path) => MessageBlock::voice_path(path),
            })
            .collect();
        MessageChain(blocks)
    }
}

/// 关键字回复
pub async fn on_group_msg(group_msg: GroupMessage, bot: Bot) -> Result<()> {
    let reply_message = {
        let config = crate::config::Config::get();
        let msg = group_msg.message.to_string();
        let rule = config
            .keyword_reply
            .iter()
            .filter(|item| item.matches(&msg))
            .next()
            .cloned();
        match rule {
            None => return Ok(()),
            Some(rule) => {
                info!("关键词匹配，触发回复");
                rule.reply()
            }
        }
    };

    group_msg.reply(reply_message, &bot).await?;
    info!("关键词回复成功");
    Ok(())
}
