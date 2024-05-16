### What the heck?

I want to become somewhat proficient in Rust.

Using some awesome crates to create a tailored YT -> mp3 converter for no commercial purpose.


### Converting mp4/webm into mp3
```
mkdir storage
ffmpeg -i ./storage/example-id.mp4 -c:a libmp3lame ./storage/example-id.mp3
```