// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use core::str;
use regex::Regex;
use std::{
    env,
    fmt::Debug,
    path::{Path, PathBuf},
    process::{Command, Stdio},
    time::{SystemTime, UNIX_EPOCH},
};
mod utils;
#[derive(Debug)]
struct VideoStream {
    width: u32,
    height: u32,
    codec_name: String,
    bit_rate: u64,
    nb_frames: u64,
    r_frame_rate: (u32, u32),
    avg_frame_rate: (u32, u32),
}
#[derive(Debug)]
struct Format {
    duration: f64,
    file_size: u64,
}
#[derive(Debug)]
struct VideoInfo {
    file: String,
    format: Format,
    video: VideoStream,
}

use ffmpeg_sidecar::{command::FfmpegCommand, event::FfmpegEvent};
// 计算预计完成时间
fn calculate_remaining_time(progress_frame: u32, total_frames: u64, current_fps_speed: f32) -> f64 {
    // 计算每秒处理的帧数（FPS）
    (total_frames as f64 - progress_frame as f64) / current_fps_speed as f64
}
fn get_video_info(video_path: &Path) -> Result<VideoInfo, ()> {
    // 使用 `ffprobe` 命令获取视频信息
    let output = Command::new("ffprobe")
        .arg("-v")
        .arg("quiet")
        .arg("-show_format")
        .arg("-show_streams")
        .arg(video_path)
        .output()
        .expect("Failed to execute `ffprobe`");

    if !output.status.success() {
        return Err(());
    }

    // 解析 `ffprobe` 输出
    let stdout = str::from_utf8(&output.stdout).unwrap();
    let duration_re = Regex::new(r"^duration=(\d+\.\d+)").unwrap();
    let size_re = Regex::new(r"^size=(\d+)").unwrap();
    let width_re = Regex::new(r"^width=(\d+)").unwrap();
    let height_re = Regex::new(r"^height=(\d+)").unwrap();
    let codec_name_re = Regex::new(r"^codec_name=(\w+)").unwrap();
    let bit_rate_re = Regex::new(r"^bit_rate=(\d+)").unwrap();
    let nb_frames_re = Regex::new(r"^nb_frames=(\d+)").unwrap();
    let r_frame_rate_re = Regex::new(r"^r_frame_rate=(\d+)/(\d+)").unwrap();
    let avg_frame_rate_re = Regex::new(r"^avg_frame_rate=(\d+)/(\d+)").unwrap();

    let mut duration = 0.0;
    let mut file_size = 0;
    let mut width = 0;
    let mut height = 0;
    let mut codec_name = String::new();
    let mut bit_rate = 0;
    let mut nb_frames = 0;
    let mut r_frame_rate = (0, 0);
    let mut avg_frame_rate = (0, 0);
    // 解析 `ffprobe` 输出
    let stdout = match str::from_utf8(&output.stdout) {
        Ok(s) => s,
        Err(_) => return Err(()),
    };

    for line in stdout.lines() {
        if let Some(caps) = duration_re.captures(line) {
            duration = caps
                .get(1)
                .map_or(0.0, |m| m.as_str().parse().unwrap_or(0.0));
        }
        if let Some(caps) = size_re.captures(line) {
            file_size = caps.get(1).map_or(0, |m| m.as_str().parse().unwrap_or(0));
        }
        if let Some(caps) = width_re.captures(line) {
            width = caps.get(1).map_or(0, |m| m.as_str().parse().unwrap_or(0));
        }
        if let Some(caps) = height_re.captures(line) {
            height = caps.get(1).map_or(0, |m| m.as_str().parse().unwrap_or(0));
        }
        if let Some(caps) = codec_name_re.captures(line) {
            codec_name = caps
                .get(1)
                .map_or(String::new(), |m| m.as_str().to_string());
        }
        if let Some(caps) = bit_rate_re.captures(line) {
            bit_rate = caps.get(1).map_or(0, |m| m.as_str().parse().unwrap_or(0));
        }
        if let Some(caps) = nb_frames_re.captures(line) {
            nb_frames = caps.get(1).map_or(0, |m| m.as_str().parse().unwrap_or(0));
        }
        if let Some(caps) = r_frame_rate_re.captures(line) {
            r_frame_rate = (
                caps.get(1).map_or(0, |m| m.as_str().parse().unwrap_or(0)),
                caps.get(2).map_or(0, |m| m.as_str().parse().unwrap_or(0)),
            );
        }
        if let Some(caps) = avg_frame_rate_re.captures(line) {
            avg_frame_rate = (
                caps.get(1).map_or(0, |m| m.as_str().parse().unwrap_or(0)),
                caps.get(2).map_or(0, |m| m.as_str().parse().unwrap_or(0)),
            );
        }
    }

    Ok(VideoInfo {
        file: video_path
            .file_name()
            .unwrap()
            .to_str()
            .unwrap()
            .to_string(),
        format: Format {
            duration,
            file_size,
        },
        video: VideoStream {
            width,
            height,
            codec_name,
            bit_rate,
            nb_frames,
            r_frame_rate,
            avg_frame_rate,
        },
    })
}
// Learn more about Tauri commands at https://tauri.app/v1/guides/features/command
#[tauri::command]
fn towebp(name: &str) -> Result<String, String> {
    let file_path = Path::new(name);
    if !(file_path.exists() && file_path.is_file()) {
        return Ok("File not found".to_string());
    }
    let binding = file_path.parent().unwrap().join("out");
    let new_path = binding.as_path();
    if !new_path.exists() {
        std::fs::create_dir(new_path).unwrap();
    }
    let new_name = new_path.join(format!(
        "{}.webp",
        file_path.file_stem().unwrap().to_str().unwrap()
    ));
    let img: ril::Image<ril::Rgb> = ril::prelude::Image::open(file_path).unwrap();
    let res = img.save_inferred(new_name);
    if res.is_ok() {
        return Ok("Image converted successfully".to_string());
    }
    let result = Command::new("explorer").args(&[new_path]).spawn();

    if let Err(err) = result {
        eprintln!("Failed to open folder: {}", err);
    }
    Ok(format!("Hello, {}! You've been to_webped from Rust!", name).to_string())
}
#[tauri::command]
fn testffmpeg(name: &str) -> Result<String, String> {
    let file_path = Path::new(name);
    println!("{:?}", file_path);

    if !(file_path.exists() && file_path.is_file()) {
        return Ok("File not found".to_string());
    }
    let binding: std::path::PathBuf = file_path.parent().unwrap().join("out");
    let info = match get_video_info(file_path) {
        Ok(info) => info,
        Err(_) => return Ok("Failed to get video info".to_string()),
    };
    println!("{:?}", info);
    println!("{:?}frames", info.video.nb_frames);
    let new_path = binding.as_path();
    if !new_path.exists() {
        std::fs::create_dir(new_path).unwrap();
    }
    let new_name = new_path.join(format!(
        "{}1.mp4",
        file_path.file_stem().unwrap().to_str().unwrap()
    ));
    let rate = 15000; // 设置你想要的比特率值，单位是 kbps

    let ffmpeg_args = [
        "-hwaccel",
        "d3d11va",
        "-i",
        file_path.to_str().unwrap(),
        "-c:v",
        "av1_amf",
        "-preset",
        "veryfast",
        "-b:v",
        &format!("{}k", rate),
        "-bufsize",
        &format!("{}k", rate * 2 * 2),
        "-rc:v",
        "cbr",
        "-vbaq",
        "true",
        "-c:a",
        "copy",
        "-threads",
        "16",
        new_name.to_str().unwrap(),
    ];

    // 获取当前系统时间
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis();
    println!("start time: {}", now);
    // return Ok("Hello, {}! You've been to_webped from Rust!".to_string());
    // 计算自 Unix 纪元以来的持续时间
    FfmpegCommand::new()
        .args(ffmpeg_args)
        .spawn()
        .unwrap()
        .iter()
        .unwrap()
        .for_each(|event: FfmpegEvent| {
            match event {
                FfmpegEvent::OutputFrame(frame) => {
                    println!("frame: {}x{}", frame.width, frame.height);
                    let _pixels: Vec<u8> = frame.data; // <- raw RGB pixels! 🎨
                }
                FfmpegEvent::Progress(progress) => {
                    let progress_num = utils::num::round_to_decimals(
                        progress.frame as f64 / info.video.nb_frames as f64,
                        2,
                    );

                    let progress_frame = progress.frame;
                    let total_frames = info.video.nb_frames;
                    let current_fps_speed = progress.fps;

                    // 计算剩余时间
                    let remaining_time = calculate_remaining_time(
                        progress_frame,
                        total_frames,
                        current_fps_speed
                    );
                    eprintln!(
                        "Current speed: {}x, time: {}, fps: {}, frame: {}, progressnum: {}, remaining time: {}",
                        progress.speed, progress.time, progress.fps, progress.frame, progress_num,
                        if remaining_time == f64::INFINITY{
                            0.0
                        } else {
                            remaining_time
                        }
                    ); // <- parsed progress updates
                }
                FfmpegEvent::Log(_level, msg) => {
                    eprintln!("[ffmpeg] {}", msg); // <- granular log message from stderr
                }
                _ => {}
            }
        });
    Ok("Hello, {}! You've been to_webped from Rust!".to_string())
}
#[tauri::command]
fn get_assets() -> Vec<String> {
    let mut assets = vec![];
    let path = env::current_dir();
    for entry in std::fs::read_dir(path.unwrap()).unwrap() {
        let entry = entry.unwrap();
        let file_name = entry.file_name();
        assets.push(file_name.to_str().unwrap().to_string());
    }
    return assets
        .join("||")
        .to_string()
        .split("||")
        .map(|s| s.to_string())
        .collect();
}
#[tauri::command]

fn get_last_assets() -> Vec<String> {
    let mut assets = vec![];
    let binding = env::current_dir().unwrap();
    let path = binding.parent();
    for entry in std::fs::read_dir(path.unwrap()).unwrap() {
        let entry = entry.unwrap();
        let file_name = entry.file_name();
        assets.push(file_name.to_str().unwrap().to_string());
    }
    return assets
        .join("||")
        .to_string()
        .split("||")
        .map(|s| s.to_string())
        .collect();
}
fn main() {
    let current_dir = env::current_dir().unwrap();
    println!("当前工作目录是: {:?}", current_dir);

    let ffmpeg_path: PathBuf;
    if cfg!(debug_assertions) {
        ffmpeg_path = current_dir.join("target\\debug\\_up_\\libs\\ffmpeg\\bin")
    } else {
        ffmpeg_path = current_dir.join("_up_\\libs\\ffmpeg\\bin")
    }
    println!("aaa:{}", ffmpeg_path.to_str().unwrap());
    env::set_var("Path", ffmpeg_path.to_str().unwrap());
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            towebp,
            testffmpeg,
            get_assets,
            get_last_assets
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
