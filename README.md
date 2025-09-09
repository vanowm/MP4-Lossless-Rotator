# Lossless MP4 rotator

This rotates MP4 files losslessly by switching the rotation matrix in the track header.

## Download

Latest version: [Windows](https://github.com/vanowm/MP4-Lossless-Rotator/releases/latest/download/mp4-rotator-x86_64-pc-windows-msvc.zip)

## Usage

Either specify file paths in the command line:
```
mp4-rotator c:\path\to\video1.mp4 c:\path\to\video2.mp4 c:\path\to\video3.mp4
```

Or copy the file(s) into the Windows clipboard

![](docs/copy.png)

and then simply run `mp4-rotator`.

By default it rotates 90° clockwise from current rotation. This can be changed with command line parameters:


|Parameter|Description|
|------------|---------------|
| `--rotate=`*`nn`* OR `-r=`*`nn`*           | Specify rotation to *`nn`* degrees. Accepted values: `0`, `90`, `180`, `270`.   |
| `--backup` OR `-b`       | Backup original file. It will create a backup file with modified date suffix|

```
mp4-rotator --rotate=180 --backup c:\path\to\video1.mp4 c:\path\to\video2.mp4
```
## Example output

The tool prints detailed information about its process and aborts if it encounters anything unexpected in the file:

```text
Start
Processing file: C:\test\Marea.mp4 (forced rotation: 180°)
Found moov box at 32
Found trak box at 148
Walking trak -> mdia -> hdlr
Track type: vide
Found trak box at 34333
Walking trak -> mdia -> hdlr
Track type: soun
Found video track
Rotation matrix found: 0° => changing to: 180°
Writing new rotation matrix: success
Finished.
```

## Credits

[original repository](https://gitlab.com/AndreKR/mp4-rotator) by [André Hänsel](https://gitlab.com/AndreKR)

Thanks to user aXeL-HH on StackOverflow for publishing
[this method](https://stackoverflow.com/questions/25031557/rotate-mp4-videos-without-re-encoding/49535017#49535017).


## TODO
* Linux version (by target gating the clipboard functionality)
