use std::{fs, io};

pub fn delete_files_in_dir(path: &str) -> io::Result<()> {
    if let Ok(entries) = fs::read_dir(path) {
        for entry in entries.flatten() {
            // Check if it's a file
            if entry.metadata()?.is_file() {
                // Delete the file
                fs::remove_file(entry.path())?;
            }
        }
    }
    Ok(())
}
