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
$ cd raykit_eduobjs
$ cargo run --release -- /PATH/TO/RAYKIT.EXE
```

# Additions to the Events Editor

As of 0.3.0, this patcher also restores most of the missing editor functionality for Edutainment objects. I've implemented the functionality described in this unused help string:
```
Special Keys in EVENT editor
'up or down arrow' Change color or scroll samples per 50
'right or left arrow' Change frame (size,icon,number or sample)
```

When you hover over a letter or number in the editor, it will include the "color" number in the name you see at the bottom of the screen, but not the frame number. When you hover over a sound sample, it will simply show the sample name (e.g. "PERDU", "CHERCHE", etc.).

Saving and loading also works, although you need to make sure you use the right event names in your `EVE.MLT` file:
* Letters should start with `MS_edul_`
* Numbers should start with `MS_educ_` (for *chiffre*)
* Artworks should start with `MS_icon`
* Sound samples should be called `MS_sample`

Also, if you share a level using these events, the end user also needs to have patched their copy of Rayman Designer with this tool.

# Limitations

* This has only been tested on the GOG version and may not work on others.
* This does not provide the sprites for letters, numbers and artworks. You need to use [Ray1Map](https://github.com/Adsolution/Ray1Map) to extract them from an Educational game.
* This does not provide any sound samples beyond the 26 available in stock Rayman Designer (but these do include the gendoor sounds from Edutainment). I'm pretty sure there's a way to copy more samples over from the Edutainment games, but I haven't looked into it yet.
  - By the way, since there are only 26 samples, the "scroll samples per 50" functionality doesn't work. However, if you somehow add 25 or more extra samples to your game (again, I need to look into it), it will work.
* When you place a sample in your map, it probably won't play until you save, quit and reenter the level. This is because samples are loaded selectively when a map is loaded.
  - I may fix this in the next version
