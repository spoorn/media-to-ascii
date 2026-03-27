use mediatoascii::video::{VideoConfig, VideoResult};
use std::path::Path;
use serde::Serialize;
use tauri::{AppHandle, Emitter};

#[derive(Clone, Serialize)]
struct VideoProgressInfo {
    percentage: f32,
    current_frame: u64,
    total_frames: u64,
}

#[tauri::command]
fn file_exists(path: &str) -> bool {
    Path::new(path).exists()
}

#[tauri::command]
async fn process_video(config: VideoConfig) -> VideoResult<()> {
    mediatoascii::video::process_video(config).inspect_err(|err| eprintln!("{:?}", err))
}

#[tauri::command]
async fn video_progress(app: AppHandle) {
    unsafe {
        while mediatoascii::video::PROGRESS_PERCENTAGE < 1.0 {
            app.emit("video-progress", VideoProgressInfo {
                percentage: mediatoascii::video::PROGRESS_PERCENTAGE,
                current_frame: mediatoascii::video::CURRENT_FRAME,
                total_frames: mediatoascii::video::TOTAL_FRAMES,
            }).unwrap();
            tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        }
    }
}

#[tauri::command]
fn cancel_processing() {
    unsafe {
        mediatoascii::video::CANCEL_REQUESTED = true;
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![process_video, video_progress, cancel_processing, file_exists])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
