This project aims to create a eink-friendly view of Archive of Our Own for Kobo devices, allowing reading and interacting with works without having to save them to local memory first. It also aims to provide functionality that Ao3 has declined to implement on their end, such as the ability to set default search parameters, and the ability to easily access saved searches.

## Development Guide (WIP)
This has only been tested on recent versions of Ubuntu Linux, and will probably need testing for other distros and OSes

- Install rustup: `curl https://sh.rustup.rs -sSf | sh`
### Building for Kobos
- Install the correct architecture target for cross-compiling `rustup target add arm-unknown-linux-gnueabihf`
- Install the Kobo toolchain. This is necessary because the Kobo firmware uses older versions of libc than are usually shipped in repositories.
    - Kobo has their build files [available on Github](https://github.com/kobolabs/Kobo-Reader/), but the toolchains require either complex magic with the GitHub API, or cloning the entire (extremely large) repo, because they're in LFS storage.
    - Instead, you can download the toolchain files [from Linaro](https://releases.linaro.org/components/toolchain/binaries/4.9-2017.01/arm-linux-gnueabihf/) - `gcc-linaro-4.9.4-2017.01-x86_64_arm-linux-gnueabihf.tar.xz` is the correct toolchain for building on modern amd64 Linux systems
    - extract the contents of the tar, and add it to the start of your system path.
        ```
        tar -xf gcc-linaro-4.9.4-2017.01-x86_64_arm-linux-gnueabihf.tar.xz
        mv gcc-linaro-4.9.4-2017.01-x86_64_arm-linux-gnueabihf kobo-build
        export PATH="$(pwd)/kobo-build/bin:$PATH"
        ```
- Run `./build.sh` to build a binary suitable for Kobos
    - By default, this pulls pre-built copies of necessary libraries from the upstream Plato project
    - If these do not work, you can recompile local copies by running `./build.sh slow`. 

### Running the emulator
- You will need system installs of all the libraries that the reader uses - MuPDF 1.23.11, DjVuLibre, FreeType, HarfBuzz, libpng, libjpeg, and Gumbo - along with SDL2, which only the emulator uses, along with their development headers.
    - **Note**: The version of MuPDF in many Linux package managers is not up-to-date, and you may need to compile from source. MuPDF has [instructions for building the library on their site](https://mupdf.readthedocs.io/en/latest/quick-start-guide.html#building-the-library)
- Run `./run-emulator.sh` to run the emulator, or if you use Visual Studio Code, you can can launch it with or without a debugger attached via the Run menu.

## Pre-emptive FAQ
**Why isn't this being made as an addon to Plato?**  
The structure of Ao3 works don't map particularly well to documents in Plato, which makes shoehorning them in tough, plus this code requires a lot of additional network functionality. But eventually there'll be a passthrough to Ao3's epub download functionality which will let you save and open stuff in Plato later.

**Why is the binary so much larger than Plato?**  
Network and HTML processing packages, mostly. If Ao3 ever releases an API, we'll be able to ditch the latter (as reqwests, the HTTP client package, has hooks for JSON processing), but until then we're stuck scraping full HTML pages and then extracting the necessary bits. But hey, it's still smaller than trying to save an entire tag off into epub and loading that, right?

**Will you make a version for [other site]?**  
No, but you're welcome to fork this and do it yourself! The changes I've already made in the code should make it a lot easier to port for another site, as various views and handlers exist already.

## Using Ao3 Reader
### "One Click" Setup
1. Connect your Kobo device to the computer
2. Download the [one click install file for AO3Reader](https://seam.rip/ao3reader/OCP-ao3reader-0.1.0.zip)
3. Unzip into the root of the Kobo (KOBOeReader drive)
4. Add the following lines to the .kobo/Kobo/Kobo eReader.conf file
```
[FeatureSettings]
ExcludeSyncFolders=(\\.(?!kobo|adobe).+|([^.][^/]*/)+\\..+)
```
5. Update the Setting file to add your Ao3 Login and your favorite tags
    * Rename the .adds/ao3reader/Settings-sample.toml file to Settings.toml
    * [Optional] Setup login - this allows you to access your Marked For Later, and any archive-locked fics
        * Set the ```username``` value to your Ao3 username
        * Set the ```password``` value to your Ao3 password
    * [Optional] Setup favorite tags
        * On the line that looks like ```faves=[]```, add any favorite tags in the form ```["Tag Name", "Tag URL"]```, with individual tags seperated by commas
    * Note: Although both login and tags are optional because the reader will still technically work, the current beta does not provide a way to look up an arbitrary tag. If you do neither, you will just get a blank screen with no way to read any works
6. Eject your Kobo - It should immediately enter an install cycle that looks like it is updating

## Developing with Docker
### Requirements
* Bash
* Coreutils
    * realpath
    * dirname
    * basename
* Findutils
    * xargs
* Docker
* X11 or Wayland

### First Time Setup
* Run `./containers/development.sh build-docker-image` to build the development Docker image (this may take a few minutes)
* Run `./containers/development.sh run-docker-image` to run the development Docker image.  This will change your terminal to be inside the Docker container.
* In the container, run `./containers/development.sh build-dependencies` to build AO3 Reader's third party dependencies (this may take a while)
* In the container, run `./containers/development.sh run-emulator` to run AO3 Reader's emulator (this may take a while on the first time)

### Subsequent Times
* Run `./containers/development.sh run-docker-image` to run the development Docker image. This will change your terminal to be inside the Docker container.
* In the container, run `./containers/development.sh run-emulator` to run AO3 Reader's emulator (this may take a while on the first time)

### Development
Edit AO3 Reader's source code with your favorite code editor or IDE. Then, _in the container_, run `./containers/development.sh run-emulator` to run and test your changes. Your code changes are immediately available in the container, there is no need to copy files to and fro.

### Testing One-time Set up (must be run in docker container)
* Install llvm coverage: ```cargo install cargo-llvm-cov```
* Go to crates/core/TestSettings.toml and add your AO3 username and password

### Testing Commands
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
