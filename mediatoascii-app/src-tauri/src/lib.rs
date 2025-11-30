use mediatoascii::image::ImageConfig;
use mediatoascii::video::{VideoConfig, VideoResult, PROGRESS_PERCENTAGE};
use tauri::{AppHandle, Emitter};

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
async fn process_video(config: VideoConfig) -> VideoResult<()> {
    unsafe {
        PROGRESS_PERCENTAGE = 0.0;
    }
    mediatoascii::video::process_video(config).inspect_err(|err| eprintln!("{:?}", err))
}

#[tauri::command]
async fn video_progress(app: AppHandle) {
    unsafe {
        while mediatoascii::video::PROGRESS_PERCENTAGE < 1.0 {
            app.emit("video-progress", mediatoascii::video::PROGRESS_PERCENTAGE).unwrap();
            tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        }
    }
}

#[tauri::command]
async fn process_image(config: ImageConfig) {
    mediatoascii::image::process_image(config);
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![process_video, video_progress, process_image])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
