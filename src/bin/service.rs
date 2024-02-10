use std::{num::NonZeroUsize, thread, time::Duration};
use tiny_http::{Server, Response}; 
use link_parser::parse_yt_link;
use std::sync::mpsc;

use redis::Commands;

extern crate redis;

enum Message {
    NewJob(String),
}

fn main() {
    let (tx, rx) = mpsc::channel::<Message>();

    let client = redis::Client::open("redis://:pass@0.0.0.0:6379").expect("Error");
    let mut conn = client.get_connection().expect("Error 2");
    const LIST_KEY: &str = "yt-jobs";

    thread::spawn(move || {
        let server = Server::http("127.0.0.1:8088").unwrap();

        for mut request in server.incoming_requests() {

            let mut content = String::new();
            request.as_reader().read_to_string(&mut content).unwrap();
            
            match parse_yt_link(&content) {
                Some(video_id) => {
                    tx.send(Message::NewJob(video_id)).unwrap();
                    request.respond(Response::from_string("got it")).unwrap();
                },
                None => {
                    let error = Response::new(tiny_http::StatusCode(400), Vec::new(), "invalid link".as_bytes(), None, None);
                    request.respond(error).unwrap();
                }
            }
          }
    });

    loop {
        match rx.try_recv() {
            Ok(result) => {
                match result {
                    Message::NewJob(video_id) => {
                        conn.lpush::<&str, &str, i8>(LIST_KEY, video_id.as_str()).unwrap();
                        println!("New job: {}", video_id);
                    },
                }
            },
            Err(_) => {
                let job_count = conn.llen::<&str, usize>(LIST_KEY).unwrap();

                if job_count > 0 {
                    match conn.lpop::<&str, Vec<String>>(LIST_KEY, NonZeroUsize::new(1)) {
                        Ok(job_info) => {
                            println!("Job: {}", job_info[0]);
                        }
                        Err(err) => println!("Failed to fetch next job info: {}", err),
                    }
                }
        
                println!("Current Job count is: {}", job_count);    
                thread::sleep(Duration::from_secs(1));
            },
        }
    }
}

