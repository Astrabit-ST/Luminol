# Luminol

![Crates.io](https://img.shields.io/crates/v/luminol)![Crates.io](https://img.shields.io/crates/l/luminol)![Crates.io](https://img.shields.io/crates/d/luminol)

Luminol is an experimental remake of the RPG Maker XP editor in Rust with love ❤️.

Luminol targets wasm and native builds with eframe. Luminol also temporarily uses [Rusty Object Notation](https://github.com/ron-rs/ron) (`.ron`) for serialization.
Marshal `.rxdata` is planned, and a custom `.lumina` format is also planned.

Luminol _may_ use `Lua` for plugins in the future. It is something I am actively looking into.

You can obtain RON versions of RPG Maker XP data using [rmxp_extractor](https://rubygems.org/gems/rmxp_extractor).
Run `rmxp_extractor export ron` in your project folder to fully export all of your data. Due to pretty printing it may take unusually long.
Using linux to do this is best since Ruby is very fast on linux.

---

## Running luminol

wasm builds are deployed to [luminol.dev](https://luminol.dev/#dev)! They work great and are deployed using the awesome [trunk](https://trunkrs.dev)

Native builds are the main focus at the moment, but no official releases will be made until Luminol os stable and usable.
