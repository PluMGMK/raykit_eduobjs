raykit_eduobjs
==============

A patcher for the Rayman Designer EXE to enable extended functionality associated with letters, numbers and artworks from the Educational Rayman games.

__If you want to learn about how the patcher works__, you can check out [this blog post](https://www.vigovproductions.net/interactive/raykit-eduobjs.html).

# Illustration

Using [Ray1Map](https://github.com/Adsolution/Ray1Map), I can transfer all the ETA/DES files from an Educational Rayman game (and also Rayman 1) to Rayman Designer,
and then proceed to make an edit like this to one of the 24 stock levels:

!["Eat at Joe's" lettering and an artwork inserted into "Peaks and Rocks" using Ray1Map](https://vigovproductions.net/interactive/images/raykit-eduobjs/snip-1603909889.png)

If I load up the modified level using the stock `RAYKIT.EXE`, however, this is what I get:

![Peaks and Rocks edit as rendered by stock RayKit](https://vigovproductions.net/interactive/images/raykit-eduobjs/screenshot-1603922394.jpg)

This patcher makes it match the expectation from Ray1Map:

![Peaks and Rocks edit as rendered by patched RayKit](https://vigovproductions.net/interactive/images/raykit-eduobjs/screenshot-1603918009.jpg)

# Usage

For this patcher to be of any use to you, you need to use Ray1Map to transfer the ETA/DES files from an Educational Rayman game into Rayman Designer.
If you don't, the patcher will still work, but the code it adds probably won't be of any use to you…

If you're on Windows (either 32- or 64-bit), you can grab the EXE from the latest entry on the [releases page](https://github.com/PluMGMK/raykit_eduobjs/releases).
Once you have downloaded it, you can drag your `RAYKIT.EXE` file onto this one, and it will patch it and create a `RAYKIT.EXE.BAK` backup file in case anything goes wrong.
Note: I've only tested this on the GOG version of Rayman Designer, so it may fail on other versions.

Alternatively, if you're comfortable with the command line and have a Rust nightly toolchain installed, you can do this to compile it yourself:
```
$ git clone https://github.com/PluMGMK/raykit_eduobjs.git
$ cd ray1_unlocklang
$ cargo run --release -- /PATH/TO/RAYKIT.EXE
```

# Limitations

* This has only been tested on the GOG version and may not work on others.
* This does not provide the sprites for letters, numbers and artworks. You need to use [Ray1Map](https://github.com/Adsolution/Ray1Map) to extract them from an Educational game.
* If you add letters and numbers to your EVE.MLT file, the Events Editor will enable you to change their colours after you apply this patch.
However, you can't use the Events Editor to change the frame, so your letters will be stuck in uppercase and you can't really use the artworks. You need Ray1Map for that.
  - The reason for this is that the colour-changing code is already in stock Rayman Designer (for Tings and butterflies), so I just needed to change some conditionals to make it apply to letters and numbers.
The frame-changing code seems to have been removed altogether though, so I'd have to re-implement it from scratch.
I'm not sure if it's even possible, given that the Events Editor uses a different format for storing data, which may not include the frame info (I'll have to check).
