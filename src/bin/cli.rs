use std::fmt::Write;
use std::fs::rename;
use inquire::Text;
use rustube::{block, Callback, CallbackArguments, Error, Id, Result, Video};
use indicatif::{ ProgressBar, ProgressState, ProgressStyle };

extern crate yt_downloader;
use yt_downloader::downloader;

fn main() {
    match std::fs::create_dir_all("./storage") {
        Ok(_) => println!("Created ./storage directory."),
        Err(_) => println!("Error creating ./storage directory"),
    }

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
            match (downloader::download(yt_id, Some(cb))) {
                Ok(path) => {
                    match path.to_str() {
                        Some(path) => {
                            let new_path = path
                                .replace(".mp4", ".mp3")
                                .replace(".webm", ".mp3");

                            match rename(path, &new_path) {
                                Ok(_) =>  println!("Done! -> {}", new_path),
                                Err(err) => println!("Something went wrong. {}", err),
                            }
                        },
                        None => println!("Something went wrong."),
                    }
                },
                Err(_) => println!("Something went wrong."),
            }
        },
        Err(_) => println!("Something went wrong."),
    }
}
