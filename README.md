# Luminol

![Crates.io](https://img.shields.io/crates/v/luminol)![Crates.io](https://img.shields.io/crates/l/luminol)![Crates.io](https://img.shields.io/crates/d/luminol)[![wakatime](https://wakatime.com/badge/user/5cff5352-cb55-44dc-819e-b47f231dcfa2/project/edee199a-95c3-4206-b23e-eb6f0a7e06ba.svg)](https://wakatime.com/badge/user/5cff5352-cb55-44dc-819e-b47f231dcfa2/project/edee199a-95c3-4206-b23e-eb6f0a7e06ba)

Luminol is an experimental remake of the RGSS RPG Maker editors in Rust with love ❤️.

Luminol targets wasm and native builds with eframe. Luminol also temporarily uses [Rusty Object Notation](https://github.com/ron-rs/ron) (`.ron`) for serialization.
Marshal `.rxdata` is planned, and a custom `.lumina` format is also planned.

Luminol _may_ use `Lua` for plugins in the future. It is something I am actively looking into.

You can obtain RON versions of RPG Maker XP data using [rmxp_extractor](https://rubygems.org/gems/rmxp_extractor).
Run `rmxp_extractor export ron` in your project folder to fully export all of your data. Due to pretty printing it may take unusually long.
Using linux to do this is best since Ruby is very fast on linux.

---

## RGSS version support
Luminol is compatible only with **RGSS1** for now. RGSS2 & 3 use different tileset formats which Luminol does not support.
There are plans to support them in the future, though. 
Lily (Luminol's main contributor) does not have a copy of VX or VX Ace yet, so until then Luminol is focused on RGSS1.

Luminol, however will have compatibility modes for various RGSS1 compatible runtimes, usually enabling extra features.
Compatibility:
- RGSS1: Equivalent to RPG Maker XP
- mkxp/mkxp-freebird: Has extra layers
- mkxp-z: Has extra layers, support for playing movies, etc
- ModShot: (Luminol's target) extra layers, OpenAL effects, ruby gem support?
- rsgss: Likely the same as ModShot

---

## Running luminol

wasm builds are deployed to [luminol.dev](https://luminol.dev/#dev)! They work great and are deployed using the awesome [trunk](https://trunkrs.dev)

Native builds are the main focus at the moment, but no official releases will be made until Luminol os stable and usable.
