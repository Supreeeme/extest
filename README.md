# Extest - X11 XTEST Reimplementation for Steam Controller on Wayland

Extest is a drop in replacement for the X11 XTEST extension. It creates a virtual device with the uinput kernel module.
It's been primarily developed for allowing the desktop functionality on the Steam Controller to work while Steam is open on Wayland.

## Usage

Be sure you have [Rust](https://www.rust-lang.org/learn/get-started) installed.

```sh
cargo build --release
```

This will create a library named `libextest.so` in `target/i686-unknown-linux-gnu/release`.
Note that this library is 32 bit by default because Steam is a 32 bit application.

You will also need to add your user to the `input` group if not added already, so that your user can be allowed to actually create fake devices:

```sh
sudo usermod -a -G input <your username>
```

You can then use `LD_PRELOAD` to override any app that wants to use XTEST functions that have been reimplemented by Extest. Example:

```sh
LD_PRELOAD=/path/to/libextest.so steam
```

The repository also comes with a script to automatically override Steam's desktop file so that whenever you launch Steam from the desktop file
(i.e. via desktop icon or application menu) Extest will be automatically preloaded. Just run it like so:

```sh
./override_steam_desktop_file.sh
```
