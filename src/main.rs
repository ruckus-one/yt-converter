use std::fmt::Write;

use inquire::Text;
use rustube::{block, Callback, CallbackArguments, Error, Id, Video, Result};
use indicatif::{ ProgressBar, ProgressState, ProgressStyle };

async fn download(yt_id: String, cb: Callback) -> Result<std::path::PathBuf> {
    let id = Id::from_raw(&yt_id.as_str()).unwrap();

    Video::from_id(id.into_owned())
        .await?
        .best_audio()
        .ok_or(Error::NoStreams).unwrap()
        .download_to_dir_with_callback(".", cb)
        .await
}

fn main() {

    let progress = ProgressBar::new(1);

    progress.set_style(ProgressStyle::with_template("{spinner:.green} [{elapsed_precise}] [{wide_bar:.cyan/blue}] {bytes}/{total_bytes} ({eta})")
        .unwrap()
        .with_key("eta", |state: &ProgressState, w: &mut dyn Write| write!(w, "{:.1}s", state.eta().as_secs_f64()).unwrap())
        .progress_chars("#>-"));

    let cb = Callback::new()
        .connect_on_progress_closure(move |args: CallbackArguments| {
            match args.content_length {
                Some(length) => {
                    progress.set_length(length);
                    progress.set_position(args.current_chunk as u64);
                },
                None => (),
            }
        });

    let yt_id = Text::new("Provide YT ID -> ").prompt();

    match yt_id {
        Ok(yt_id) => {
            match block!(download(yt_id, cb)) {
                Ok(_) => print!("ok"),
                Err(_) => print!("err"),
            }
        },
        Err(_) => println!("Something went wrong."),
    }
}
