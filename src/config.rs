use anyhow::Result;
use miraie::bot::QQ;
use serde::Deserialize;
use std::collections::HashSet;

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    pub qq: QQ,
    pub verify_key: String,
    pub addr: String,

    /// path for sled db
    pub db_path: String,

    pub admins: HashSet<QQ>,

    pub keyword_reply: crate::plugins::keyword_reply::KeywordReplyConfig,

    pub asoul_weekly: crate::plugins::asoul_weekly::Config,
}

impl Config {
    /// 从 config.yaml 读
    pub fn new() -> Result<Self> {
        let reader = std::fs::File::open("config.yaml")?;
        let config = serde_yaml::from_reader(reader)?;
        Ok(config)
    }

    pub fn is_admin(&self, qq: QQ) -> bool {
        self.admins.contains(&qq)
    }
}
