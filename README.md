# Luminol

[![wakatime](https://wakatime.com/badge/user/5cff5352-cb55-44dc-819e-b47f231dcfa2/project/edee199a-95c3-4206-b23e-eb6f0a7e06ba.svg)](https://wakatime.com/badge/user/5cff5352-cb55-44dc-819e-b47f231dcfa2/project/edee199a-95c3-4206-b23e-eb6f0a7e06ba)![GitHub code size in bytes](https://img.shields.io/github/languages/code-size/Astrabit-ST/Luminol)[![Build status](https://img.shields.io/github/actions/workflow/status/Astrabit-ST/Luminol/build.yml)](https://github.com/Astrabit-ST/Luminol/actions/workflows/rust.yml)![GitHub issues](https://img.shields.io/github/issues/Astrabit-ST/Luminol)![v1.0](https://img.shields.io/github/milestones/progress/Astrabit-ST/Luminol/1?logo=steam&label=Steam%20release%20progress)

Luminol is an experimental remake of the RGSS RPG Maker editors in Rust with love ❤️.

### Join [our discord](https://discord.gg/8jZKmesKJy) if you're interested in the project!

Luminol targets native builds with eframe. Luminol currently reads *only* rxdata (not rvdata or rvdata2, sorry VX and VX Ace users). In the past, Luminol used to exclusively read rusty object notation (ron) files made from [rmxp_extractor](https://github.com/Speak2Erase/rmxp-extractor). Now, it uses [alox-48](https://github.com/Speak2Erase/alox-48) to deserialize rxdata. It is not 100% perfect, if it does not open your project properly, [please file an issue](https://github.com/Astrabit-ST/Luminol/issues).

In the future a custom `.lumina` format is planned, as well as [ron](https://github.com/ron-rs/ron), `rvdata1` & `rvdata1`, and `json`.

Luminol *may* use `Lua` for plugins in the future. It is something I am actively looking into.

## RGSS version support

Luminol is compatible only with **RGSS1** for now. RGSS2 & 3 use different tileset formats which Luminol does not support (yet).
There are plans to support them in the future, though.

~~Melody (Luminol's main contributor) does not have a copy of VX or VX Ace yet, so until then Luminol is focused on RGSS1. If you want, [you can buy her a copy](https://steamcommunity.com/id/melody-rs/).~~

Scratch that, thank you to [bobhostern?](https://steamcommunity.com/id/bobhostern/) for buying Melody VX Ace.

Luminol, however will have compatibility modes for various RGSS1 compatible runtimes, usually enabling extra features.

## Browser support

For the foreseeable future, Luminol can't support Firefox due to [Mozilla's stance on the Filesystem Access API](https://mozilla.github.io/standards-positions/).
Aside from Firefox, any recent chromium based browser should support Luminol!

This includes Chrome (obviously) as well as Opera and Edge. 
If you're on Linux at the moment for best performance you'll need Chrome canary as Google hasn't stabilized Linux WebGPU support yet.

## Running luminol

Native builds are the main focus at the moment, but no official releases will be made until Luminol is stable.
If you want to test out Luminol anyway, you can grab a build from [our build workflow](https://github.com/Astrabit-ST/Luminol/actions/workflows/build.yml). 
It's currently WIP, but there's [a website](https://luminol.dev) where you can try the latest development build of Luminol!

If you'd like to compile luminol yourself, you can by grabbing your favorite nightly rust toolchain from [rustup](https://rustup.rs) and running `cargo build`.
Additionally, to enable steamworks support pass `--features steamworks` to `cargo build`.

Once cargo is finished compiling, the Luminol binary should be located at `target/release/luminol`. 

If you enabled steamworks support you'll also need to place the steamworks redistributable from `steamworks/redistributable_bin/` alongside your Luminol binary.

If you are on Linux, you will also need to grab `clang` and `mold` from your package manager. 
If your particular distro doesn't have those (or you can't use them) you can comment out these lines in [.cargo/config.toml](/.cargo/config.toml):
```toml
[target.x86_64-unknown-linux-gnu]
rustflags = [
	"-C",
	"linker=clang",
	"-C",
	"link-arg=-fuse-ld=mold",
	"-Z",
	"threads=8",
]
```

We've also turned on the unstable `-Z threads=8` compiler flag to speed up build times. 

This is a pretty unstable feature at the moment and may cause compiler deadlocks.
Luckily cargo will detect when that happens and halt your build. Re-running `cargo build` continue your build without issue, though.

Luminol has like a bajillion dependencies right now so it may take upwards of 15 minutes to compile!

Luminol's native build currently can compile on stable Rust, however we pin the toolchain to nightly for wasm32 and the aforementioned `-Z threads=8` flag.

## Credits

- [@Speak2Erase](https://github.com/Speak2Erase): Luminol's creator
- [@somedevfox](https://github.com/somedevfox): Occasional contributor and creator of rsgss (a sister project of Luminol)
- [@white-axe](https://github.com/white-axe): Brought back Luminol's Web build
- [@Lionmeow](https://github.com/Lionmeow): Designer of Luminol's icon and Lumi

## Functionality

Please see [FUNCTIONALITY.md](/FUNCTIONALITY.md)