# Icon sources

- `app-icon.svg`: the app icon (taskbar, installer, start menu, window).
- `app-icon-small.svg`: a simpler version (one thick ring) for tiny sizes: the
  tray and the 16 to 32 px icons.

Regenerate the icons in `src-tauri/icons/` with ImageMagick (8-bit RGBA, as
Tauri requires):

```bash
cd src-tauri/icons
r() { magick -background none -density 1536 "$1" -resize "${2}x${2}" -depth 8 "PNG32:$3"; }
r source/app-icon.svg 128 128x128.png
r source/app-icon.svg 256 128x128@2x.png
r source/app-icon.svg 512 icon.png
r source/app-icon-small.svg 32 32x32.png
r source/app-icon-small.svg 64 tray.png
for s in 16 24 32; do r source/app-icon-small.svg $s /tmp/ico-$s.png; done
for s in 48 64 128 256; do r source/app-icon.svg $s /tmp/ico-$s.png; done
magick /tmp/ico-{16,24,32,48,64,128,256}.png icon.ico
```
