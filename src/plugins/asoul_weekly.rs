//! A-SOUL周报相关插件

use std::collections::{BTreeMap, HashSet};
use std::fmt::Write;

use chrono::{Duration, Utc};
use lazy_static::lazy_static;
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

/// 归档，会在内部回复消息
async fn generate_summary(msg: GroupMessage, bot: Bot, base_url: &str) -> Result<()> {
    let t = match msg.as_message().to_string().as_str() {
        "今日归档" | "今天归档" | "归档" => Utc::now(),
        "昨天归档" | "昨日归档" => Utc::now() - Duration::days(1),
        "前天归档" => Utc::now() - Duration::days(2),
        _ => return Ok(()),
    };
    let t = t.to_rfc3339_opts(chrono::SecondsFormat::Secs, true);
    info!("t = {}", t);
    let url = format!("{}/summary?t={}", base_url, t);

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

/// kpi，内部回复消息
async fn generate_kpi(msg: GroupMessage, bot: Bot, base_url: &str) -> Result<()> {
    let t = match msg.as_message().to_string().to_lowercase().as_str() {
        "今日kpi" | "今天kpi" | "kpi" => Utc::now(),
        "昨天kpi" | "昨日kpi" => Utc::now() - Duration::days(1),
        "前天kpi" => Utc::now() - Duration::days(2),
        _ => return Ok(()),
    };
    let t = t.to_rfc3339_opts(chrono::SecondsFormat::Secs, true);
    info!("t = {}", t);
    let url = format!("{}/kpi?t={}", base_url, t);

    info!("kpi链接: {}", url);
    #[derive(Debug, Deserialize)]
    struct Row {
        name: String,
        times: u32,
    }
    let kpi: Vec<Row> = reqwest::get(url).await?.json().await?;
    let mut message = String::new();
    for row in kpi {
        writeln!(message, "【{}】筛选了 【{}】 个", row.name, row.times)?;
    }
    msg.reply_unquote(message, &bot).await?;
    Ok(())
}

/// 修改分类
async fn change_category(msg: GroupMessage, base_url: &str) -> Result<()> {
    lazy_static::lazy_static! {
        static ref PATTERN: Regex = Regex::new(r"^修改分类\s+(\w+)\s+([^\s]+)$").unwrap();
    }
    let msg_s = msg.as_message().to_string();
    match PATTERN.captures(&msg_s) {
        Some(cap) => {
            let id = cap.get(1).ok_or_else(|| anyhow!("缺少"))?.as_str();
            let category = cap.get(2).ok_or_else(|| anyhow!("缺少分类"))?.as_str();

            let url = format!("{}/items/{}/category", base_url, id);
            let client = reqwest::Client::new();
            // 如果分类是“删除”或者null就删掉
            let req = match category.to_lowercase().as_str() {
                cat if cat.contains("删除") || cat.contains('-') => {
                    client.delete(url).send().await
                }
                cat if cat.starts_with('+') => {
                    let cat = cat.trim_start_matches('+');
                    client
                        .post(url)
                        .json(&json!({ "category": cat }))
                        .send()
                        .await
                }
                cat => {
                    client
                        .patch(url)
                        .json(&json!({ "category": cat }))
                        .send()
                        .await
                }
            };
            let rsp: Value = req?.json().await?;
            if let Some(r) = rsp.get("error").cloned() {
                if let Some(s) = r.as_str() {
                    bail!(s.to_string())
                }
            }
        }
        None => {
            bail!("格式错误，格式需要是：\n修改分类 BVxxxxxxxx 翻唱/删除")
        }
    }
    Ok(())
}

/// 从 b23 短链或者 t.bilibili.com 长链解析出动态 id
async fn get_redirected_id(url: &str) -> Result<String> {
    lazy_static! {
        static ref ID_REGEXP: Regex =
            Regex::new(r"https://((t\.bilibili\.com)|(m\.bilibili\.com/dynamic))/(?P<did>\d+)")
                .unwrap();
    }
    if let Some(cap) = ID_REGEXP.captures(url) {
        info!("匹配到原始链接中的 id，不需要重定向");
        if let Some(id) = cap.name("did").map(|s| s.as_str()) {
            return Ok(id.to_string());
        }
    }
    if !url.starts_with("https://b23.tv/") {
        bail!("链接不识别，应该是 b23.tv 短链或者 t.bilibili.com 长链");
    }
    info!("进行重定向，url = {}", url);
    let client = reqwest::ClientBuilder::new()
        .redirect(reqwest::redirect::Policy::none())
        .build()?;
    let response = client.get(url).send().await?;
    let location = match response.headers().get("location") {
        Some(header) => header.to_str()?,
        None => {
            warn!("重定向失败：response = {:?}", response);
            bail!("短链重定向失败");
        }
    };
    info!("location = {:?}", location);
    if let Some(cap) = ID_REGEXP.captures(location) {
        info!("在重定向的连接中匹配到 id");
        if let Some(id) = cap.name("did").map(|s| s.as_str()) {
            return Ok(id.to_string());
        }
    }
    bail!("未在重定向后的链接内解析出动态 id");
}

/// 判断是否是简写增加动态并返回提取的链接
fn extract_shortcut_change_category_url(s: &str) -> Option<&str> {
    lazy_static! {
        static ref URL_REGEXP: Regex =
            Regex::new(r"https://((b23\.tv|.+\.bilibili.com)/\w+)").unwrap();
    }
    if !(s.ends_with('+') || s.ends_with('-')) {
        return None;
    }
    if let Some(cap) = URL_REGEXP.captures(s) {
        return cap.get(0).map(|m| m.as_str());
    }
    None
}

/// 缩写的实现，返回（是否是增加，回复的消息）
async fn change_category_shortcut(
    msg_s: &str,
    base_url: &str,
    url: &str,
) -> Result<(bool, String)> {
    let id = get_redirected_id(url).await?;
    let url = format!("{}/items/{}/category", base_url, id);

    let client = reqwest::Client::new();
    let (add, req) = if msg_s.ends_with('+') {
        (true, client.post(url).json(&json!({ "category": "动态" })))
    } else {
        (false, client.delete(url))
    };
    let response = req.send().await?;
    if response.status() != reqwest::StatusCode::OK {
        let json: serde_json::Value = response.json().await?;
        info!("请求返回 json：{:?}", json);
        if let Some(r) = json.get("error").cloned() {
            if let Some(s) = r.as_str() {
                bail!(s.to_string())
            }
        }
        bail!("请求错误：{:?}", json);
    }

    Ok((add, id))
}

async fn on_message(msg: GroupMessage, config: Data<crate::Config>, bot: Bot) -> Result<()> {
    if !config
        .asoul_weekly
        .allow_groups
        .contains(&msg.sender.group.id)
    {
        return Ok(());
    }

    let base_url = &config.asoul_weekly.url;

    let msg_s = msg.as_message().to_string();
    if msg_s.contains("归档") {
        generate_summary(msg, bot, base_url).await?;
    } else if msg_s.to_lowercase().contains("kpi") {
        generate_kpi(msg, bot, base_url).await?;
    } else if msg_s.starts_with("修改分类") {
        match change_category(msg.clone(), base_url).await {
            Ok(()) => {
                msg.reply("修改分类成功", &bot).await?;
            }
            Err(e) => {
                msg.reply(e.to_string(), &bot).await?;
            }
        }
    } else if let Some(url) = extract_shortcut_change_category_url(&msg_s) {
        info!("简写增加，url = {}", url);
        match change_category_shortcut(&msg_s, base_url, url).await {
            Ok((true, id)) => {
                msg.reply(format!("增加动态 id={} 成功", id), &bot).await?;
            }
            Ok((false, id)) => {
                msg.reply(format!("删除动态 id={} 成功", id), &bot).await?;
            }
            Err(e) => {
                msg.reply(e.to_string(), &bot).await?;
            }
        }
    }

    Ok(())
}

#[test]
fn test_is_shortcut_change_category() {
    assert_eq!(
        extract_shortcut_change_category_url("https://b23.tv/oNcAbk +"),
        Some("https://b23.tv/oNcAbk")
    );
    assert_eq!(
        extract_shortcut_change_category_url("https://b23.tv/oNcAbk    +"),
        Some("https://b23.tv/oNcAbk")
    );
    assert_eq!(
        extract_shortcut_change_category_url("https://b23.tv/oNcAbk    -"),
        Some("https://b23.tv/oNcAbk")
    );
    assert_eq!(
        extract_shortcut_change_category_url("+ https://b23.tv/oNcAbk +"),
        Some("https://b23.tv/oNcAbk")
    );
    assert_eq!(
        extract_shortcut_change_category_url("https://t.bilibili.com/548810564605393067  +"),
        Some("https://t.bilibili.com/548810564605393067")
    );

    assert_eq!(
        extract_shortcut_change_category_url("https://example.com/123  +"),
        None
    );
}

#[tokio::test]
async fn test_get_redirected_id() {
    pretty_env_logger::try_init().ok();
    assert_eq!(
        get_redirected_id("https://t.bilibili.com/581071094762952016")
            .await
            .unwrap(),
        "581071094762952016"
    );
    assert_eq!(
        get_redirected_id("https://m.bilibili.com/dynamic/581071094762952016?share_")
            .await
            .unwrap(),
        "581071094762952016"
    );
    assert_eq!(
        get_redirected_id("https://b23.tv/oNcAbk").await.unwrap(),
        "581071094762952016"
    );
    assert!(get_redirected_id("https://example.com").await.is_err());
    assert!(get_redirected_id("https://b23.tv/AOLrjh").await.is_err());
}
