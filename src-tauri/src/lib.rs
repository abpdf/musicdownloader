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

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Mp3Data {
    mp3_url: String,
    mp3_name: String,   // 新增
}

#[tauri::command]
fn greet(message: String) {
    println!("[前端]{}", message);
}

fn inject_script(window: &tauri::WebviewWindow) {
    if let Ok(url) = window.url() {
        println!("[Rust] 执行 inject_script，URL: {}", url);
    }

    let script = r#"
        (function() {
            if (window.__mp3_injected) {
                console.log('[Tauri] 该页面已注入，跳过');
                return;
            }
            window.__mp3_injected = true;

            // ====== 新增：拦截所有新窗口打开 ======
            // 拦截 <a target="_blank"> 点击
            document.addEventListener('click', function(e) {
                let target = e.target.closest('a');
                if (target && target.tagName === 'A' && target.target === '_blank') {
                    e.preventDefault();
                    // 在当前窗口跳转
                    window.location.href = target.href;
                }
            }, true);

            // 拦截 window.open
            window.open = function(url, name, features) {
                // 忽略参数，直接在当前窗口跳转
                window.location.href = url;
                // 返回一个模拟窗口对象（避免报错）
                return {
                    closed: false,
                    close: function() {},
                    focus: function() {},
                    blur: function() {},
                    postMessage: function() {}
                };
            };
            // 也拦截浏览器自身的新窗口行为（例如 middle-click 或 Ctrl+click）
            // 但无法拦截所有，以上已覆盖主要场景。

            // 通知 Rust 注入成功
            if (window.__TAURI__ && window.__TAURI__.core) {
                window.__TAURI__.core.invoke("greet", { message: "injected" });
            } else {
                console.error('[Tauri] __TAURI__ 不可用');
            }

            console.log('[Tauri] 注入脚本成功，当前URL:', location.href);

            if (!location.href.startsWith('https://www.gequhai.com/play')) {
                console.log('[Tauri] URL不匹配，跳过监听');
                return;
            }

            console.log('[Tauri] URL匹配，开始轮询 mp3_url');

            let attempts = 0;
            const maxAttempts = 30;
            const interval = setInterval(() => {
                attempts++;
                const url = window.mp3_url;
                console.log(`[Tauri] 尝试 ${attempts}: mp3_url =`, url);

                if (typeof url === 'string' && url.startsWith('http')) {
                    clearInterval(interval);
                    console.log('[Tauri] 成功捕获 mp3_url:', url);

                    if (window.__TAURI__ && window.__TAURI__.event) {
                        window.__TAURI__.event.emit('mp3_captured', { mp3_url: url , mp3_name: window.mp3_name })
                            .then(() => console.log('[Tauri] 事件发送成功'))
                            .catch(err => console.error('[Tauri] 事件发送失败:', err));
                    } else {
                        console.error('[Tauri] window.__TAURI__.event 不可用');
                    }
                } else if (attempts >= maxAttempts) {
                    clearInterval(interval);
                    console.log('[Tauri] 轮询超时，未捕获到 mp3_url');
                }
            }, 500);
        })();
    "#;
    let _ = window.eval(script);
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
                                download::download_file(&data.mp3_url, html_escape::decode_html_entities(&data.mp3_name).to_string().as_str());
                                    let _ = window_for_download.eval(r#"
        // 清除 localStorage 和 sessionStorage
        localStorage.clear();
        sessionStorage.clear();

        // 清除所有 Cookie
        document.cookie.split(';').forEach(function(c) {
            c = c.trim();
            if (c) {
                var eq = c.indexOf('=');
                var name = eq > -1 ? c.substr(0, eq) : c;
                document.cookie = name + '=; expires=Thu, 01 Jan 1970 00:00:00 GMT; path=/';
            }
        });

        // 清除 IndexedDB（如果支持）
        if (window.indexedDB && typeof window.indexedDB.databases === 'function') {
            window.indexedDB.databases().then(function(dbs) {
                dbs.forEach(function(db) {
                    window.indexedDB.deleteDatabase(db.name);
                });
            }).catch(function() {});
        }

        // 返回上一页
        window.history.back();
    "#);
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
                            inject_script(&window_clone);
                        }
                    }
                    thread::sleep(Duration::from_millis(150));
                }
            });

            // 初次注入（线程会在首次循环中检测到并注入，但这里先注入一次以加快响应）
            inject_script(&main_window);

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![greet])
        .run(tauri::generate_context!())
        .expect("启动失败");
}
