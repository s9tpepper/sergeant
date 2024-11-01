use std::{
    env,
    fs::{self, File},
    io::Read,
};

use directories::ProjectDirs;
use essi_ffmpeg::*;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct TrackAnalysis {
    output_i: String,
    output_tp: String,
    output_lra: String,
    output_thresh: String,
    target_offset: String,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = env::args().collect::<Vec<_>>();

    // Automatically download FFmpeg if not found
    if let Some((handle, _)) = FFmpeg::auto_download().await.unwrap() {
        handle.await.unwrap().unwrap();
    }

    #[allow(clippy::single_match)]
    match &args[..] {
        [_, twitch_name, url, start, end] => {
            println!("{twitch_name}, {url}, {start}, {end}");

            if let Some(project_directories) = ProjectDirs::from("com", "s9tpepper", "SetIntro") {
                let mut data_directory = project_directories.data_dir().to_path_buf();
                let path = "audio";
                data_directory.push(path);

                if !data_directory.exists() {
                    std::fs::create_dir_all(&data_directory)?;
                }

                let file_path = format!("{}/test_file.mp3", data_directory.to_str().unwrap());
                let file_path = file_path.as_str();
                let _ = File::create(file_path);

                // TODO: Fix the path of the mp3 to what it should actually be from set_intro
                // (which should have its name fixed, extract_audio maybe)
                let mp3_path = "/Volumes/documents/stream/chatbot/audio/test_file.mp3";

                let mut ffmpeg = FFmpeg::new()
                    .args(["-af", "loudnorm=print_format=json", "-f", "null", "-"])
                    .input_with_file(mp3_path.into())
                    .done()
                    // .output_as_file("/Volumes/documents/stream/chatbot/audio/analyzed_test_file.mp3".into())
                    .output_as_file("/tmp/temp.mp3".into())
                    .done()
                    .start()
                    .unwrap();

                if let Some(ref mut stdout_ref) = ffmpeg.take_stderr() {
                    ffmpeg.wait().unwrap();
                    let mut output = String::new();
                    stdout_ref.read_to_string(&mut output)?;

                    let left_brace = output.find("{");
                    let right_brace = output.find("}");
                    #[allow(clippy::single_match)]
                    match [left_brace, right_brace] {
                        [Some(left), Some(right)] => {
                            let json = &output[left..=right];
                            println!("Json: {json}");

                            let results = serde_json::from_str::<TrackAnalysis>(json)
                                .map(|track_analysis| normalize_track(mp3_path, track_analysis))
                                .map_err(|error| {
                                    println!("error: {error}");
                                })
                                .unwrap();

                            match results {
                                Ok(mut ffmpeg_command) => {
                                    ffmpeg_command.wait().unwrap();
                                    // if let Some(ref mut stdout_ref) = ffmpeg_command.take_stderr() {
                                    //     ffmpeg_command.wait().unwrap();
                                    //     let mut output = String::new();
                                    //     stdout_ref.read_to_string(&mut output)?;
                                    //
                                    //     println!("Normalize output: {output}");
                                    // } else {
                                    //     println!("Could not get stderr");
                                    // }
                                    println!("Success normalizing track");
                                }
                                Err(error) => {
                                    dbg!(error);
                                }
                            };
                        }

                        _ => {}
                    }
                }
            }
        }

        _ => {}
    };

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
            println!("og_file: {og_file}, new_file: {new_file}");

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
