# gphoto-sort
Sort photos from a Google Takeout directory structure into existing Google Drive/Google Photos directory structure

This is for those of us who were used to sync'ing Google Photos to Google Drive, and then sync'ing Google Drive locally. I
have a large local cache of photos which I backup independent of Google. However, Google has ended their support
for sync'ing Google Photos to Drive.

Google Takeout allows me to download all of my stored photos, if less conveniently that before. However, the directory structure
is quite different from the Google Drive version, meaning I'd have to re-backup handreds of GB of files just because
they moved to a new directory. Plus it included all of the files I already have as well as new ones; I don't want to 
store the duplicates.

This utility finds each image and video file in the Takeout archive, checks to see if it already exists in a 
Google Drive-organized tree and either deletes the duplicate or moves it into the correct location. This makes syncing
by downloading a new archive every couple of months slightly less painful.

# Example usage
If your Takeout archive is extracted to /tmp and your Google Drive sync'ing is in /archive, then:

./gphoto-sort /tmp/Takeout /archive/Google\ Drive/Google\ Photos

will find all of .jpg and .mp4 files in the Takeout directory with date-based names and move them into
the appropriate date-based directory under "Google Photos". Specifically, I found files of the following forms:
 * IMG_<year>_<month>_<day>*.jpg
 * VID_<year>_<month>_<day>*.mp4
 * \<year\>\_\<month>\_\<day\>\*.\*

The 'Google Photos' directory in the example above is organized with a directory for each year, and a subdirectory
for each month. Files matching the three patterns here are moved into the appropriate year/month directory under
"Google Photos".

# What's not moved
I found a number of directories under takout matching albums I had created and the image file names were either
apparently human-assigned or some numbering scheme I didn't recognize. **These are not moved**. Please be careful
to look for files this utility doesn't handle to decide whether they should be moved or not.

You can use the Linux 'find' command to search for remaining files like this:
* find . -name "*.jpg"

Note that ".jpg" and ".JPG" are both used and distinct on Linux so you need to search both.

# Platform
This utility was written for Linux and may or may not work on others.

Please contact me with any questions.

# Why Rust?
Rust is a system-level language geared towards performance and reliability, not quick scripting together of hack. I
originally wrote this in Bash, but got to the point where it exceeded my Bash-foo and I was having to look too much up.
Normally I'd failover to Python, but I really like Rust and figured I'd try it out. I'm really very impressed with
Rust's "dynamic range" in being able to hack together a very high-level script like this. Doing this in C would be
painful given the lack of library support. C++ would be ok if Boost is already built and available, but I still
wouldn't do it.
