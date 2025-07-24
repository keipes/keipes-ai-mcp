use std::time::{SystemTime, UNIX_EPOCH};

fn main() {
    let now = SystemTime::now();
    let duration = now.duration_since(UNIX_EPOCH).expect("Time went backwards");
    eprintln!("cargo:rustc-env=BUILD_TIME={}", duration.as_millis());
}
