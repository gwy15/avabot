use anyhow::Result;
use miraie::bot::QQ;
use parking_lot::RwLock;
use serde::Deserialize;
use std::{collections::HashSet, net::SocketAddr};

use crate::keyword_reply::ReplyRule;

lazy_static::lazy_static! {
    static ref CONFIG: RwLock<Config> = RwLock::new(Config::new().expect("Failed to parse config at init."));
}

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    pub qq: QQ,
    pub verify_key: String,
    pub addr: SocketAddr,

    pub admins: HashSet<QQ>,

    pub keyword_reply: Vec<ReplyRule>,
}

impl Config {
    /// 从 config.yaml 读
    pub fn new() -> Result<Self> {
        let reader = std::fs::File::open("config.yaml")?;
        let config = serde_yaml::from_reader(reader)?;
        Ok(config)
    }

    pub fn get<'a>() -> parking_lot::RwLockReadGuard<'a, Self> {
        CONFIG.read()
    }

    pub fn refresh() -> Result<()> {
        let this = Self::new()?;
        *CONFIG.write() = this;
        Ok(())
    }

    pub fn is_admin(&self, qq: QQ) -> bool {
        self.admins.contains(&qq)
    }
}
