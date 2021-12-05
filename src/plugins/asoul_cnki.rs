//! 枝网查重
//!

use std::collections::HashMap;

use chrono::{DateTime, Utc};

use crate::prelude::*;

pub fn init(bot: Bot) {
    bot.handler(on_message);
}

async fn on_message(group_message: GroupMessage, bot: Bot) -> Result<()> {
    let cmd_msg = group_message
        .message
        .0
        .iter()
        .filter(|b| matches!(b, MessageBlock::Text { .. }))
        .map(|b| b.to_string())
        .collect::<Vec<_>>()
        .join("");

    if cmd_msg.trim() != "枝网查重" {
        return Ok(());
    }

    let source = group_message
        .message
        .0
        .iter()
        .filter_map(|b| match b {
            MessageBlock::Quote { origin, .. } => Some(origin),
            _ => None,
        })
        .next();

    let content = match source {
        Some(source) => source.clone(),
        None => {
            // 主动要
            let r = group_message.prompt("输入查重内容", &bot).await?;
            r.message
        }
    };
    let result = get_asoul_cnki(content).await?;

    // 返回结果
    group_message.reply(result, &bot).await?;
    Ok(())
}

#[derive(Debug, Deserialize)]
struct Response {
    code: i32,
    message: String,
    data: Option<ResponseData>,
}

#[derive(Debug, Deserialize)]
#[allow(unused)]
struct ResponseData {
    #[serde(rename = "rate")]
    similarity: f64,
    #[serde(with = "chrono::serde::ts_seconds")]
    start_time: DateTime<Utc>,
    #[serde(with = "chrono::serde::ts_seconds")]
    end_time: DateTime<Utc>,
    /// 相似率，原文，原文链接
    related: Vec<Related>,
}

#[derive(Debug, Deserialize)]
struct Related {
    #[serde(rename = "rate")]
    similarity: f64,

    reply: Reply,

    reply_url: String,
}

#[derive(Debug, Deserialize)]
struct Reply {
    #[serde(rename = "m_name")]
    author: String,

    #[serde(rename = "ctime", with = "chrono::serde::ts_seconds")]
    #[allow(unused)]
    create_time: DateTime<Utc>,

    // dynamic_id: i64,
    content: String,
}

async fn get_asoul_cnki(chain: MessageChain) -> Result<String> {
    //
    let s = chain
        .0
        .into_iter()
        .filter_map(|b| match b {
            MessageBlock::Text { text } => Some(text),
            _ => None,
        })
        .collect::<Vec<_>>()
        .join("");
    get_asoul_cnki_from_str(&s).await
}

async fn get_asoul_cnki_from_str(s: &str) -> Result<String> {
    let resp = reqwest::Client::new()
        .post("https://asoulcnki.asia/v1/api/check")
        .json(&{
            let mut m = HashMap::new();
            m.insert("text", s);
            m
        })
        .send()
        .await?;
    let text = resp.text().await?;
    debug!("查重返回结果 = {}", text);
    let resp: Response = serde_json::from_str(&text)?;
    if resp.code != 0 || resp.data.is_none() {
        error!("resp.code = {}, message = {}", resp.code, resp.message);
        if resp.message.contains("Illegal Capacity") {
            return Ok(String::from("小作文太短了捏~"));
        } else {
            bail!("枝网查重返回错误：{}", resp.message);
        }
    }

    let data = resp.data.unwrap();
    let mut res = format!("查重结果：相似度 {:.2}%", data.similarity * 100.);
    if !data.related.is_empty() {
        let first = &data.related[0];
        res.push_str(&format!(
            "\n相似小作文：相似度 {:.2}%\n作者：{}\n链接：{}\n{}",
            first.similarity * 100.0,
            first.reply.author,
            first.reply_url,
            first.reply.content,
        ));
    }

    Ok(res)
}

#[tokio::test]
#[ignore]
async fn test_get_asoul_cnki() {
    let s = "我把泪水搜集，暴晒在阳光下，不知道有没有到达然然哪里。";
    assert!(get_asoul_cnki_from_str(s).await.unwrap().contains("辈咯立"));
}
