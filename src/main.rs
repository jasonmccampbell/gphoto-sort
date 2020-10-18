extern crate clap;
extern crate crypto;
extern crate regex;
extern crate walkdir;

use clap::{App, Arg};
use crypto::digest::Digest;
use crypto::md5;
use regex::Regex;
use std::ffi::OsStr;
use std::io::Read;
use std::path::*;
use walkdir::{DirEntry, WalkDir};

const YEAR_DATE_RE: &str = r"^(?i)(IMG_|VID_|MVIMG_)?(\d{4})[-_]?(\d{2})[-_]?(\d{2}).*$";

// History:
//  2019-Nov   jason   First version, does what I want
//                     Duplicate checks are simple name comparisons, most error checks are 'unwrap' asserts
//  2020-Jan   jason   Added hashing check for duplicates and handling for more file name variations
//                     Added -n/--dry-run option
//                     More file types and date-based naming patterns are recognized
//                     Reports total number of moved and deleted files at the end of the run

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
    let unprefix_re = Regex::new(YEAR_DATE_RE).unwrap();

    let (moved, deleted) = WalkDir::new(src.join("Google Photos"))
        .into_iter()
        .filter_entry(|e| e.file_name().to_str().map_or(false, |s| !s.starts_with("Hangout")))
        .filter_map(|e| e.ok())
        .filter(|e| !e.path().is_dir())
        .filter(is_of_interest)
        .map(|entry| {
            let path = entry.path();
            if let Some(file_stem) = path.file_stem() {
                let file_stem_str = file_stem.to_string_lossy();
                let file_ext = path.extension().unwrap();
                let containing_dir = path
                    .parent()
                    .map_or_else(|| std::ffi::OsStr::new("_"), |p| p.file_name().unwrap())
                    .to_string_lossy();
                if let Some((year, month)) = extract_year_month(&unprefix_re, &containing_dir, &file_stem_str) {
                    let dst_dir = dst.join(year).join(month);
                    if !dst_dir.exists() {
                        std::fs::create_dir_all(&dst_dir).unwrap_or_else(|_| panic!("Unable to create directory {}", dst_dir.display()));
                    }

                    let file_name = format!("{}.{}", file_stem_str, file_ext.to_string_lossy());
                    match move_or_delete(&path, dst_dir.join(file_name), file_stem, &file_ext, dry_run, 0) {
                        Ok(true) => (1, 0),
                        Ok(false) => (0, 1),
                        Err(e) => {
                            println!("Error on file {}: {}", path.display(), e);
                            (0, 0)
                        }
                    }
                } else {
                    (0, 0)
                }
            } else {
                (0, 0)
            }
        })
        .fold((0, 0), |res, acc| (res.0 + acc.0, res.1 + acc.1));
    println!("Moved {} new files into place, deleted {} duplicate files", moved, deleted);
    true
}

/// Move the file into the Google Photo's directory if new, otherwise we can delete it if a duplicate.
/// Files are first checked for name matches and, if found, checked by calculating the hash for both.
/// Returns true if moved, false if deleted
fn move_or_delete(
    src: &Path,
    mut dst: PathBuf,
    orig_file_stem: &OsStr,
    file_ext: &OsStr,
    dry_run: bool,
    move_count: usize,
) -> Result<bool, std::io::Error> {
    // Append a "variant" number to the end of the file name. eg. IMG_2001_0203.jpg -> IMG_2001_0203-1.jpg
    incremented_variant(&mut dst, orig_file_stem, file_ext, move_count);

    if dst.exists() {
        if get_file_hash(dst.as_path())? == get_file_hash(src)? {
            println!("{} is duplicate - delete", src.display());
            if !dry_run {
                std::fs::remove_file(src).unwrap_or_else(|_| panic!("Unable to delete file {}", src.display()));
            }
            Ok(false)
        } else {
            assert!(move_count < 1000); // something will have gone horribly wrong...
            println!("Duplicate found at {}, uniquifying...", dst.display());
            move_or_delete(src, dst, orig_file_stem, file_ext, dry_run, move_count + 1)
        }
    } else {
        println!("Moving {} to {}", src.display(), dst.display());
        if !dry_run {
            std::fs::rename(&src, &dst).unwrap_or_else(|_| panic!("Move of {} to {} failed", src.display(), dst.display()));
        }
        Ok(true)
    }
}

/// Mutate the filename in 'dst' to have a "-N" suffix if move_count is greater than zero where 'N' is move count.
/// e.g.:   /a/b/img_1234.jpg  ->  /a/b/img_1234-1.jpg if move_count == 1
fn incremented_variant(dst: &mut PathBuf, file_stem: &OsStr, file_ext: &OsStr, move_count: usize) {
    if move_count != 0 {
        // Long because it is done as OsString instead of String. Is there format! for OsString?
        let mut new_fn = file_stem.to_os_string();
        new_fn.push("-");
        new_fn.push(move_count.to_string());
        new_fn.push(".");
        new_fn.push(file_ext);
        dst.set_file_name(&new_fn);
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
fn extract_year_month<'a>(unprefix_re: &Regex, containing_dir: &'a str, file_stem: &'a str) -> Option<(&'a str, &'a str)> {
    if let Some(caps) = unprefix_re.captures(file_stem) {
        // Captures 2 and 3 are year and month so files can get sorted under the right directory
        // Ignore 3 and 4, the day and extension; 1 is the 'IMG' or other prefix
        Some((caps.get(2).unwrap().as_str(), caps.get(3).unwrap().as_str()))
    } else if let Some(caps) = unprefix_re.captures(containing_dir) {
        Some((caps.get(2).unwrap().as_str(), caps.get(3).unwrap().as_str()))
    } else {
        None
    }
}

/// Flags files of interest, like actual images/videos, and rejects the metadata
fn is_of_interest(entry: &DirEntry) -> bool {
    match entry.path().extension() {
        None => false,
        Some(ext) => {
            // est is OsStr. Is there a better way? A static, immutable HashSet is the C++-ish pattern I'd normally use
            // but construction isn't const (yet)
            ext.eq("jpg")
                || ext.eq("JPG")
                || ext.eq("png")
                || ext.eq("PNG")
                || ext.eq("mp4")
                || ext.eq("MP4")
                || ext.eq("mov")
                || ext.eq("MOV")
                || ext.eq("gif")
                || ext.eq("GIF")
        }
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

#[test]
fn test_fn_incr() {
    let path = Path::new("/archive/Google Drive/Google Photos/2001/04/IMG_2001_04_03_12345.jpg");
    let file_stem = path.file_stem().unwrap();
    let file_ext = path.extension().unwrap();
    let mut pb = path.to_path_buf();
    assert_eq!(pb.to_str(), Some("/archive/Google Drive/Google Photos/2001/04/IMG_2001_04_03_12345.jpg"));

    incremented_variant(&mut pb, &file_stem, &file_ext, 0);
    assert_eq!(pb.to_str(), Some("/archive/Google Drive/Google Photos/2001/04/IMG_2001_04_03_12345.jpg"));

    incremented_variant(&mut pb, &file_stem, &file_ext, 1);
    assert_eq!(
        pb.to_str(),
        Some("/archive/Google Drive/Google Photos/2001/04/IMG_2001_04_03_12345-1.jpg")
    );

    incremented_variant(&mut pb, &file_stem, &file_ext, 2);
    assert_eq!(
        pb.to_str(),
        Some("/archive/Google Drive/Google Photos/2001/04/IMG_2001_04_03_12345-2.jpg")
    );
}

#[test]
fn test_extra_year_date() {
    let unprefix_re = Regex::new(YEAR_DATE_RE).unwrap();

    assert_eq!(extract_year_month(&unprefix_re, "My Album", "IMG_2013_04_02"), Some(("2013", "04")));
    assert_eq!(extract_year_month(&unprefix_re, "My Album", "VID_2014_05_02"), Some(("2014", "05")));
    assert_eq!(extract_year_month(&unprefix_re, "My Album", "mvimg_2014_05_02"), Some(("2014", "05")));
    assert_eq!(extract_year_month(&unprefix_re, "My Album", "2014_12_31"), Some(("2014", "12")));
    assert_eq!(
        extract_year_month(&unprefix_re, "2020-14-17", "IMG_0004-edited(17)"),
        Some(("2020", "14"))
    );
    assert_eq!(
        extract_year_month(&unprefix_re, "2021-13-17 #2", "IMG_0004-edited(17)"),
        Some(("2021", "13"))
    );
}
