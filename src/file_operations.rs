use chrono::{DateTime, Local};
use std::{fs, io, path::Path};

pub fn copy_directory(src: &Path, dst: &Path) -> io::Result<()> {
    println!("Attempting to copy from {:?} to {:?}", src, dst);
    let local: DateTime<Local> = Local::now();
    let timestamp = local.format("%d.%m.%Y %H.%M.%S").to_string(); // Ensure no illegal characters for file paths
    let dst_with_timestamp = dst.join(timestamp);
    println!("Creating directory: {:?}", dst_with_timestamp);

    fs::create_dir_all(&dst_with_timestamp)?;

    // Recursively copy all contents from src to the new destination directory
    let result = copy_contents_recursively(src, src, &dst_with_timestamp);
    if result.is_ok() {
        eprintln!("Backup completed successfully");
    } else {
        let err_msg = format!("Failed to copy directory: {:?}", result.unwrap_err());
        eprintln!("Backup Error: {}", err_msg);
        // Return the error with details
        return Err(io::Error::new(io::ErrorKind::Other, err_msg));
    }

    result
}

/// Recursively copies contents from the source directory to the destination directory, maintaining the structure.
fn copy_contents_recursively(base: &Path, src: &Path, dst: &Path) -> io::Result<()> {
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let path = entry.path();
        // Get the relative path with respect to the base
        let relative_path = path.strip_prefix(base).unwrap();
        let destination_path = dst.join(relative_path);

        if entry.file_type()?.is_dir() {
            fs::create_dir_all(&destination_path)?;
            // Recursive call to handle subdirectories
            copy_contents_recursively(base, &path, &destination_path)?;
        } else {
            if let Some(parent) = destination_path.parent() {
                fs::create_dir_all(parent)?; // Ensure the directory exists
            }
            // println!("Copying file {:?} to {:?}", path, destination_path);
            fs::copy(&path, &destination_path)?;
        }
    }
    Ok(())
}
