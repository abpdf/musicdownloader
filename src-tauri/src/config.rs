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
});
}

#[tauri::command]
pub fn read_saved_folder(app: tauri::AppHandle) -> Result<String, String> {
    let config_dir = app.path().app_config_dir()
        .map_err(|e| e.to_string())?;
    let file_path = config_dir.join("saved_folder.txt");

    if !file_path.exists() {
        return Err("尚未保存过文件夹路径".into());
    }

    fs::read_to_string(&file_path)
        .map_err(|e| e.to_string())
}