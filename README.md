This project aims to create a eink-friendly view of Archive of Our Own for Kobo devices, allowing reading and interacting with works without having to save them to local memory first. It also aims to provide functionality that Ao3 has declined to implement on their end, such as the ability to set default search parameters, and the ability to easily access saved searches.

## Pre-emptive FAQ
**Why isn't this being made as an addon to Plato?**  
The structure of Ao3 works don't map particularly well to documents in Plato, which makes shoehorning them in tough, plus this code requires a lot of additional network functionality. But eventually there'll be a passthrough to Ao3's epub download functionality which will let you save and open stuff in Plato later.

**Why is the binary so much larger than Plato?**  
Network and HTML processing packages, mostly. If Ao3 ever releases an API, we'll be able to ditch the latter (as reqwests, the HTTP client package, has hooks for JSON processing), but until then we're stuck scraping full HTML pages and then extracting the necessary bits. But hey, it's still smaller than trying to save an entire tag off into epub and loading that, right?

**Will you make a version for [other site]?**  
No, but you're welcome to fork this and do it yourself! The changes I've already made in the code should make it a lot easier to port for another site, as various views and handlers exist already.

## Compiling from Soure Code
### First Time Setup
(Based on the [build instructions for Plato](https://github.com/baskerville/plato/blob/master/doc/BUILD.md))
1. Install the [Kobo Developer Toolchain](https://drive.google.com/drive/folders/1YT6x2X070-cg_E8iWvNUUrWg5-t_YcV0)
    1. Download the toolchain from Google Drive
    2. Unzip the folder into the parent directory of your bin.  (```echo $PATH``` in a terminal, and finding a directory in the that looks similar to /home/{username}/bin. As an example, if your PATH contains /home/qwertynerd97/bin/, you would unzip the file into /home/qwertynerd97)
2. Install rustup: ```curl https://sh.rustup.rs -sSf | sh```
3. Install the rustup ARM target: ```rustup target add arm-unknown-linux-gnueabihf```

### Release Build
1. Run the build script: ```./build.sh slow```

## Developing
### First Time Setup
1. Install dev dependencies
    * MacOS (requires at least MacOS 13): ```brew install cmake mupdf harfbuzz djvulibre sdl2```
    * Fedora (requires at least Fedora 39): ```sudo dnf install mupdf-devel harfbuzz djvulibre SDL2-devel freetype-devel jbig2dec-devel gumbo-parser-devel openjpeg2-devel  libjpeg-turbo-devel djvulibre-devel```
        * On lower versions of Fedora, the default mupdf-devel is incompatible with this project, which requires mupdf 1.23, so additional steps are needed to build the proper mupdf version
            1. Download the [mupdf 1.23.11 source code](https://mupdf.com/downloads/archive/mupdf-1.23.11-source.tar.gz) unzip it
            2. Run ```sudo dnf install mesa-libGL-devel mesa-libGLU-devel xorg-x11-server-devel libXcursor-devel libXrandr-devel libXinerama-devel```
            2. In the unzipped folder, run ```make HAVE_X11=no HAVE_GLUT=no prefix=/home/{user} install``, replacing {user} with your username
2. Run the build script: ``` ./build.sh slow``` - this installs all the project libraries

### Development Commands
* Run unit tests (with coverage checker): ```cargo llvm-cov```
* Run unit tests with HTML coverage report (viewable in browser): ```cargo llvm-cov --open```
* Run cucumber integration tests (requires internet access): ```TBD``
* Build dev version: ```cargo build```

## Credits
* Original core code (including all of the HTML engine, event system, and rendering system) - [Plato](https://github.com/baskerville/plato)
* Additional icons - [Material Design Icons](https://materialdesignicons.com/)
* A number of friends who encouraged me to turn the idea kicking around my head for months into an actual app, and provided feedback on very early iterations.
* [ao3downloader](https://github.com/nianeyna/ao3downloader), which helped point in the direction of fixing the problems I was having with login.

## Known Issues:
* Notification position drifts (issue underlying in Plato)
* Deleted works look weird in the list
* Kudos button not always working?
* Works with anonymous authors will show as having no authors
* Changing reader display settings doesn't alter current reader instances
* About Work overlay currently missing stats items
* Summary in About Work overlay ignores HTML styling
* Bookmarks icon does nothing.

## To be Done:
* Other views (Author, Series, Bookmarks, Collections)
* Nicer landing page
* Search form
* Access to reading/leaving comments
* Work blocklist
* Worklist navigation history
* Option to set global default search parameters
* Option to save works for offline reading
* Show Ao3 symbols grid in various views
* Rendering of remote images

## Questions:
* About Work overlay info and work items in indexes are currently kind of ugly - how could they be better?
* At the moment works are loaded entirely, to dodge around having to figure out how to handle chapter navigation, but this results in long load times for large works, and no way of telling what chapter you're in - is this acceptable? How can we handle remote chapters?
