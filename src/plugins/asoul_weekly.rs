//! A-SOUL周报相关插件

use std::collections::{BTreeMap, HashSet};
use std::fmt::Write;

use chrono::{Duration, Utc};

use crate::prelude::*;

pub fn init(bot: Bot) {
    bot.handler(on_message);
}

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    url: String,
    allow_groups: HashSet<QQ>,
}

async fn on_message(msg: GroupMessage, bot: Bot) -> Result<()> {
    let url = {
        let config = crate::Config::get();
        if !config
            .asoul_weekly
            .allow_groups
            .contains(&msg.sender.group.id)
        {
            return Ok(());
        }
        let t = match msg.as_message().to_string().as_str() {
            "今日归档" | "今天归档" | "归档" => Utc::now(),
            "昨天归档" | "昨日归档" => Utc::now() - Duration::days(1),
            "前天归档" => Utc::now() - Duration::days(2),
            _ => return Ok(()),
        };
        let t = t.to_rfc3339_opts(chrono::SecondsFormat::Secs, true);
        info!("t = {}", t);
        format!("{}?t={}", config.asoul_weekly.url, t)
    };
    info!("归档链接: {}", url);
    let summary: BTreeMap<String, Vec<String>> = reqwest::get(url).await?.json().await?;
    let mut message = String::new();
    for (category, ids) in summary {
        writeln!(&mut message, "------ {} -----", category)?;
        if category == "动态" {
            for id in ids {
                message += &id;
                message += "\n";
            }
        } else {
            for id in ids.chunks(2) {
                message += &id.join("  ");
                message += "\n";
            }
        }
    }
    msg.reply_unquote(message, &bot).await?;

    Ok(())
}
