//! 获取 bilibili 封面
use crate::prelude::*;
use biliapi::requests::Request;
use lazy_static::lazy_static;
use once_cell::sync::Lazy;
use regex::Regex;

pub fn init(bot: Bot) {
    bot.handler(on_message::<FriendMessage>)
        .handler(on_message::<GroupMessage>);
}

async fn on_message<T: Conversation>(msg: T, bot: Bot) -> Result<()> {
    let s = msg.as_message().to_string();
    if !s.starts_with("封面") {
        return Ok(());
    }

    // 匹配全部 bv 号
    lazy_static! {
        static ref BV_REGEX: Regex = Regex::new(r"BV[\dA-Za-z]+").unwrap();
    }

    let client = Lazy::<reqwest::Client>::new(|| biliapi::connection::new_client().unwrap());

    for m in BV_REGEX.find_iter(&s) {
        let bv = m.as_str();
        let video_info = biliapi::requests::VideoInfo::request(&client, bv.to_string()).await?;
        let reply = MessageChain::new().image_url(video_info.cover_url);
        msg.reply(reply, &bot).await?;
    }

    Ok(())
}
