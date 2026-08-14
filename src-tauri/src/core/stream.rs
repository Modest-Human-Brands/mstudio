use std::sync::Mutex;
// use tauri::Emitter;
use tauri_plugin_shell::process::CommandChild;
use tauri_plugin_shell::process::CommandEvent;
use tauri_plugin_shell::ShellExt;

pub struct PreviewState(pub Mutex<Option<CommandChild>>);

#[derive(serde::Serialize)]
pub struct MediaDevices {
    video: Vec<String>,
    audio: Vec<String>,
}

pub fn get_hls_dir() -> std::path::PathBuf {
    let dir = std::env::temp_dir().join("msync_hsl_stream");
    let _ = std::fs::create_dir_all(&dir);
    dir
}

pub fn clean_device_label(mut label: &str) -> String {
    if label.starts_with("Default - ") {
        label = &label["Default - ".len()..];
    }
    if let Some(idx) = label.rfind(" (") {
        let suffix = &label[idx..];
        if suffix.contains(':') && suffix.ends_with(')') {
            return label[..idx].to_string();
        }
    }
    label.to_string()
}

#[tauri::command]
pub fn stop_preview(state: tauri::State<PreviewState>) -> Result<(), String> {
    if let Some(child) = state.0.lock().unwrap().take() {
        let _ = child.kill();
    }
    Ok(())
}

#[tauri::command]
pub async fn get_devices(app: tauri::AppHandle) -> Result<MediaDevices, String> {
    let output = app
        .shell()
        .sidecar("ffmpeg")
        .map_err(|e| e.to_string())?
        .args(["-list_devices", "true", "-f", "dshow", "-i", "dummy"])
        .output()
        .await
        .map_err(|e| e.to_string())?;

    let stderr = String::from_utf8_lossy(&output.stderr);
    let mut video = Vec::new();
    let mut audio = Vec::new();
    let mut parsing_video = true;

    for line in stderr.lines() {
        let lower = line.to_lowercase();

        if lower.contains("directshow video devices") {
            parsing_video = true;
        } else if lower.contains("directshow audio devices") {
            parsing_video = false;
        } else if lower.contains("alternative name") {
            continue;
        } else if let (Some(start), Some(end)) = (line.find('"'), line.rfind('"')) {
            if start != end {
                let device_name = &line[start + 1..end];
                if device_name != "dummy" {
                    let name_lower = device_name.to_lowercase();

                    let is_audio = name_lower.contains("microphone")
                        || name_lower.contains("audio")
                        || name_lower.contains("sound");

                    let is_video = name_lower.contains("camera")
                        || name_lower.contains("webcam")
                        || name_lower.contains("video")
                        || name_lower.contains("capture");

                    if is_audio && !is_video {
                        if !audio.contains(&device_name.to_string()) {
                            audio.push(device_name.to_string());
                        }
                    } else if is_video && !is_audio {
                        if !video.contains(&device_name.to_string()) {
                            video.push(device_name.to_string());
                        }
                    } else {
                        if parsing_video {
                            if !video.contains(&device_name.to_string()) {
                                video.push(device_name.to_string());
                            }
                        } else {
                            if !audio.contains(&device_name.to_string()) {
                                audio.push(device_name.to_string());
                            }
                        }
                    }
                }
            }
        }
    }

    Ok(MediaDevices { video, audio })
}

#[tauri::command]
pub async fn start_stream(
    app: tauri::AppHandle,
    preview_state: tauri::State<'_, PreviewState>,
    stream_key: String,
    video_device: String,
    audio_device: String,
) -> Result<String, String> {
    if let Some(child) = preview_state.0.lock().unwrap().take() {
        let _ = child.kill();
    }

    let rtmp_url = format!("rtmps://a.rtmps.youtube.com/live2/{}", stream_key);
    let safe_video = clean_device_label(&video_device).replace(":", "\\:");
    let safe_audio = clean_device_label(&audio_device).replace(":", "\\:");
    let video_input = format!("video={}", safe_video);
    let audio_input = format!("audio={}", safe_audio);

    let hls_dir = get_hls_dir();
    let _ = std::fs::remove_dir_all(&hls_dir);
    let _ = std::fs::create_dir_all(&hls_dir);
    let hls_path = hls_dir.join("index.m3u8");

    let mut ffmpeg_args = vec![
        "-y".to_string(),
        "-f".to_string(),
        "dshow".to_string(),
        "-rtbufsize".to_string(),
        "100M".to_string(),
        "-i".to_string(),
        video_input,
        "-f".to_string(),
        "dshow".to_string(),
        "-i".to_string(),
        audio_input,
    ];

    if !stream_key.trim().is_empty() {
        ffmpeg_args.extend(vec![
            "-c:v".to_string(),
            "libx264".to_string(),
            "-preset".to_string(),
            "veryfast".to_string(),
            "-tune".to_string(),
            "zerolatency".to_string(),
            "-maxrate".to_string(),
            "4000k".to_string(),
            "-bufsize".to_string(),
            "8000k".to_string(),
            "-pix_fmt".to_string(),
            "yuv420p".to_string(),
            "-g".to_string(),
            "60".to_string(),
            "-c:a".to_string(),
            "aac".to_string(),
            "-b:a".to_string(),
            "128k".to_string(),
            "-f".to_string(),
            "flv".to_string(),
            rtmp_url,
        ]);
    }

    ffmpeg_args.extend(vec![
        "-c:v".to_string(),
        "libx264".to_string(),
        "-preset".to_string(),
        "veryfast".to_string(),
        "-tune".to_string(),
        "zerolatency".to_string(),
        "-crf".to_string(),
        "18".to_string(),
        "-pix_fmt".to_string(),
        "yuv420p".to_string(),
        "-g".to_string(),
        "15".to_string(),
        "-keyint_min".to_string(),
        "15".to_string(),
        "-sc_threshold".to_string(),
        "0".to_string(),
        "-c:a".to_string(),
        "aac".to_string(),
        "-b:a".to_string(),
        "128k".to_string(),
        "-f".to_string(),
        "hls".to_string(),
        "-hls_time".to_string(),
        "0.5".to_string(),
        "-hls_list_size".to_string(),
        "3".to_string(),
        "-hls_flags".to_string(),
        "delete_segments+independent_segments".to_string(),
        "-hls_segment_type".to_string(),
        "fmp4".to_string(),
        "-hls_fmp4_init_filename".to_string(),
        "init.mp4".to_string(),
        hls_path.to_str().unwrap().to_string(),
    ]);

    let sidecar_command = app
        .shell()
        .sidecar("ffmpeg")
        .map_err(|e| e.to_string())?
        .args(ffmpeg_args)
        .current_dir(hls_dir);

    let (mut rx, child) = sidecar_command.spawn().map_err(|e| e.to_string())?;

    *preview_state.0.lock().unwrap() = Some(child);

    tauri::async_runtime::spawn(async move {
        while let Some(event) = rx.recv().await {
            match event {
                CommandEvent::Stdout(_line) | CommandEvent::Stderr(_line) => {
                    // let log = String::from_utf8_lossy(&line).to_string();
                    // let _ = app.emit("ffmpeg-log", &log);
                }
                _ => {}
            }
        }
    });

    Ok("Stream initialization dispatched.".to_string())
}
