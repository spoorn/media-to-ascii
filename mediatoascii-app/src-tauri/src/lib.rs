use mediatoascii::video::{VideoConfig, VideoResult};
use tauri::{AppHandle, Emitter};

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
async fn process_video(config: VideoConfig) -> VideoResult<()> {
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

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![process_video, video_progress])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
