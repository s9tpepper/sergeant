use psimple::Simple;
use pulse::sample::Format;
use pulse::sample::Spec;
use pulse::stream::Direction;
use rodio::Decoder;
use rodio::Source;

use crate::File;

use crate::BufReader;

use std::process::exit;

use crate::db::get_connection_pool;

pub async fn play(name: &str) -> anyhow::Result<()> {
    let pool = get_connection_pool().await?;
    let result = sqlx::query!(r#"SELECT file_path FROM intros WHERE name = ?1 AND approved = 1"#, name)
        .fetch_one(&pool)
        .await;

    if result.is_err() {
        let result = sqlx::query!(r#"SELECT approved FROM intros WHERE name = ?1"#, name)
            .fetch_one(&pool)
            .await;

        if result.is_err() {
            println!("No intro found for user: {:?}", name);
            exit(3);
        }

        println!("Intro not approved yet");
        exit(2);
    }

    let file = BufReader::new(File::open(result.unwrap().file_path)?);
    let source = Decoder::new(file)?.convert_samples::<f32>();

    let spec = Spec {
        format: Format::FLOAT32NE,
        channels: source.channels() as u8,
        rate: source.sample_rate(),
    };

    let sink = Simple::new(
        None,                  // Use the default server
        "intros",              // Our application’s name
        Direction::Playback,   // We want a playback stream
        Some("rodio.capture"), // Use the default device if failed
        "programmatic audio",  // Description of our stream
        &spec,                 // Our sample format
        None,                  // Use default channel map
        None,                  // Use default buffering attributes
    )
    .unwrap();

    let audio_data = source.into_iter().collect::<Vec<_>>();
    let audio = audio_data
        .iter()
        .flat_map(|&x| x.to_le_bytes().to_vec())
        .collect::<Vec<_>>();

    let audio_chunks = audio.chunks(1024);
    for chunk in audio_chunks {
        sink.write(chunk)?;
    }

    Ok(())
}
