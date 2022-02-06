//! 生成诈骗链接
use crate::prelude::*;
use biliapi::requests::Request;
use regex::Regex;

lazy_static::lazy_static! {
    static ref BV_REGEX: Regex = Regex::new(r"BV[\da-zA-Z]+").unwrap();
}

pub fn init(bot: Bot) {
    bot.command("诈骗", on_message::<GroupMessage>);
}

async fn generate_fraud_url(real_bv: String, fake_bv: String) -> Result<String> {
    info!("目标 BV {real_bv}，虚假 BV {fake_bv}，尝试获取 avid");

    // 获取真实 av 号
    use biliapi::requests::VideoInfo;
    let client = biliapi::connection::new_client()?;
    let video_info: VideoInfo = VideoInfo::request(&client, real_bv).await?;
    let real_av = video_info.aid;
    info!("avid 为 {real_av}");

    Ok(format!(
        "https://www.bilibili.com/video/av{real_av}?{fake_bv}"
    ))
}

async fn on_message<T: Conversation + Sync>(msg: T, bot: Bot) -> Result<()> {
    let real_bv: T = msg.prompt("输入诈骗目标 BV", &bot).await?;
    let real_bv = real_bv.as_message().to_string();
    if !BV_REGEX.is_match(&real_bv) {
        bail!("输入不是 BV 号");
    }

    let fake_bv: T = msg.prompt("输入虚假 BV", &bot).await?;
    let fake_bv: String = fake_bv.as_message().to_string();
    if !BV_REGEX.is_match(&fake_bv) {
        bail!("输入不是 BV 号");
    }

    let url = generate_fraud_url(real_bv, fake_bv).await?;
    debug!("已生成链接：{url}");
    msg.reply(url, &bot).await?;

    Ok(())
}

#[tokio::test]
async fn test_generate_url() -> Result<()> {
    let url = generate_fraud_url("BV17b4y1J7ed".to_string(), "BV1nS4y1574h".to_string()).await?;
    assert_eq!(
        url,
        "https://www.bilibili.com/video/av635700727?BV1nS4y1574h"
    );
    Ok(())
}
