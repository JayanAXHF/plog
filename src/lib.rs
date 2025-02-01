use decrypt_cookies::{Browser, ChromiumBuilder, FirefoxBuilder};

#[tokio::main]
pub async fn main() -> miette::Result<()> {
    dbg!("safasf 0");
    let chromium = ChromiumBuilder::new(Browser::Chrome).build().await?;
    dbg!("safasf 1");
    let all_cookies = chromium.get_cookies_all().await?;
    dbg!("safasf");
    dbg!(&all_cookies
        .iter()
        .filter(|cookie| cookie.host_key.contains("lv"))
        .collect::<Vec<_>>());
    let jar: reqwest::cookie::Jar = all_cookies.into_iter().collect();

    Ok(())
}
