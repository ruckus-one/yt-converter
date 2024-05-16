use regex::Regex;

pub fn parse_yt_link(url: &str) -> Option<String> {
    if url.contains("youtu.be/") {
        let video_id = url.split('/').last().unwrap();
        return Some(video_id.to_string());
    } else if url.contains("youtube.com/watch?v=") {
        let video_id = url.split("?v=").last().unwrap();
        return Some(video_id.to_string());
    }
    

    let pattern = Regex::new(r"^[a-zA-Z0-9_]{11}$").unwrap();

    if let Some(captures) = pattern.captures(url) {
        return Some(String::from(captures.get(0).map_or("nodata", |c| c.as_str()))); 
    } else {
        return None;
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_parse_yt_link_valid() {
        let video_id = parse_yt_link("https://youtu.be/abc123").unwrap();
        assert_eq!(video_id, "abc123");
        
        let video_id = parse_yt_link("https://www.youtube.com/watch?v=xyz456xyz45").unwrap();
        assert_eq!(video_id, "xyz456xyz45");
        
        let video_id = parse_yt_link("R_sot9ZK8X8").unwrap();
        assert_eq!(video_id, "R_sot9ZK8X8");
    }

    #[test]
    fn test_parse_yt_link_invalid() {
        let video_id = parse_yt_link("https://example.com/video");
        assert!(video_id.is_none());
    }
}