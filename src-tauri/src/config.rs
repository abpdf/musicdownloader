use tauri::Manager;
use tauri_plugin_dialog::DialogExt;
use std::fs;

#[tauri::command]
pub fn pick_and_save_folder(app: tauri::AppHandle){
    app.dialog().file().pick_folder(move |folder_path| {
    let Some(path) = folder_path else {
        return;
    };

    // 直接使用 to_string() 获取路径字符串
    let path_str = path.to_string();

    let config_dir = match app.path().app_config_dir() {
        Ok(dir) => dir,
        Err(e) => {
            eprintln!("获取配置目录失败: {}", e);
            return;
        }
    };

    if let Err(e) = std::fs::create_dir_all(&config_dir) {
        eprintln!("创建目录失败: {}", e);
        return;
    }

    let file_path = config_dir.join("saved_folder.txt");
    if let Err(e) = std::fs::write(&file_path, &path_str) {
        eprintln!("写入文件失败: {}", e);
    }
    let _ = app.get_webview_window("main").unwrap().eval("window.forceRefresh()");

});
}

#[tauri::command]
pub fn read_saved_folder(app: tauri::AppHandle) -> Result<String, String> {
    // 使用 ? 获取 PathBuf，如果出错则直接返回 Err(String)
    let config_dir = app.path().app_config_dir()
        .map_err(|e| e.to_string())?;  // 注意这里的 ? 

    let file_path = config_dir.join("saved_folder.txt");

    if !file_path.exists() {
        return Err("尚未保存过文件夹路径".into());
    }

    // 读取文件，? 同样会传播错误
    fs::read_to_string(&file_path)
        .map_err(|e| e.to_string())  // 这里不能用 ?，因为最后要返回 Result<String, String>
        // 但 map_err 已经转换了错误类型，所以可以直接返回
}

#[tauri::command]
pub fn reset_path(app: tauri::AppHandle)-> Result<(), String>{
    let config_dir = app.path().app_config_dir()
        .map_err(|e| e.to_string())?;
    let file_path = config_dir.join("saved_folder.txt");

    if file_path.exists() {
    let _ = fs::remove_file(file_path);
    }
    let _ = app.get_webview_window("main").unwrap().eval("window.forceRefresh()");
    Ok(())
}
