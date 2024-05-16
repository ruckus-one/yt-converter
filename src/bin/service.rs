use std::{num::NonZeroUsize, sync::{Arc, Mutex}, thread, time::Duration};
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
    let conn = Arc::new(Mutex::new(client.get_connection().expect("Error 2")));
    let conn2 = conn.clone();
    const LIST_KEY: &str = "yt-jobs";

    thread::spawn(move || {
        let server = Server::http("127.0.0.1:8088").unwrap();

        for mut request in server.incoming_requests() {
            match request.url() {
                "/jobs" => {
                    match request.method() {
                        tiny_http::Method::Get => {
                            match conn.try_lock() {
                                Ok(mut conn) => {
                                    let job_count = (*conn).llen::<&str, usize>(LIST_KEY).unwrap();
                                    let response = Response::from_string(job_count.to_string());
                                    request.respond(response).unwrap();
                                },
                                Err(_) => {
                                    let error = Response::new(tiny_http::StatusCode(500), Vec::new(), "internal server error".as_bytes(), None, None);
                                    request.respond(error).unwrap();
                                }
                            }
                        },
                        tiny_http::Method::Post => {
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
                        },
                        _ => {
                            let error = Response::new(tiny_http::StatusCode(405), Vec::new(), "method not allowed".as_bytes(), None, None);
                            request.respond(error).unwrap();
                        },
                    }
                },
                _ => {
                    let error = Response::new(tiny_http::StatusCode(404), Vec::new(), "not found".as_bytes(), None, None);
                    request.respond(error).unwrap();
                },
            }
          }
    });

    loop {
        match rx.try_recv() {
            Ok(result) => {
                match result {
                    Message::NewJob(video_id) => {
                        match conn2.try_lock() {
                            Ok(mut conn) => {
                                conn.lpush::<&str, &str, i8>(LIST_KEY, video_id.as_str()).unwrap();
                                println!("New job: {}", video_id);
                            },
                            Err(_) => {
                                println!("Failed to lock redis connection");
                            }
                        };
                    },
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
                                }
                                Err(err) => println!("Failed to fetch next job info: {}", err),
                            }
                        }
                
                        println!("Current Job count is: {}", job_count);  
                    },
                    Err(_) => {
                        println!("Failed to lock redis connection");
                    }
                };
  
                thread::sleep(Duration::from_secs(1));
            },
        }
    }
}


