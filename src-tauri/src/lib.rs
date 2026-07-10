use tauri::{Builder, Manager, Webview, PageLoadPayload, webview::PageLoadEvent};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Mp3Data {
    mp3_url: String,
}

#[tauri::command]
fn greet(message: String) {
    println!("[Rust] Received greet: {}", message);
}

// 注入脚本（参数不变，仍为 WebviewWindow）
fn inject_script(window: &tauri::WebviewWindow) {
    let script = r#"
        (function() {
            if (typeof window.__TAURI__ === 'undefined') {
                console.warn('__TAURI__ not available');
                return;
            }

            window.__TAURI__.invoke('greet', { message: 'Injected!' })
                .catch(err => console.error('invoke greet error:', err));

            if (window.__mp3_listener_active) return;
            window.__mp3_listener_active = true;

            let lastMp3Url = null;

            function checkAndEmit() {
                const url = window.mp3_url;
                if (typeof url === 'string' && url.startsWith('http')) {
                    if (url !== lastMp3Url) {
                        lastMp3Url = url;
                        const emit = window.__TAURI__.emit || window.__TAURI__.core?.emit;
                        if (emit) {
                            emit('mp3_captured', { mp3_url: url })
                                .catch(err => console.error('emit error:', err));
                            console.log('[Frontend] Emitted mp3_captured:', url);
                        } else {
                            console.warn('emit function not found');
                        }
                    }
                }
            }

            let attempts = 0;
            const maxAttempts = 30;
            const interval = setInterval(() => {
                attempts++;
                checkAndEmit();
                if (attempts >= maxAttempts || window.mp3_url) {
                    clearInterval(interval);
                }
            }, 500);

            const origPushState = history.pushState;
            history.pushState = function() {
                origPushState.apply(this, arguments);
                setTimeout(checkAndEmit, 300);
            };
            const origReplaceState = history.replaceState;
            history.replaceState = function() {
                origReplaceState.apply(this, arguments);
                setTimeout(checkAndEmit, 300);
            };
            window.addEventListener('popstate', () => setTimeout(checkAndEmit, 300));

            checkAndEmit();
        })();
        true;
    "#;
    let _ = window.eval(script);
}

pub fn run() {
    Builder::default()
        // 👇 使用 Builder::on_page_load，闭包参数是 &Webview
        .on_page_load(|webview: &Webview, payload: &PageLoadPayload| {
                // 从 Webview 获取对应的窗口引用
                let window = webview.window(); // 返回 &WebviewWindow
                // 只对标签为 "main" 的窗口注入
                if window.label() == "main" {
                    inject_script(window);
                }
        })
        .setup(|app| {
            let window = app.get_webview_window("main").unwrap();

            // 注册事件监听器（只注册一次）
            window.listen("mp3_captured", |event| {
                let payload = event.payload();
                if !payload.is_empty() {
                    match serde_json::from_str::<Mp3Data>(payload) {
                        Ok(data) => println!("✅ [Rust] Captured mp3_url: {}", data.mp3_url),
                        Err(e) => eprintln!("⚠️ [Rust] Failed to parse payload: {}", e),
                    }
                } else {
                    println!("⚠️ [Rust] Empty payload received");
                }
            });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![greet])
        .run(tauri::generate_context!())
        .expect("启动失败");
}