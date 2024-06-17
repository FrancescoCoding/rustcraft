use std::env;
use std::fs::File;
use std::io::Write;
use std::path::Path;

fn main() {
    // Retrieve the output directory from the environment variables set by Cargo.
    let out_dir = env::var("OUT_DIR").expect("Could not get OUT_DIR");

    // Build the destination path for the manifest file.
    let dest_path = Path::new(&out_dir).join("Rustcraft.exe.manifest");

    // Create the file in the destination path.
    let mut f = File::create(dest_path).expect("Failed to create file");

    // Include the manifest file's bytes and write them to the destination file.
    f.write_all(include_bytes!("Rustcraft.exe.manifest"))
        .expect("Failed to write to file");
}
