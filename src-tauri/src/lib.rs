/*
// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![greet])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
*/

use serde::{Deserialize, Serialize};
use std::thread;
use std::time::Duration;
use tauri::Listener;
use tauri::Manager;

mod download;
mod js_eval;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Mp3Data {
    mp3_url: String,
    mp3_name: String, // 新增
}

#[tauri::command]
fn greet(message: String) {
    println!("[前端]{}", message);
}

fn inject_script(window: &tauri::WebviewWindow) {
    if let Ok(url) = window.url() {
        println!("[Rust] 执行 inject_script，URL: {}", url);
    }

    let _ = window.eval(js_eval::inject());
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            let main_window = app.get_webview_window("main").unwrap();
            let value = main_window.clone();

            // 监听来自前端的 mp3_captured 事件
            main_window.listen("mp3_captured", move |event| {
                let payload = event.payload();
                if !payload.is_empty() {
                    match serde_json::from_str::<Mp3Data>(payload) {
                        Ok(data) => {
                            println!("✅ 获取到 mp3_url: {}", data.mp3_url);
                            let window_for_download = value.clone();
                            thread::spawn(move || {
                                let m = download::download_file(
                                    &data.mp3_url,
                                    html_escape::decode_html_entities(&data.mp3_name)
                                        .to_string()
                                        .as_str(),
                                );
                                let m = m.as_str();
                                let _ = match m {
                                    "Done" => window_for_download.eval(js_eval::cleanup_and_back()),
                                    _ => window_for_download.eval(js_eval::show_error(m)),
                                };
                            });
                        }
                        Err(e) => eprintln!("⚠️ 解析失败: {:?}", e),
                    }
                } else {
                    println!("⚠️ 收到空 payload");
                }
            });

            // 启动后台线程，轮询 URL 变化并注入脚本
            let window_clone = main_window.clone();
            thread::spawn(move || {
                let mut last_url = String::new();
                loop {
                    // 获取当前 URL
                    if let Ok(url) = window_clone.url() {
                        let url_str = url.to_string();
                        if url_str != last_url {
                            last_url = url_str;
                            // URL 发生变化，注入脚本
                            println!("[Rust] URL 变化，注入脚本，当前 URL: {}", last_url);
                            inject_script(&window_clone);
                        }
                    }
                    thread::sleep(Duration::from_millis(100));
                }
            });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![greet])
        .run(tauri::generate_context!())
        .expect("启动失败");
}
