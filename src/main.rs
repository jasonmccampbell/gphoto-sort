extern crate clap;
extern crate regex;
extern crate walkdir;

use clap::{App, Arg};
use regex::Regex;
use std::path::*;
use walkdir::{DirEntry, WalkDir};

// History:
//  2019-Nov   jason   First version, does what I want
//                     Duplicate checks are simple name comparisons, most error checks are 'unwrap' asserts

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
            0 // Success
        }
    });
}

/// Bulk of the code: recursively walks the source directory looking for image and video files and ignoring the rest of the
/// rubbish / metadata files. And found files are checked against the destination directory and moved over if they don't
/// already exist.
fn move_files(src: &Path, dst: &Path) -> bool {
    let unprefix_re = Regex::new(r"^(\d{4})-(\d{2})-(\d{2}).*[.]([a-zA-Z0-9]+)$").unwrap();

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
            if let Some((year, month)) = extract_year_month(&unprefix_re, &filename_str) {
                let dst_dir = dst.join(year).join(month);
                if !dst_dir.exists() {
                    std::fs::create_dir_all(&dst_dir).expect(&format!("Unable to create directory {}", dst_dir.display()));
                }
                move_or_delete(&path, dst_dir.join(filename));
            }
        }
    }
    true
}

/// Move the file into the Google Photo's directory if new, otherwise we can delete it if a duplicate.
///
/// *NOTE:* 'duplicate' simply means the file exists with the same name, no MD5 or similar checking is done.
fn move_or_delete(src: &Path, dst: PathBuf) {
    if dst.exists() {
        println!("{} is duplicate - delete", src.display());
        std::fs::remove_file(src).expect(&format!("Unable to delete file {}", src.display()));
    } else {
        println!("Moving {} to {}", src.display(), dst.display());
        std::fs::rename(&src, &dst).expect(&format!("Move of {} to {} failed", src.display(), dst.display()));
    }
}

/// Figures out the year and month from the following file name formats:
///   * IMG_<year>_<month>_<day>*.jpg
///   * VID_<year>_<month>_<day>*.mp4
///   * <year>_<month>_<day>*.<ext>
///
/// For those new to Rust and compiler geeks, the signature of this function impresses me. The 'a annotation
/// means the result has the same lifetime as 'filename'. That is, the resulting 'str' values are
/// views using the data from wherever 'filename' gets it from. But note that we return the string views from
/// 'caps' which is returned by the regex. The compiler correctly sorts all of this out "knowing" that 'filename'
/// was passed in so these strings views are backed by the memory used for 'filename'. Through all of this
/// we never have to copy parts of 'filename' around!
fn extract_year_month<'a>(unprefix_re: &Regex, filename: &'a str) -> Option<(&'a str, &'a str)> {
    if filename.starts_with("IMG_") || filename.starts_with("VID_") {
        Some((&filename[4..8], &filename[8..10]))
    } else if let Some(caps) = unprefix_re.captures(filename) {
        // Captures 1 and 2 are year and month so files can get sorted under the right directory
        // Ignore 3 and 4, the day and extension
        Some((caps.get(1).unwrap().as_str(), caps.get(2).unwrap().as_str()))
    } else {
        None
    }
}

/// Flags files of interest, like actual images/videos, and rejects the metadata
fn is_of_interest(entry: &DirEntry) -> bool {
    match entry.path().extension() {
        None => false,
        Some(ext) => ext.eq("jpg") || ext.eq("JPG") || ext.eq("mp4") || ext.eq("MP4"),
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
