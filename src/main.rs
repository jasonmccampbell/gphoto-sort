extern crate clap;
extern crate walkdir;

use clap::{App, Arg};
use std::path::*;
use walkdir::{DirEntry, WalkDir};

fn main() {
    let matches = App::new("GPhoto-Sort")
        .version("0.1.0")
        .about("GPhoto-Sort: Moves photo's from Google Takeout structure into Google Docs/Google Photos directory structure")
        .arg(
            Arg::with_name("source-takeout-dir")
                .takes_value(true)
                .required(true)
                .help("Takeout directory containing the files to be moved"),
        )
        .arg(
            Arg::with_name("dest-gphotos-dir")
                .takes_value(true)
                .required(true)
                .help("Destination Goole Photos directory organized by <year>/<month>"),
        )
        .get_matches();
    let src = std::path::Path::new(matches.value_of("source-takeout-dir").unwrap());
    let dst = std::path::Path::new(matches.value_of("dest-gphotos-dir").unwrap());
    std::process::exit({
        if !validate_takeout_dir(src) || !validate_gphoto_dir(dst) {
            1
        } else if !move_files(src, dst) {
            1
        } else {
            1
        }
    });
}

/// Bulk of the code: recursively walks the source directory looking for image and video files and ignoring the rest of the
/// rubbish / metadata files. And found files are checked against the destination directory and moved over if they don't
/// already exist.
fn move_files(src: &Path, dst: &Path) -> bool {
    for entry in WalkDir::new(src.join("Google Photos"))
        .into_iter()
        .filter_entry(|e| e.file_name().to_str().map_or(false, |s| !s.starts_with("Hangout")))
        .filter_map(|e| e.ok())
        .filter(|e| !e.path().is_dir())
        .filter(is_of_interest)
    {
        let path = entry.path();
        if let Some(filename) = path.file_name() {
            let filename_str = filename.to_string_lossy();
            if let Some((year, month)) = extract_year_month(&filename_str) {
                move_or_delete(&path, dst.join(year).join(month).join(filename));
            }
        }
    }
    true
}

fn move_or_delete(src: &Path, dst: PathBuf) {
    println!("Moving {} to {}", src.display(), dst.display());
}

fn extract_year_month(filename: &str) -> Option<(&str, &str)> {
    if filename.starts_with("IMG_") || filename.starts_with("VID_") {
        Some((&filename[4..8], &filename[8..10]))
    } else {
        None
    }
}

/// Flags files of interest, like actual images/videos, and rejects the metadata
fn is_of_interest(entry: &DirEntry) -> bool {
    match entry.path().extension() {
        None => false,
        Some(ext) => ext.eq("jpg") || ext.eq("mp4"),
    }
}

/// Just verifies that we can read the source directory and that the structure is sort of what we expect
fn validate_takeout_dir(src: &Path) -> bool {
    if !src.exists() {
        println!(
            "Error: Google takeout source directory '{}' does not exist or is not readable",
            src.to_string_lossy()
        );
        false
    } else if !src.join("Google Photos").exists() {
        println!("Error: expected to find directory 'Google Photos' under Takeout directory, path may be incorrect");
        false
    } else {
        true
    }
}

/// Validates that the destination exists at least
/// TODO: Validate writability?
fn validate_gphoto_dir(dst: &Path) -> bool {
    if !dst.exists() {
        println!(
            "Error: Google Photos destination directory '{}' does not exist or is not writable",
            dst.to_string_lossy()
        );
        false
    } else {
        true
    }
}
