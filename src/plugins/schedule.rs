use anyhow::Result;
use futures::StreamExt;
use miraie::prelude::*;
use parking_lot::Mutex;

pub fn init(bot: Bot) {
    bot.handler(on_msg::<GroupMessage>)
        .handler(on_msg::<FriendMessage>);
}

lazy_static::lazy_static! {
    static ref SCHEDULE_URL: Mutex<Option<String>> = Mutex::new(None);
}

async fn on_msg<T: Conversation>(msg: T, bot: Bot) -> Result<()> {
    match msg.as_message().to_string().as_str() {
        "日程表" => {
            let url = SCHEDULE_URL.lock().as_ref().cloned();
            match url {
                Some(url) => {
                    msg.reply(MessageBlock::image_url(url), &bot).await?;
                }
                None => {
                    msg.reply("日程表图片还未设置，使用【新日程表】指令设置", &bot)
                        .await?;
                }
            }
        }
        "新日程表" => {
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
                    {
                        *SCHEDULE_URL.lock() = Some(url.clone());
                    }
                    let reply = MessageChain::new().text("日程表已经设置为").image_url(url);
                    next_msg.reply(reply, &bot).await?;
                }
                _ => return Ok(()),
            }
        }
        _ => {}
    }
    Ok(())
}
