use crate::{prelude::*, Config};
use biliapi::Request;

async fn main() -> Result<i64> {
    let (client, cookies) = gen_article::persisted_client("./persisted_cookies.json").await?;

    match biliapi::requests::MyAccountInfo::request(&client, ()).await {
        Ok(data) => {
            info!("my account info: {:?}", data);
        }
        Err(e) => {
            bail!("当前账号登录过期，请重新设置 cookie: {}", e);
        }
    }

    let csrf = cookies
        .lock()
        .unwrap()
        .get("bilibili.com", "/", "bili_jct")
        .ok_or_else(|| anyhow!("missing csrf(bili_jct) cookie"))?
        .value()
        .to_string();

    let r = gen_article::generate(client, csrf).await?;

    Ok(r.aid)
}

pub async fn generate_daily(msg: GroupMessage, bot: Bot, config: Data<Config>) -> Result<()> {
    if !config
        .asoul_weekly
        .allow_groups
        .contains(&msg.sender.group.id)
    {
        return Ok(());
    }

    if !config.admins.contains(&msg.sender.id) {
        return Ok(());
    }

    msg.reply("开始生成日报", &bot).await?;
    match main().await {
        Ok(aid) => {
            msg.reply(format!("已生成今日日报，aid={}", aid), &bot)
                .await?;
        }
        Err(e) => {
            msg.reply(format!("生成日报发生错误：{:?}", e), &bot)
                .await?;
        }
    }
    Ok(())
}
