use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

fn main() {
    let now = SystemTime::now();
    let duration = now.duration_since(UNIX_EPOCH).expect("Time went backwards");
    eprintln!("cargo:rustc-env=BUILD_TIME={}", duration.as_millis());

    // Generate FlatBuffers code
    generate_flatbuffers();
}

fn generate_flatbuffers() {
    let schema_path = "schemas/test.fbs";
    let output_dir = "src/generated";

    // Tell cargo to rerun this build script if the schema changes
    println!("cargo:rerun-if-changed={}", schema_path);

    // Create output directory if it doesn't exist
    std::fs::create_dir_all(output_dir).expect("Failed to create output directory");

    // Run flatc to generate Rust code
    let output = Command::new("flatc")
        .args(&["--rust", "-o", output_dir, schema_path])
        .output();

    match output {
        Ok(output) => {
            if !output.status.success() {
                eprintln!("flatc failed:");
                eprintln!("stdout: {}", String::from_utf8_lossy(&output.stdout));
                eprintln!("stderr: {}", String::from_utf8_lossy(&output.stderr));
                panic!("FlatBuffers code generation failed");
            }
        }
        Err(e) => {
            eprintln!(
                "Warning: Could not run flatc ({}). FlatBuffers code generation skipped.",
                e
            );
            eprintln!("Make sure flatc is installed and in your PATH.");
            eprintln!("You can install it from: https://github.com/google/flatbuffers/releases");
        }
    }
}
