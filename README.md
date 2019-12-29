# gphoto-sort
Incrementally sort photos from a Google Takeout directory structure into an existing Google Drive/Google Photos 
directory structure

This is for those of us who were used to sync'ing Google Photos to Google Drive, and then sync'ing Google Drive locally
as a backup. Unfortunately the directory structures of the two are different, plus the Google Takeout archives are
full archives, not incremental drops. This means it takes a bit of automation to pull out the new photos from a Takeout
archive and insert them into the Google Drive directory structure, as well as delete all of the duplicates.

Gphoto-sort does this: it finds each image and video file in the Takeout archive, generates the expected path in a
Google-drive organized tree, then checks to see if the file exists or not. New files are moved into the appropriate
location and duplicates are deleted.

Of course, the easiest option would be to switch to the Takeout directory structure. However, if the Google Drive 
structure has already been backed up to a different cloud service, changing the structure may result in re-uploading
dozens or hundreds of GBs of photos. Maintaining the Google Drive structure avoids this.

# Example usage
If your Takeout archive is extracted to /tmp and your Google Drive sync'ing is in /archive, then:

./gphoto-sort /tmp/Takeout /archive/Google\ Drive/Google\ Photos

will find all .jpg and .mp4 files in the Takeout directory with date-based names and move them into
the appropriate date-based directory under "Google Photos". Specifically, files are typically named using
the following forms:
 * IMG_<year>_<month>_<day>*.jpg
 * VID_<year>_<month>_<day>*.mp4
 * \<year\>\_\<month>\_\<day\>\*.\*

The 'Google Photos' directory in the example command line above is organized with a directory for each year, 
and a subdirectory for each month. e.g.,
 * 2019/02/IMG_20190207_123456.jpg for 07-Feb-2019
 
Files matching the three Takeout patterns are moved into the appropriate year/month directory under
"Google Photos".

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

Please contact me with any questions.

# Why Rust?
Rust is a system-level language geared towards performance and reliability, not quickly scripting together a hack. I
originally wrote this in Bash, but got to the point where it exceeded my Bash-foo and I was having to look too much up.
Normally I'd failover to Python, but I really like Rust and figured I'd try it out. I'm impressed with
Rust's "dynamic range" in being able to hack together a utility high-level script like this, as well handle more typical
low-level, performant projects.
