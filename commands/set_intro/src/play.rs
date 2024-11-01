use rodio::Decoder;

use crate::File;

use crate::BufReader;

use rodio::OutputStream;
use rodio::Sink;

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

    let (_stream, stream_handle) = OutputStream::try_default()?;
    let sink = Sink::try_new(&stream_handle).unwrap();

    let file = BufReader::new(File::open(result.unwrap().file_path)?);
    let source = Decoder::new(file)?;
    sink.append(source);
    sink.set_volume(0.25);
    sink.sleep_until_end();

    Ok(())
}
