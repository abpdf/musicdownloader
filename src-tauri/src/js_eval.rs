pub fn inject() -> String {
    r#"
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
            const maxAttempts = 100;
            const interval = setInterval(() => {
                attempts++;
                const url = window.mp3_url;
                console.log(`[Tauri] 尝试 ${attempts}: mp3_url =`, url);

                if (typeof url === 'string' && url.startsWith('http')) {
                    clearInterval(interval);
                    console.log('[Tauri] 成功捕获 mp3_url:', url);

                    if (window.__TAURI__ && window.__TAURI__.event) {
                        window.__TAURI__.event.emit('mp3_captured', { mp3_url: url , mp3_name: window.mp3_name })
                            .then(() => {
                                console.log('[Tauri] 事件发送成功');
                                document.getElementsByTagName("body")[0].innerHTML="<br><br><br><h1>正在下载中，当下载完成时会自动返回上一页</h1>"
                            })
                            .catch(err => console.error('[Tauri] 事件发送失败:', err));
                    } else {
                        console.error('[Tauri] window.__TAURI__.event 不可用');
                    }
                } else if (attempts >= maxAttempts) {
                    clearInterval(interval);
                    console.log('[Tauri] 轮询超时，未捕获到 mp3_url');
                }
            }, 100);
        })();
    "#.to_string()
}

pub fn cleanup_and_back() -> String {
    r#"
    /*
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
*/
        // 返回上一页
        window.history.back();
    "#
    .to_string()
}

pub fn show_error(message: &str) -> String {
    format!("document.getElementsByTagName('body')[0].innerHTML='<br><br><br><h1>下载失败: {}</h1><br><button onclick=\"window.history.back()\"><h1>返回</h1></button>'",message)
}
