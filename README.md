# MP4 Lossless Rotator


Rotate MP4 videos instantly and losslessly by editing the rotation matrix (no re-encoding).
<p align="center">
	<a href="https://github.com/vanowm/MP4-Lossless-Rotator/releases/latest"><img src="https://img.shields.io/github/v/release/vanowm/MP4-Lossless-Rotator?label=Latest%20Release" alt="Latest Release"></a>
	<a href="https://github.com/vanowm/MP4-Lossless-Rotator/blob/master/LICENSE"><img src="https://img.shields.io/github/license/vanowm/MP4-Lossless-Rotator.svg" alt="License"></a>
</p>

## Download

**Download for Windows [latest release](https://github.com/vanowm/MP4-Lossless-Rotator/releases/latest/download/mp4-rotator-x86_64-pc-windows-msvc.zip)**


## Usage

### A. Command line
```sh
mp4-rotator C:\path\to\video1.mp4 C:\path\to\video2.mp4
```

### B. Clipboard (Windows Explorer)
Copy the file(s) in Explorer<p align="center"><img src="docs/copy.png" width="300" alt="Copy to clipboard"></p>

then run:
```sh
mp4-rotator
```
## Options
- Default: rotate 90° clockwise from current orientation.

| Option | Description |
|:--|:--|
| `--rotate=`nn, `-r=`nn | Set rotation to `0`, `90`, `180`, or `270`. |
| `--backup`, `-b` | Create a timestamped backup before modifying. |

Example:
```sh
mp4-rotator --rotate=180 --backup C:\path\to\video1.mp4 C:\path\to\video2.mp4
```

## Configuration
If present, _<b>`mp4-rotator.ini`<b>_ in the executable directory sets defaults.<br>See example:
[mp4-rotator_example.ini](https://github.com/vanowm/MP4-Lossless-Rotator/blob/master/mp4-rotator_example.ini)

## Example output
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

- Based on the [original repository](https://gitlab.com/AndreKR/mp4-rotator) by [André Hänsel](https://gitlab.com/AndreKR)
- Thanks to user aXeL-HH on StackOverflow for publishing [this method](https://stackoverflow.com/questions/25031557/rotate-mp4-videos-without-re-encoding/49535017#49535017)



## Roadmap
- Linux version (gate clipboard functionality by target)
