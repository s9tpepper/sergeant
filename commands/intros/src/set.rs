use std::{
    fs,
    io::Read,
    process::{Command, ExitStatus},
};

use essi_ffmpeg::{FFmpeg, FFmpegCommand};
use lib::fs::get_project_directory;
use serde::Deserialize;
use sqlx::sqlite::SqliteQueryResult;

use crate::db::get_connection_pool;

#[derive(Debug, Deserialize)]
struct TrackAnalysis {
    output_i: String,
    output_tp: String,
    output_lra: String,
    output_thresh: String,
    target_offset: String,
}

fn get_file_path(file_name: &str) -> anyhow::Result<String> {
    let path = get_project_directory("SetIntro", "audio")?;

    let path_err = anyhow::Error::msg("Could not build file path");
    Ok(format!("{}/{}.mp3", path.to_str().ok_or(path_err)?, file_name))
}

const HOUR: i32 = 60 * 60;
const MIN: i32 = 60;

fn get_seconds_from_time(time: &str) -> anyhow::Result<i32> {
    let mut split_parts = time.split(':');
    let split_options = (split_parts.next(), split_parts.next(), split_parts.next());

    #[allow(clippy::single_match)]
    match split_options {
        (Some(hours), Some(mins), Some(seconds)) => {
            let hours_seconds = hours.parse::<i32>()? * HOUR;
            let minute_seconds = mins.parse::<i32>()? * MIN;
            let seconds = seconds.parse::<i32>()?;

            Ok(hours_seconds + minute_seconds + seconds)
        }

        _ => Err(anyhow::Error::msg("Invalid time input")),
    }
}

fn get_start_end(start: &str, end: &str) -> anyhow::Result<(String, String)> {
    let start_seconds = get_seconds_from_time(start)?;
    let end_seconds = get_seconds_from_time(end)?;

    let mut end_time = end.to_string();
    if (end_seconds - start_seconds) > 8 {
        end_time = end.replace(
            end_seconds.to_string().as_str(),
            (start_seconds + 8).to_string().as_str(),
        );
    }

    Ok((start_seconds.to_string(), end_time))
}

fn run_yt_dlp(url: &str, download_args: &str, file_path: &str) -> Result<ExitStatus, std::io::Error> {
    Command::new("yt-dlp")
        .arg("--external-downloader")
        .arg("ffmpeg")
        .arg("--external-downloader-args")
        .arg(download_args)
        .arg("--extract-audio")
        .arg("--audio-format")
        .arg("mp3")
        .arg(url)
        .arg("-o")
        .arg(file_path)
        .status()
}

async fn insert_intro(name: &str, file_path: &str, message: &str) -> anyhow::Result<SqliteQueryResult> {
    let pool = get_connection_pool().await?;
    let mut connection = pool.acquire().await?;
    Ok(sqlx::query!(
                    r#"INSERT INTO intros (name, file_path, message, approved) VALUES (?1, ?2, ?3, 0)
                        ON CONFLICT(name) DO UPDATE SET file_path = excluded.file_path, message = excluded.message, approved = 0
                    "#,
                    name,
                    file_path,
                    message
                )
                .execute(&mut *connection).await?)
}

async fn download_ffmpeg() {
    // Automatically download FFmpeg if not found
    if let Some((handle, _)) = FFmpeg::auto_download().await.unwrap() {
        handle.await.unwrap().unwrap();
    }
}

pub async fn set(url: &str, start: &str, end: &str, file_name: &str, name: &str) -> anyhow::Result<()> {
    download_ffmpeg().await;

    let file_path = get_file_path(file_name)?;
    let exit_status = get_start_end(start, end)
        .map(|(start, end)| format!("-ss {start} -to {end}"))
        .and_then(|download_args| Ok(run_yt_dlp(url, &download_args, &file_path)?));

    match exit_status {
        Ok(status) => match status.success() {
            true => Ok(insert_intro(name, &file_path, "").await?),
            false => Err(anyhow::Error::msg("Could not insert")),
        },

        Err(error) => Err(error),
    }?;

    post_process(&file_path)?;

    Ok(())
}

fn get_ffmpeg_json(output: &str) -> anyhow::Result<TrackAnalysis> {
    let left_brace = output.find("{");
    let right_brace = output.find("}");

    #[allow(clippy::single_match)]
    match [left_brace, right_brace] {
        [Some(left), Some(right)] => {
            let json = &output[left..=right];

            Ok(serde_json::from_str::<TrackAnalysis>(json)?)
        }

        _ => Err(anyhow::Error::msg("Error parsing FFMpeg analysis JSON")),
    }
}

fn post_process(file_path: &str) -> anyhow::Result<()> {
    let mut ffmpeg = FFmpeg::new()
        .args(["-af", "loudnorm=print_format=json", "-f", "null", "-"])
        .input_with_file(file_path.into())
        .done()
        .output_as_file("/tmp/temp.mp3".into())
        .done()
        .start()
        .unwrap();

    let mut stdout_ref = ffmpeg
        .take_stderr()
        .ok_or(anyhow::Error::msg("Could not get ffmpeg analysis output"))?;

    ffmpeg.wait()?;

    let mut output = String::new();
    stdout_ref.read_to_string(&mut output)?;

    let track_analysis = get_ffmpeg_json(&output)?;
    let mut ffmpeg_command = normalize_track(file_path, track_analysis)?;
    ffmpeg_command.wait()?;

    Ok(())
}

fn normalize_track(file_path: &str, track_analysis: TrackAnalysis) -> anyhow::Result<FFmpegCommand> {
    file_path
        .split("/")
        .last()
        .map(|file_name| file_path.replace(file_name, format!("og_{file_name}").as_str()))
        .map(|og_file| match fs::rename(file_path, og_file.clone()) {
            Ok(_) => (og_file, file_path.to_string()),
            Err(_) => (file_path.to_string(), file_path.to_string()),
        })
        .map(|(og_file, new_file)| {
            let TrackAnalysis {
                output_i,
                output_lra,
                output_tp,
                output_thresh,
                target_offset,
                ..
            } = track_analysis;

            let i = format!("measured_I={output_i}");
            let lra = format!("measured_LRA={output_lra}");
            let tp = format!("measured_TP={output_tp}");
            let thresh = format!("measured_thresh={output_thresh}");
            let offset = format!("offset={target_offset},volume=0.15");
            let analysis_fields = [
                "loudnorm=I=-16",
                "TP=-1.5",
                "LRA=11",
                i.as_str(),
                lra.as_str(),
                tp.as_str(),
                thresh.as_str(),
                offset.as_str(),
            ];

            let options = &analysis_fields.join(":");
            let normalize_options = options.to_string();

            FFmpeg::new()
                .args(["-af", &normalize_options, &new_file])
                .input_with_file(og_file.into())
                .done()
                .output_as_file(new_file.into())
                .done()
                .start()
        })
        .unwrap()
}
