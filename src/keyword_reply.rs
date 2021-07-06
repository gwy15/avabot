use anyhow::*;
use miraie::{
    messages::{MessageBlock, MessageChain},
    prelude::*,
};
use rand::prelude::*;
use regex::Regex;
use serde::Deserialize;

use crate::config;

#[derive(Debug, Clone, Deserialize)]
pub struct ReplyRule {
    pub r#match: String,
    #[serde(default)]
    pub mode: MatchMode,
    pub reply: Reply,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MatchMode {
    Full,
    Regex,
}
impl Default for MatchMode {
    fn default() -> Self {
        MatchMode::Full
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
pub enum Reply {
    #[serde(rename = "random")]
    Random(String),
    Message(Vec<ReplyBlock>),
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
        match &self.reply {
            Reply::Random(_) => {
                let config = config::Config::get();
                let msg_replies = config
                    .keyword_reply
                    .iter()
                    .filter_map(|r| match &r.reply {
                        Reply::Message(m) => Some(m),
                        Reply::Random(_) => None,
                    })
                    .collect::<Vec<_>>();

                msg_replies
                    .choose(&mut thread_rng())
                    .map(|blocks| Self::reply_blocks_to_chain(blocks))
                    .unwrap_or_else(|| MessageChain::new().text("还没有配置呢"))
            }
            Reply::Message(m) => Self::reply_blocks_to_chain(&m),
        }
    }

    fn reply_blocks_to_chain(blocks: &[ReplyBlock]) -> MessageChain {
        MessageChain(
            blocks
                .iter()
                .map(|r| match r {
                    ReplyBlock::Text(s) => MessageBlock::text(s),
                    ReplyBlock::Image(path) => MessageBlock::image_path(path),
                    ReplyBlock::Voice(path) => MessageBlock::voice_path(path),
                })
                .collect(),
        )
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
