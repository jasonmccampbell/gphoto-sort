# gphoto-sort
A utility to incrementally sort photos from a Google Takeout directory structure into an existing Google Drive/Google Photos 
directory structure

This is for those of us who were used to sync'ing Google Photos to Google Drive, and then sync'ing Google Drive locally
as a backup. Unfortunately the directory structures of the two are different, plus the Google Takeout archives are
full archives, not incremental drops. This means it takes a bit of automation to pull out the new photos from a Takeout
archive and insert them into the Google Drive directory structure, as well as delete all of the duplicates.

Gphoto-sort does this: it finds each image and video file in the Takeout archive, generates the expected path in a
Google-drive organized tree, then checks to see if the file exists or not. New files are moved into the appropriate
location and duplicates are deleted.

Of course, the easiest option would be to switch to the Takeout directory structure. However, if the Google Drive 
structure has already been backed up to a different cloud service, changing the structure results in re-uploading
dozens or hundreds of GBs of photos. Maintaining the Google Drive structure avoids this.

This utility is also useful for merging multiple Takeout archives (multiple family members) into a single tree. It de-duplicates the images
that have the same name (such as from shared albums) while keeping differing images that have name collisions.

# Example usage
If your Takeout archive is extracted to /tmp and your Google Drive sync'ing is in /archive, then:

./gphoto-sort [-n | --dry-run] /tmp/Takeout /archive/Google\ Drive/Google\ Photos

will find common image and video files (JPG, PNG, GIF, MP4, MOV) in the Takeout directory with date-based names 
and move them into the appropriate date-based directory under "Google Photos". Specifically, files are typically named using
the following forms:
 * IMG_<year>_<month>_<day>*.jpg (or .png, etc)
 * MVIMG_<year>_<month>_<day>*.gif
 * VID_<year>_<month>_<day>*.mp4 (or .mov)
 * \<year\>\_\<month>\_\<day\>\*.\*
 * \<year\>\_\<month>\_\<day\>\*/IMG_*

The 'Google Photos' directory in the example command line above is organized with a directory for each year, 
and a subdirectory for each month. e.g.,
 * 2019/02/IMG_20190207_123456.jpg for 07-Feb-2019
 
Files matching the above Takeout patterns are moved into the appropriate year/month directory under
"Google Photos".

If -n or --dry-run is given, it will report the actions to be taken, but not actually perform them. This is highly
recommended for a first-run just to make sure it going to do what you expect it to.

If a file with the same name already exists, the content of the two files is compared. If identical, the duplicate
is deleted. If the two are different, a suffix of "-1" is appended and the duplicate/compare check is repeated.
This continues until either a file with identical content is found or no matching file name exists, in which
case the file is moved to the suffixed file name.

# What's not moved
The Takeout archive can contain directories with names matching named albums and with human-named image files.
**These are not moved**. Please be careful to look for these files as this utility ignores them (doesn't move them,
doesn't delete them.)

You can use the Linux 'find' command to search for any remaining files like this:
* find . -name "*.jpg"

Note that ".jpg" and ".JPG" are both used and distinct on most Linux filesystems so you need to search both. 
Ditto for .mp4/.MP4 files.

# Platform
This utility was written for Linux and may or may not work on others.

Comments, questions, bug reports, and pull-requests are all welcome.

# Why Rust?
I originally started writing this in Bash, but got to the point where it exceeded my Bash-foo and I was having to look too much up.
Normally I'd failover to Python, but I write C++ professionally and like the idea of utilities being small and fast. However, I would
never use C++ because it's too much work. I was curious, how is Rust's "dynamic range", its ability to do high-level script-type work 
as well as the typical low-level development? 

As you can see, doing this in Rust is only a little more code than Python would likely have been. It is really the 
[walkdir](https://crates.io/crates/walkdir) and [regex](https://crates.io/crates/regex) crates combined with Cargo that make it so easy. 
Cargo actually makes package management considerably easier than using Python, IMHO, because packages are local to a project.

A big secondary win is leveraging the excellent [Rayon](https://crates.io/crates/rayon) crate. Converting this from a 'dumb' sequential
script to a modern parallel utility was as simple as converting the for loop to map() call and adding a trivial reduce step at the end. 

Big props to the authors of all these crates.
