# Lossless MP4 rotator

This rotates MP4 files losslessly by switching the rotation matrix in the track header.

## Download

Latest version: [Windows](https://gitlab.com/AndreKR/mp4-rotator/-/jobs/artifacts/master/download?job=binaries)

## Usage

Either specify a file path on the command line:
```shell
mp4-rotator c:\path\to\video.mp4
```

Or copy the file into the Windows clipboard

![](docs/copy.png)

and then simply run `mp4-rotator`.  


## TODO
* Linux version (by target gating the clipboard functionality)
* (Optionally) create backup file?