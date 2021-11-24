use anyhow::*;
use regex::Regex;

/// 从 b23 短链或者 t.bilibili.com 长链解析出动态 id
pub async fn get_redirected_id(url: &str) -> Result<String> {
    lazy_static::lazy_static! {
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
