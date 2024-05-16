mod utils;
use rustube::{Callback, CallbackArguments};
use utils::parse_yt_link;

mod downloader;

use std::sync::mpsc;
use std::{
    num::NonZeroUsize,
    sync::{Arc, Mutex},
    thread,
    time::Duration,
};
use tiny_http::{Response, Server};
use std::fs;
use std::path::Path;
use redis::Commands;
use std::fs::rename;

extern crate redis;

enum Message {
    NewJob(String),
}

fn generate_directory_listing(path: &Path) -> String {
    let mut listing = String::new();
    listing.push_str("<html><body><ul>");

    if let Ok(entries) = fs::read_dir(path) {
        for entry in entries {
            if let Ok(entry) = entry {
                let file_name = entry.file_name().into_string().unwrap();
                listing.push_str(&format!("<li><a href=\"files/{}\">{}</a></li>", file_name, file_name));
            }
        }
    }

    listing.push_str("</ul></body></html>");
    listing
}

fn main() {
    let (tx, rx) = mpsc::channel::<Message>();

    let client = redis::Client::open("redis://:pass@0.0.0.0:6379").expect("Error");
    let conn = Arc::new(Mutex::new(client.get_connection().expect("Error 2")));
    let conn2 = conn.clone();
    const LIST_KEY: &str = "yt-jobs";

    thread::spawn(move || {
        let server = Server::http("127.0.0.1:8088").unwrap();

        for mut request in server.incoming_requests() {
            match request.url() {
                url if url.starts_with("/files") => match request.method() {
                    tiny_http::Method::Get => {
                        let url = request.url().trim_start_matches("/files");
                        let url = format!("./storage{}", url);
                        let path = Path::new(&url);
                        println!("Request: {}", url);

                        if path.is_dir() {
                            let dir_listing = generate_directory_listing(path);
                            let response = Response::from_string(dir_listing).with_header(
                                "Content-Type: text/html".parse::<tiny_http::Header>().unwrap()
                            );
                            request.respond(response).unwrap();
                        } else if path.is_file() {
                            let file_content = fs::read(path).unwrap();
                            let response = Response::from_data(file_content).with_header(
                                "Content-Type: application/octet-stream".parse::<tiny_http::Header>().unwrap()
                            );
                            request.respond(response).unwrap();
                        } else {
                            let response = Response::from_string("404 Not Found")
                                .with_status_code(404)
                                .with_header("Content-Type: text/plain".parse::<tiny_http::Header>().unwrap());
                            request.respond(response).unwrap();
                        }
                    },
                    _ => {
                        let error = Response::new(
                            tiny_http::StatusCode(405),
                            Vec::new(),
                            "method not allowed".as_bytes(),
                            None,
                            None,
                        );
                        request.respond(error).unwrap();
                    }
                },
                "/jobs" => match request.method() {
                    tiny_http::Method::Get => match conn.try_lock() {
                        Ok(mut conn) => {
                            let job_count = (*conn).llen::<&str, usize>(LIST_KEY).unwrap();
                            let response = Response::from_string(job_count.to_string());
                            request.respond(response).unwrap();
                        }
                        Err(_) => {
                            let error = Response::new(
                                tiny_http::StatusCode(500),
                                Vec::new(),
                                "internal server error".as_bytes(),
                                None,
                                None,
                            );
                            request.respond(error).unwrap();
                        }
                    },
                    tiny_http::Method::Post => {
                        let mut content = String::new();
                        request.as_reader().read_to_string(&mut content).unwrap();

                        match parse_yt_link(&content) {
                            Some(video_id) => {
                                tx.send(Message::NewJob(video_id)).unwrap();
                                request.respond(Response::from_string("got it")).unwrap();
                            }
                            None => {
                                let error = Response::new(
                                    tiny_http::StatusCode(400),
                                    Vec::new(),
                                    "invalid link".as_bytes(),
                                    None,
                                    None,
                                );
                                request.respond(error).unwrap();
                            }
                        }
                    }
                    _ => {
                        let error = Response::new(
                            tiny_http::StatusCode(405),
                            Vec::new(),
                            "method not allowed".as_bytes(),
                            None,
                            None,
                        );
                        request.respond(error).unwrap();
                    }
                },
                _ => {
                    let error = Response::new(
                        tiny_http::StatusCode(404),
                        Vec::new(),
                        "not found".as_bytes(),
                        None,
                        None,
                    );
                    request.respond(error).unwrap();
                }
            }
        }
    });

    loop {
        match rx.try_recv() {
            Ok(result) => match result {
                Message::NewJob(video_id) => {
                    match conn2.try_lock() {
                        Ok(mut conn) => {
                            conn.lpush::<&str, &str, i8>(LIST_KEY, video_id.as_str())
                                .unwrap();
                            println!("New job: {}", video_id);
                        }
                        Err(_) => {
                            println!("Failed to lock redis connection");
                        }
                    };
                }
            },
            Err(_) => {
                match conn2.try_lock() {
                    Ok(mut conn) => {
                        let job_count = conn.llen::<&str, usize>(LIST_KEY).unwrap();

                        if job_count > 0 {
                            match conn.lpop::<&str, Vec<String>>(LIST_KEY, NonZeroUsize::new(1)) {
                                Ok(job_info) => {
                                    println!("Job: {}", job_info[0]);

                                    let cb = Callback::new()
                                    .connect_on_progress_closure(move |args: CallbackArguments| {
                                        match args.content_length {
                                            Some(length) => {
                                                println!("Downloading: ({}/{})", args.current_chunk, length);
                                            },
                                            None => (),
                                        }
                                    });

                                    match downloader::download(job_info[0].clone(), Some(cb)) {
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
                                }
                                Err(err) => println!("Failed to fetch next job info: {}", err),
                            }
                        }

                        println!("Current Job count is: {}", job_count);
                    }
                    Err(_) => {
                        println!("Failed to lock redis connection");
                    }
                };

                thread::sleep(Duration::from_secs(1));
            }
        }
    }
}
