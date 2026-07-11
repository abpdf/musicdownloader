// src-tauri/src/download.rs
use std::fs::File;
use std::io::Write;
use reqwest::blocking::Client;

/// 下载文件到系统下载目录
/// - url: 文件下载链接
/// - name: 文件名（不含扩展名），若为空则使用 "music"
pub fn download_file(url: &str, name: &str) {
    // 获取系统下载目录
    let dir = match dirs::download_dir() {
        Some(dir) => dir,
        None => {
            eprintln!("⚠️ 无法获取下载目录");
            return;
        }
    };

    // 从 URL 提取扩展名（最后一个点之后的部分）
    let extension = url
        .split('/')
        .last()
        .and_then(|part| part.split('.').last())
        .unwrap_or("mp3");
    // 确保扩展名不带查询参数
    let extension = extension.split('?').next().unwrap_or("mp3");

    // 组合文件名：使用提供的 name，若为空则用 "music"
    let base_name = if name.is_empty() { "music" } else { name };
    let filename = format!("{}.{}", base_name, extension);
     // 创建子目录路径
    let download_folder = dir.join("musicdownloaded");

    // 如果目录不存在，则创建（包括父目录）
    if !download_folder.exists() {
        if let Err(e) = std::fs::create_dir_all(&download_folder) {
            eprintln!("❌ 创建目录失败: {}", e);
            return;
        }
        println!("📁 创建目录: {:?}", download_folder);
    }
    let file_path = download_folder.join(filename);

    println!("📥 开始下载: {} -> {:?}", url, file_path);

    let client = Client::new();
    match client.get(url).send() {
        Ok(response) => {
            if response.status().is_success() {
                let bytes = match response.bytes() {
                    Ok(b) => b,
                    Err(e) => {
                        eprintln!("❌ 读取响应数据失败: {}", e);
                        return;
                    }
                };
                match File::create(&file_path) {
                    Ok(mut file) => {
                        if let Err(e) = file.write_all(&bytes) {
                            eprintln!("❌ 写入文件失败: {}", e);
                        } else {
                            println!("✅ 下载成功: {}", file_path.display());
                        }
                    }
                    Err(e) => eprintln!("❌ 创建文件失败: {}", e),
                }
            } else {
                eprintln!("❌ HTTP 错误: {}", response.status());
            }
        }
        Err(e) => eprintln!("❌ 下载请求失败: {}", e),
    }
}