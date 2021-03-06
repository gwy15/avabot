//! 日程表
//!

use crate::Config;
use anyhow::Result;
use futures::StreamExt;
use miraie::prelude::*;

static KEY: &str = "A-SOUL_SCHEDULE_URL";

pub fn init(bot: Bot) {
    bot.command("日程表", on_日程表::<GroupMessage>)
        .command("日程表", on_日程表::<FriendMessage>)
        .command("新日程表", on_新日程表::<GroupMessage>)
        .command("新日程表", on_新日程表::<FriendMessage>);
}

fn get_url(db_path: &str) -> Result<Option<String>> {
    let db = sled::open(db_path)?;
    let v = db.get(KEY)?;
    Ok(v.map(|b| String::from_utf8_lossy(&b).to_string()))
}

fn set_url(db_path: &str, url: &str) -> Result<()> {
    let db = sled::open(db_path)?;
    db.insert(KEY, url)?;
    db.flush()?;
    Ok(())
}

async fn on_日程表<T: Conversation>(msg: T, bot: Bot, config: Data<Config>) -> Result<()> {
    match get_url(&config.db_path)? {
        Some(url) => {
            msg.reply(MessageBlock::image_url(url), &bot).await?;
        }
        None => {
            msg.reply("日程表图片还未设置，使用【新日程表】指令设置", &bot)
                .await?;
        }
    }
    Ok(())
}

async fn on_新日程表<T: Conversation>(msg: T, bot: Bot, config: Data<Config>) -> Result<()> {
    let db_path = config.as_ref().db_path.clone();
    msg.reply("在群里发送图片以设置新的日程表", &bot).await?;
    let next_msg = match msg.followed_sender_messages(&bot).next().await {
        Some(n) => n,
        None => return Ok(()),
    };
    let next_block = match next_msg.as_message().0.last() {
        Some(i) => i,
        None => return Ok(()),
    };
    info!("新日程表: {:?}", next_block);
    match next_block {
        MessageBlock::Image {
            image_id,
            url,
            base64,
        } => {
            info!("image: {}, {}, {:?}", image_id, url, base64);
            set_url(&db_path, url)?;
            let reply = MessageChain::new().text("日程表已经设置为").image_url(url);
            next_msg.reply(reply, &bot).await?;
        }
        _ => return Ok(()),
    }
    Ok(())
}

#[test]
fn test_sled_save_load() -> Result<()> {
    pretty_env_logger::try_init().ok();
    let dir = tempfile::tempdir()?;

    let path = dir.path().as_os_str().to_str().unwrap();

    assert_eq!(get_url(&path)?, None);

    set_url(&path, "HELLO_WORLD")?;
    assert_eq!(get_url(&path)?, Some("HELLO_WORLD".to_string()));

    set_url(&path, "向晚大魔王")?;
    assert_eq!(get_url(&path)?, Some("向晚大魔王".to_string()));

    Ok(())
}
