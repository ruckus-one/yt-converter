use inquire::Text;

fn main() {
    let ytId = Text::new("Provide YT ID -> ").prompt();
    // x1U1Ue_5kq8
    match ytId {
        Ok(ytId) => {
            let url = format!("https://www.youtube.com/watch?v={}", ytId);

            match rustube::blocking::download_worst_quality(url.as_str()) {
                Ok(path) => {
                    match path.to_str() {
                        Some(path) => println!("Your video is here: {}", path),
                        None => println!("Error getting video path"),

                    }
                },
                Err(_) => println!("Error getting video path"),
            }
            
        },
        Err(_) => println!("Something went wrong."),
    }
}
