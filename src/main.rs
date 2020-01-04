extern crate clap;
extern crate crypto;
extern crate regex;
extern crate walkdir;

use clap::{App, Arg};
use crypto::digest::Digest;
use crypto::md5;
use regex::Regex;
use std::io::Read;
use std::path::*;
use walkdir::{DirEntry, WalkDir};

// History:
//  2019-Nov   jason   First version, does what I want
//                     Duplicate checks are simple name comparisons, most error checks are 'unwrap' asserts
//  2020-Jan   jason   Added hashing check for duplicates and handling for more file name variations

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
        .arg(
            Arg::with_name("dry-run")
                .long("dry-run")
                .short("n")
                .help("Report actions to be taken, but don't do anything"),
        )
        .get_matches();

    let src = std::path::Path::new(matches.value_of("source-takeout-dir").unwrap());
    let dst = std::path::Path::new(matches.value_of("dest-gphotos-dir").unwrap());
    std::process::exit({
        if !validate_takeout_dir(src) || !validate_gphoto_dir(dst) || !move_files(src, dst, matches.is_present("dry-run")) {
            1
        } else {
            0 // Success
        }
    });
}

/// Bulk of the code: recursively walks the source directory looking for image and video files and ignoring the rest of the
/// rubbish / metadata files. And found files are checked against the destination directory and moved over if they don't
/// already exist.
fn move_files(src: &Path, dst: &Path, dry_run: bool) -> bool {
    let unprefix_re = Regex::new(r"^(\d{4})-(\d{2})-(\d{2}).*([.][a-zA-Z0-9]+)?$").unwrap();

    let mut moved = 0;
    let mut deleted = 0;

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
            let containing_dir = path
                .parent()
                .map_or_else(|| std::ffi::OsStr::new("_"), |p| p.file_name().unwrap())
                .to_string_lossy();
            if let Some((year, month)) = extract_year_month(&unprefix_re, &containing_dir, &filename_str) {
                let dst_dir = dst.join(year).join(month);
                if !dst_dir.exists() {
                    std::fs::create_dir_all(&dst_dir).unwrap_or_else(|_| panic!("Unable to create directory {}", dst_dir.display()));
                }
                match move_or_delete(&path, dst_dir.join(filename), dry_run) {
                    Ok(true) => moved += 1,
                    Ok(false) => deleted += 1,
                    Err(e) => {
                        println!("Error on file {}: {}", path.display(), e);
                    }
                }
            }
        }
    }
    println!("Moved {} new files into place, deleted {} duplicate files", moved, deleted);
    true
}

/// Move the file into the Google Photo's directory if new, otherwise we can delete it if a duplicate.
///
/// *NOTE:* 'duplicate' simply means the file exists with the same name, no MD5 or similar checking is done.
/// Returns true if moved, false if deleted
fn move_or_delete(src: &Path, dst: PathBuf, dry_run: bool) -> Result<bool, std::io::Error> {
    if dst.exists() {
        if get_file_hash(dst.as_path())? == get_file_hash(src)? {
            println!("{} is duplicate - delete", src.display());
            if !dry_run {
                std::fs::remove_file(src).unwrap_or_else(|_| panic!("Unable to delete file {}", src.display()));
            }
        } else {
            // TODO: Need to generate unique file name
            println!(
                "{} appears to be a duplicate of {}, but contents are not the same",
                src.display(),
                dst.display()
            );
        }
        Ok(false)
    } else {
        println!("Moving {} to {}", src.display(), dst.display());
        if !dry_run {
            std::fs::rename(&src, &dst).unwrap_or_else(|_| panic!("Move of {} to {} failed", src.display(), dst.display()));
        }
        Ok(true)
    }
}

/// Figures out the year and month from the following file name formats:
///   * IMG_<year>_<month>_<day>*.jpg
///   * VID_<year>_<month>_<day>*.mp4
///   * <year>_<month>_<day>*.<ext>
///   * 2013-03-16 #2/IMG_0003-edited(1).jpg
///
/// For those new to Rust and compiler geeks, the signature of this function impresses me. The 'a annotation
/// means the result has the same lifetime as 'filename'. That is, the resulting 'str' values are
/// views using the data from wherever 'filename' gets it from. But note that we return the string views from
/// 'caps' which is returned by the regex. The compiler correctly sorts all of this out "knowing" that 'filename'
/// was passed in so these strings views are backed by the memory used for 'filename'. Through all of this
/// we never have to copy parts of 'filename' around!
fn extract_year_month<'a>(unprefix_re: &Regex, containing_dir: &'a str, filename: &'a str) -> Option<(&'a str, &'a str)> {
    if filename.starts_with("IMG_") || filename.starts_with("VID_") || filename.starts_with("MVIMG_") {
        let year = &filename[4..8];
        let month = &filename[8..10];
        if year.parse::<i32>().is_ok() && month.parse::<i32>().is_ok() {
            Some((&filename[4..8], &filename[8..10]))
        } else {
            // Example: 2013-03-16 #2/IMG_0003-edited(1).jpg
            // There isn't a year and month in the same so try to parse the containing directory
            if let Some(caps) = unprefix_re.captures(containing_dir) {
                Some((caps.get(1).unwrap().as_str(), caps.get(2).unwrap().as_str()))
            } else {
                println!(
                    "Unable to parse year and month from filename {}, or containing directory {}",
                    filename, containing_dir
                );
                None
            }
        }
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

/// Return the hash (MD5 or other) of a file for comparing whether they are actually identical or not
fn get_file_hash(p: &Path) -> Result<String, std::io::Error> {
    const BUF_SIZE: usize = 1024 * 1024;
    let mut md5digest = md5::Md5::new();
    let mut f = std::fs::OpenOptions::new().read(true).open(p)?;
    let mut buf: [u8; BUF_SIZE] = [0; BUF_SIZE];
    let mut size = f.read(&mut buf[..])?;
    while size > 0 {
        md5digest.input(&buf[0..size]);
        size = f.read(&mut buf[..])?;
    }
    Ok(md5digest.result_str())
}
