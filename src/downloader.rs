use rustube::{block, Callback, Error, Id, Result, Video};

pub fn download(yt_id: String, cb: Option<Callback>) -> Result<std::path::PathBuf> {
    let id = Id::from_raw(&yt_id.as_str()).unwrap();

    match cb {
        Some(cb) => {
            block!(Video::from_id(id.into_owned())
            .await?
            .best_audio()
            .ok_or(Error::NoStreams).unwrap()
            .download_to_dir_with_callback("./storage", cb))
        },
        None => {
            block!(Video::from_id(id.into_owned())
            .await?
            .best_audio()
            .ok_or(Error::NoStreams).unwrap()
            .download_to_dir("./storage"))
        },
    }
}
