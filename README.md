# gphoto-sort
Sort photos from a Google Takeout directory structure into existing Google Drive/Google Photos directory structure

This is for those of us who used to sync Google Photos to Google Drive, and then sync Google Drive locally. I
have a large cache of photos stored locally and backed up. 

Google Takeout allows me to download all of my stored photos, if less conveniently. However, the directory structure
is quite different from the Google Drive version, meaning I'd have to re-backup handreds of GB of files just because
they moved to a new directory. Plus it included all of the files I already have as well as new ones; I don't want to 
re-backup the duplicates.

This utility finds each image and video file in the Takeout archive, checks to see if it already exists in a 
Google Drive-organized tree and either deletes the duplicate or moves it into the correct location. This makes syncing
by downloading a new archive every couple of months slightly less painful.
