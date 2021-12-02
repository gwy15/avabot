//! A-SOUL周报相关插件

use chrono::{Duration, Utc};
use regex::Regex;
use std::collections::HashSet;

use crate::prelude::*;

mod command;
mod utils;

use command::Command;

pub fn init(bot: Bot) {
    bot.handler(on_message);
}

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    /// asoul_weekly 的链接
    url: String,
    allow_groups: HashSet<QQ>,
}

async fn on_message(msg: GroupMessage, config: Data<crate::Config>, bot: Bot) -> Result<()> {
    if !config
        .asoul_weekly
        .allow_groups
        .contains(&msg.sender.group.id)
    {
        return Ok(());
    }

    match parse_message(msg.as_message().to_string().as_str().trim()).await {
        Ok(Some(cmd)) => {
            debug!("消息 {:?} 匹配成功: {:?}", msg, cmd);
            let reply = cmd.execute(&config.asoul_weekly.url).await?;
            msg.reply(reply, &bot).await?;
        }
        Ok(None) => {
            // 没匹配到
        }
        Err(e) => {
            // 匹配过程中发生错误，需要打回客户端
            warn!("匹配发生错误: {:?}", e);
            msg.reply(format!("发生错误: {:?}", e), &bot).await?;
        }
    }

    Ok(())
}

async fn parse_message(msg: &str) -> Result<Option<Command>> {
    // 查询分类
    if let Ok(cmd) = parse_query_category(msg) {
        return Ok(Some(cmd));
    }

    match msg {
        msg if msg.starts_with("分类") || msg.starts_with('?') => {
            let command = parse_query_category(msg)?;
            Ok(Some(command))
        }
        // 归档
        msg if msg.ends_with("归档") => {
            let date = match msg {
                "今日归档" | "今天归档" | "归档" => Utc::now(),
                "昨天归档" | "昨日归档" => Utc::now() - Duration::days(1),
                "前天归档" => Utc::now() - Duration::days(2),
                _ => return Ok(None),
            };
            Ok(Some(Command::Summary { date }))
        }
        // kpi
        msg if msg.to_lowercase().ends_with("kpi") => {
            let date = match msg.to_lowercase().as_str() {
                "今日kpi" | "今天kpi" | "kpi" => Utc::now(),
                "昨天kpi" | "昨日kpi" => Utc::now() - Duration::days(1),
                "前天kpi" => Utc::now() - Duration::days(2),
                _ => return Ok(None),
            };
            Ok(Some(Command::Kpi { date }))
        }
        // 修改分类
        msg if msg.starts_with("修改分类") => {
            let command = parse_change_category(msg)?;
            Ok(Some(command))
        }
        // 缩写
        msg => parse_shortcut(msg).await,
    }
}

fn parse_query_category(msg: &str) -> Result<Command> {
    lazy_static::lazy_static! {
        static ref PATTERN: Regex = Regex::new(r"^(分类|\?|？)\s+(?P<id>\w+)$").unwrap();
    }
    match PATTERN.captures(msg) {
        Some(cap) => {
            let id = cap
                .name("id")
                .ok_or_else(|| anyhow!("获取分类缺少 ID"))?
                .as_str()
                .to_string();
            Ok(Command::Query { id })
        }
        None => {
            bail!("格式错误，格式需要是：\n分类 BVxxxxxxx")
        }
    }
}

/// 处理以 修改分类 开始的指令
fn parse_change_category(msg: &str) -> Result<Command> {
    lazy_static::lazy_static! {
        static ref PATTERN: Regex = Regex::new(r"^修改分类\s+(\w+)\s+([^\s]+)$").unwrap();
    }
    match PATTERN.captures(msg) {
        Some(cap) => {
            let id = cap
                .get(1)
                .ok_or_else(|| anyhow!("修改分类缺少 ID"))?
                .as_str()
                .to_string();
            let category = cap.get(2).ok_or_else(|| anyhow!("缺少分类"))?.as_str();

            match category {
                "删除" | "-" => Ok(Command::Delete { id }),
                cat if cat.starts_with('+') => {
                    let cat = cat.trim_start_matches('+');
                    Ok(Command::Add {
                        id,
                        category: cat.to_string(),
                    })
                }
                cat => Ok(Command::Change {
                    id,
                    category: cat.to_string(),
                }),
            }
        }
        None => {
            bail!("格式错误，格式需要是：\n修改分类 BVxxxxxxxx 翻唱/删除")
        }
    }
}

/// 尝试按照简写形式进行匹配
async fn parse_shortcut(msg: &str) -> Result<Option<Command>> {
    lazy_static::lazy_static! {
        static ref URL_REGEXP: Regex =
            Regex::new(r"https://((b23\.tv|.+\.bilibili.com)/\w+)").unwrap();
    }
    if !(msg.ends_with('+') || msg.ends_with('-')) {
        return Ok(None);
    }
    let url = match URL_REGEXP.captures(msg) {
        Some(cap) => cap.get(0).map(|m| m.as_str()),
        None => return Ok(None),
    };
    let url = match url {
        Some(url) => url,
        None => return Ok(None),
    };
    //
    let id = utils::get_redirected_id(url).await?;
    let cmd = if msg.ends_with('+') {
        Command::Add {
            id,
            category: "动态".to_string(),
        }
    } else {
        Command::Delete { id }
    };
    Ok(Some(cmd))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_shortcut() -> Result<()> {
        assert_eq!(
            parse_message("https://b23.tv/oNcAbk +").await?,
            Some(Command::Add {
                id: "581071094762952016".to_string(),
                category: "动态".to_string()
            })
        );

        assert_eq!(
            parse_message("https://b23.tv/oNcAbk [@人] +").await?,
            Some(Command::Add {
                id: "581071094762952016".to_string(),
                category: "动态".to_string()
            })
        );

        assert_eq!(
            parse_message("[@人] https://b23.tv/oNcAbk +").await?,
            Some(Command::Add {
                id: "581071094762952016".to_string(),
                category: "动态".to_string()
            })
        );

        assert_eq!(
            parse_message("https://b23.tv/oNcAbk   -").await?,
            Some(Command::Delete {
                id: "581071094762952016".to_string(),
            })
        );
        assert_eq!(
            parse_message("https://t.bilibili.com/548810564605393067  +").await?,
            Some(Command::Add {
                id: "548810564605393067".to_string(),
                category: "动态".to_string()
            })
        );
        assert_eq!(parse_message("https://example.com/123  +").await?, None);

        assert!(parse_message("https://b23.tv/bjPZgml  +").await.is_err());

        Ok(())
    }

    #[tokio::test]
    async fn test_change_category() -> Result<()> {
        assert_eq!(
            parse_message("修改分类  BV1AR4y147gy  其他").await?,
            Some(Command::Change {
                id: "BV1AR4y147gy".to_string(),
                category: "其他".to_string()
            })
        );

        assert_eq!(
            parse_message("修改分类  BV1AR4y147gy  +其他").await?,
            Some(Command::Add {
                id: "BV1AR4y147gy".to_string(),
                category: "其他".to_string()
            })
        );

        assert_eq!(
            parse_message("修改分类  BV1AR4y147gy -").await?,
            Some(Command::Delete {
                id: "BV1AR4y147gy".to_string(),
            })
        );

        assert_eq!(
            parse_message("修改分类  BV1AR4y147gy 删除").await?,
            Some(Command::Delete {
                id: "BV1AR4y147gy".to_string(),
            })
        );

        assert_eq!(
            parse_message("修改分类  BV1AR4y147gy  +A-SOUL").await?,
            Some(Command::Add {
                id: "BV1AR4y147gy".to_string(),
                category: "A-SOUL".to_string()
            })
        );

        assert_eq!(
            parse_message("修改分类  BV1AR4y147gy  A-SOUL").await?,
            Some(Command::Change {
                id: "BV1AR4y147gy".to_string(),
                category: "A-SOUL".to_string()
            })
        );

        Ok(())
    }

    #[tokio::test]
    async fn test_query_category() -> Result<()> {
        assert_eq!(
            parse_message("分类  BV1AR4y147gy").await?,
            Some(Command::Query {
                id: "BV1AR4y147gy".to_string(),
            })
        );
        assert_eq!(
            parse_message("分类  587803185410047312").await?,
            Some(Command::Query {
                id: "587803185410047312".to_string(),
            })
        );
        assert_eq!(
            parse_message("分类 587803185410047312").await?,
            Some(Command::Query {
                id: "587803185410047312".to_string(),
            })
        );
        assert_eq!(
            parse_message("? 587803185410047312").await?,
            Some(Command::Query {
                id: "587803185410047312".to_string(),
            })
        );
        assert_eq!(parse_message("？啥").await?, None);
        Ok(())
    }

    #[tokio::test]
    async fn test_other_command() -> Result<()> {
        assert!(matches!(
            parse_message("kpi").await?,
            Some(Command::Kpi { .. })
        ));

        assert!(matches!(
            parse_message("归档").await?,
            Some(Command::Summary { .. })
        ));

        Ok(())
    }
}
