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

## Example output

The tool prints detailed information about its process and aborts if it encounters anything unexpected in the file:

```text
Start
Processing file: C:\test\Marea.mp4
Found moov box at 32
Found trak box at 148
Walking trak -> mdia -> hdlr
Track type: vide
Found trak box at 34333
Walking trak -> mdia -> hdlr
Track type: soun
Found video track
Rotation matrix found: No rotation => changing to: 90°
Writing new rotation matrix now
Done.
```

## Credits

Thanks to user aXeL-HH on StackOverflow for publishing
[this method](https://stackoverflow.com/questions/25031557/rotate-mp4-videos-without-re-encoding/49535017#49535017).


## TODO
* Linux version (by target gating the clipboard functionality)
* (Optionally) create backup file?