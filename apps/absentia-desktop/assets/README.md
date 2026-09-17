# Icons

## The window icon

`icon.png` is built into the program at compile time and is what appears in the
title bar, the taskbar and the alt-tab switcher.

Replace it and rebuild:

```
cargo build --release
```

Any size works; 256x256 is a good choice. It must be a PNG, and transparency is
kept.

If this file is missing, the build generates one rather than failing. That
matters more than it sounds: the first version used `include_bytes!`, which
resolves at compile time, so anybody whose copy of `icon.png` had not arrived
could not compile the program at all. A decoration had become a build
dependency.

If the file is present but malformed, the program starts with the toolkit's
default icon rather than refusing to run.

## The executable icon on Windows

Separate, and it does a different job: this is what File Explorer shows for
`absentia.exe` before anybody runs it. It has to be embedded by the linker, so
it needs a build script and a second file.

This is **not set up by default**, deliberately. It requires a Windows-only
dependency that could not be built or tested in the environment this project
was developed in, and shipping an untested Windows build step has broken this
project three times already. Adding it yourself takes two minutes and you will
find out immediately whether it worked.

**1.** Convert your PNG to an `.ico` holding several sizes, at least 16, 32,
48 and 256 pixels. Any converter will do this; ImageMagick is
`magick icon.png -define icon:auto-resize=256,48,32,16 icon.ico`.

Save it as `apps/absentia-desktop/assets/icon.ico`.

**2.** Add to `apps/absentia-desktop/Cargo.toml`:

```toml
[target.'cfg(windows)'.build-dependencies]
winresource = "0.1"
```

**3.** Create `apps/absentia-desktop/build.rs`:

```rust
fn main() {
    println!("cargo:rerun-if-changed=assets/icon.ico");

    #[cfg(target_os = "windows")]
    {
        if std::path::Path::new("assets/icon.ico").exists() {
            let mut res = winresource::WindowsResource::new();
            res.set_icon("assets/icon.ico");
            if let Err(e) = res.compile() {
                // A cosmetic failure must not break the build.
                println!("cargo:warning=could not embed the icon: {e}");
            }
        }
    }
}
```

**4.** `cargo build --release`, then look at `target\release\absentia.exe` in
Explorer. Windows caches icons aggressively, so if it looks unchanged, copy the
file to a new name to see the real result.

Nothing happens on macOS or Linux: there an icon belongs to a `.app` bundle or
a `.desktop` entry rather than to the executable, and a plain `cargo build`
produces neither.

## What ships now

`icon.png` is a placeholder: a ring and a keyhole in the program's own colours.
It exists so the window does not show the toolkit's default letter. Replace it
with something of your own.
