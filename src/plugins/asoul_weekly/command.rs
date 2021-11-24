use crate::prelude::*;
use chrono::{DateTime, Utc};
use reqwest::Client;
use serde_json::json;
use std::collections::BTreeMap;
use std::fmt::Write;

macro_rules! errorcheck {
    ($response:expr) => {{
        let status = $response.status();
        if status != reqwest::StatusCode::OK {
            let json: serde_json::Value = $response.json().await?;
            info!("请求返回 json：{:?}", json);
            if let Some(r) = json.get("error").cloned() {
                if let Some(s) = r.as_str() {
                    return Ok(s.to_string());
                }
            }
            error!("请求错误：{:?}", json);
            return Ok(json.to_string());
        }
    }};
}

#[derive(Debug, PartialEq, Eq)]
pub enum Command {
    Add { id: String, category: String },
    Delete { id: String },
    Change { id: String, category: String },
    Summary { date: DateTime<Utc> },
    Kpi { date: DateTime<Utc> },
}
impl Command {
    pub async fn execute(self, base_url: &str) -> Result<String> {
        let client = Client::new();
        match &self {
            Command::Add { id, category } => {
                let response = client
                    .post(format!("{}/items/{}/category", base_url, id))
                    .json(&json!({ "category": category }))
                    .send()
                    .await?;
                errorcheck!(response);
                Ok(format!("新建 id {} 为 {} 成功", id, category))
            }
            Command::Delete { id } => {
                let response = client
                    .delete(format!("{}/items/{}/category", base_url, id))
                    .send()
                    .await?;
                errorcheck!(response);
                Ok(format!("删除 id {} 分类成功", id))
            }
            Command::Change { id, category } => {
                let response = client
                    .patch(format!("{}/items/{}/category", base_url, id))
                    .json(&json!({ "category": category }))
                    .send()
                    .await?;
                errorcheck!(response);
                Ok(format!("修改 id {} 为 {} 成功", id, category))
            }
            Command::Summary { date } => {
                let t = date.to_rfc3339_opts(chrono::SecondsFormat::Secs, true);
                let url = format!("{}/summary?t={}", base_url, t);
                let response = client.get(url).send().await?;
                errorcheck!(response);

                let summary: BTreeMap<String, Vec<String>> = response.json().await?;
                let mut message = String::new();
                for (category, ids) in summary {
                    writeln!(&mut message, "------ {} -----", category)?;
                    if category == "动态" {
                        for id in ids {
                            message += &id;
                            message += "\n";
                        }
                    } else {
                        for id in ids.chunks(4) {
                            message += &id.join("  ");
                            message += "\n";
                        }
                    }
                }
                Ok(message)
            }
            Command::Kpi { date } => {
                let t = date.to_rfc3339_opts(chrono::SecondsFormat::Secs, true);
                let url = format!("{}/kpi?t={}", base_url, t);
                let response = client.get(url).send().await?;
                errorcheck!(response);

                #[derive(Debug, Deserialize)]
                struct Row {
                    name: String,
                    times: u32,
                }
                let kpi: Vec<Row> = response.json().await?;
                let mut message = String::new();
                for row in kpi {
                    writeln!(message, "【{}】筛选了 【{}】 个", row.name, row.times)?;
                }
                Ok(message)
            }
        }
    }
}
