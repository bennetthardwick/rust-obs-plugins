# scroll-focus-filter

## Requirements
- Linux
- OBS
- Xorg

## Usage
### Installation
```
https://github.com/bennetthardwick/rust-obs-plugins rust-obs-plugins
cd rust-obs-plugins
cargo build -p scroll-focus-filter --release
# Create a symlink to the plugin so that you don't need to copy when rebuilding, you
# could alternatively run `sudo cp ./target/release/scrollfocus.so /usr/lib/obs-plugins/`
sudo ln -s $(pwd)/target/release/libscrollfocus.so /usr/lib/obs-plugins/libscrollfocus.so
```
### Enabling

After installing the plugin, you need to add it as a filter to the source that's recording your desktop.

1. Open OBS
1. In the sources section, click the plus (+) and select `Screen Capture (XSHM)`
1. In the popup, select the display you want to record
1. The sources section will show the newly added source, right click and select `Filters`
1. At the bottom of the left-hand `Effect Filters` panel, click the plus (+)
1. Select `Scroll Focus Filter`
1. Note: if the screen goes black, restart OBS
1. In the properties section input your `Screen width` and `Screen height` in pixels. If the selected monitor isn't the top-left-most, input it's offset in pixels.
1. Enter the amount to zoom in the desktop

For more information on adding filters to OBS, check out [this forum post](https://obsproject.com/forum/resources/obs-studio-filters-for-sources-scenes-devices.226/)
