use decrypt_cookies::{Browser, ChromiumBuilder};

#[tokio::main]
pub async fn main() -> miette::Result<()> {
    let chromium = ChromiumBuilder::new(Browser::Chromium).build().await?;
    let all_cookies = chromium.get_cookies_all().await?;

    dbg!(&all_cookies[0]);

    let jar: reqwest::cookie::Jar = all_cookies.into_iter().collect();

    Ok(())
}
