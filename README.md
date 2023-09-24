# Luminol

## LUMINOL IS LOOKING FOR CONTRIBUTORS! PLEASE CONTACT leelee.rs ON DISCORD OR EMAIL <lily@nowaffles.com> IF YOU WANT TO HELP

![Crates.io](https://img.shields.io/crates/v/luminol)![Crates.io](https://img.shields.io/crates/l/luminol)![Crates.io](https://img.shields.io/crates/d/luminol)[![wakatime](https://wakatime.com/badge/user/5cff5352-cb55-44dc-819e-b47f231dcfa2/project/edee199a-95c3-4206-b23e-eb6f0a7e06ba.svg)](https://wakatime.com/badge/user/5cff5352-cb55-44dc-819e-b47f231dcfa2/project/edee199a-95c3-4206-b23e-eb6f0a7e06ba)![GitHub code size in bytes](https://img.shields.io/github/languages/code-size/Astrabit-ST/Luminol)[![CI](https://github.com/Astrabit-ST/Luminol/actions/workflows/rust.yml/badge.svg)](https://github.com/Astrabit-ST/Luminol/actions/workflows/rust.yml)![GitHub issues](https://img.shields.io/github/issues/Astrabit-ST/Luminol)

Luminol is an experimental remake of the RGSS RPG Maker editors in Rust with love ❤️.

Luminol targets native builds with eframe. Luminol currently reads *only* rxdata (not rvdata or rvdata2, sorry VX and VX Ace users). In the past, Luminol used to exclusively read rusty object notation (ron) files made from [rmxp_extractor](https://github.com/Speak2Erase/rmxp-extractor). Now, it uses [alox-48](https://github.com/Speak2Erase/alox-48) to deserialize rxdata. It is not 100% perfect, if it does not open your project properly, [please file an issue](https://github.com/Astrabit-ST/Luminol/issues).

In the future a custom .lumina format is planned, as well as ron, rvdata 1 & 2, and json.

Luminol *may* use `Lua` for plugins in the future. It is something I am actively looking into.

## Credits

- [@Speak2Erase](https://github.com/Speak2Erase): Luminol's main contributor
- [@somedevfox](https://github.com/somedevfox): Occasional contributor and creator of rsgss (a sister project of Luminol)
- [@white-axe](https://github.com/white-axe): New contributor
- [@Lionmeow](https://github.com/Lionmeow): Designer of Luminol's icon and Lumi

## RGSS version support

Luminol is compatible only with **RGSS1** for now. RGSS2 & 3 use different tileset formats which Luminol does not support.
There are plans to support them in the future, though.

~~Lily (Luminol's main contributor) does not have a copy of VX or VX Ace yet, so until then Luminol is focused on RGSS1. If you want, [you can buy her a copy](https://steamcommunity.com/id/lily-panpan/).~~

Scratch that, thank you to [bobhostern?](https://steamcommunity.com/id/bobhostern/) for buying Lily VX Ace.

Luminol, however will have compatibility modes for various RGSS1 compatible runtimes, usually enabling extra features.
Compatibility:

- RGSS1: Equivalent to RPG Maker XP
- mkxp/mkxp-freebird: Has extra layers
- mkxp-z: Has extra layers, support for playing movies, etc
- ModShot: (Luminol's target) extra layers, OpenAL effects, ruby gem support?
- rsgss: Likely the same as ModShot

## Running luminol

Native builds are the main focus at the moment, but no official releases will be made until Luminol is stable.
Instead, you will have to compile luminol yourself, by grabbing your favorite nightly rust toolchain from [rustup](https://rustup.rs) and running `cargo build`.

If you are on Linux, you will also need to grab clang and mold from your package manager.

Luminol has like a bajillion dependencies right now so it may take upwards of 15 minutes to compile.

**You can not use one of the stable release channels.**

## Functionality

### RPG Maker XP

Basic functionality:

- [x] Load from rxdata
- [x] Load projects
- [x] Make new projects
- [ ] Create new maps
- [ ] Reorder maps
- [ ] Resize maps
- [x] Open events
- [ ] Edit event commands
- [x] View event commands
- [x] Change tiles on map
- [x] Multiple brush types
- [x] Change autotiles on map
- [x] Hardware accelerated tilemap
- [x] Properly render blend modes and opacity
- [x] Sound test
- [ ] Actor editor
- [ ] Class editor
- [ ] Skill editor
- [x] Item editor
- [ ] Weapon editor
- [ ] Armor editor
- [ ] Enemy editor
- [ ] Troop editor
- [ ] State editor
- [ ] Animation editor
- [ ] Tileset editor
- [x] Common event editor
- [ ] System editor
- [x] Script editor

Extra functionality:

- [x] Edit multiple maps at the same time
- [x] Edit multiple events at the same time
- [x] Edit multiple scripts
- [ ] Language server support for script editor?
- [x] Custom event commands
- [x] Procedural event commands
- [ ] Debugger support?
- [ ] Custom data formats
- [ ] Extra layers
- [x] Move route previews
- [ ] r48 style raw manipulation of values
- [ ] Custom themes (sorta implemented)
- [ ] Styling different from egui's
- [ ] Lua plugin API?
- [ ] Text based event editor [based on keke](https://github.com/Astrabit-ST/keke)
- [ ] Extra properties

## Star History

[![Star History Chart](https://api.star-history.com/svg?repos=Astrabit-ST/Luminol&type=Date)](https://star-history.com/#Astrabit-ST/Luminol&Date)
