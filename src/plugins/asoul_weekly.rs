//! A-SOUL周报相关插件

use std::collections::{BTreeMap, HashSet};
use std::fmt::Write;

use chrono::{Duration, Utc};
use regex::Regex;
use serde_json::{json, Value};

use crate::prelude::*;

pub fn init(bot: Bot) {
    bot.handler(on_message);
}

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    url: String,
    allow_groups: HashSet<QQ>,
}

async fn generate_summary(msg: GroupMessage, bot: Bot, base_url: String) -> Result<()> {
    let t = match msg.as_message().to_string().as_str() {
        "今日归档" | "今天归档" | "归档" => Utc::now(),
        "昨天归档" | "昨日归档" => Utc::now() - Duration::days(1),
        "前天归档" => Utc::now() - Duration::days(2),
        _ => return Ok(()),
    };
    let t = t.to_rfc3339_opts(chrono::SecondsFormat::Secs, true);
    info!("t = {}", t);
    let url = format!("{}?t={}", base_url, t);

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

async fn change_category(msg: GroupMessage, base_url: String) -> Result<()> {
    lazy_static::lazy_static! {
        static ref PATTERN: Regex = Regex::new(r"^修改分类 (BV.+) (.+)$").unwrap();
    }
    let msg_s = msg.as_message().to_string();
    match PATTERN.captures(&msg_s) {
        Some(cap) => {
            let bv = cap.get(1).ok_or_else(|| anyhow!("缺少"))?.as_str();
            let cat = cap.get(2).ok_or_else(|| anyhow!("缺少分类"))?.as_str();
            let url = format!("{}/items/{}/category", base_url, bv);
            let r: Value = reqwest::Client::new()
                .patch(url)
                .json(&json!({ "category": cat }))
                .send()
                .await?
                .json()
                .await?;
            if let Some(r) = r.get("error").cloned() {
                if let Some(s) = r.as_str() {
                    bail!(s.to_string())
                }
            }
        }
        None => {
            bail!("格式错误，格式需要是：\n修改分类 BVxxxxxxxx 翻唱")
        }
    }
    Ok(())
}

async fn on_message(msg: GroupMessage, bot: Bot) -> Result<()> {
    let base_url = {
        let config = crate::Config::get();
        if !config
            .asoul_weekly
            .allow_groups
            .contains(&msg.sender.group.id)
        {
            return Ok(());
        }
        config.asoul_weekly.url.clone()
    };

    let msg_s = msg.as_message().to_string();
    if msg_s.contains("归档") {
        generate_summary(msg, bot, base_url).await?;
    } else if msg_s.starts_with("修改分类") {
        match change_category(msg.clone(), base_url).await {
            Ok(()) => {
                msg.reply("修改分类成功", &bot).await?;
            }
            Err(e) => {
                msg.reply(e.to_string(), &bot).await?;
            }
        }
    }

    Ok(())
}
