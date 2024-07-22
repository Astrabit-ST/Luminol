> [!IMPORTANT]
> luminol-egui-wgpu is currently based on emilk/egui@0.28.1

> [!NOTE]
> This is Luminol's modified version of egui-wgpu. The original version is dual-licensed under MIT and Apache 2.0.
>
> To merge changes from upstream into this crate, first add egui as a remote:
>
> ```
> git remote add -f --no-tags egui https://github.com/emilk/egui
> ```
>
> Now, decide on which upstream egui commit you want to merge from and figure out the egui commit that the previous upstream merge was based on. The basis of the previous upstream merge should be written at the top of this README. **Please update the top of this README after merging.**
>
> In this example, we are merging from commit `bd087ffb8d7467e0b5aa06d17dd600d511d6a5e8` (egui 0.24.0) and the previous merge was based on commit `5a0186fa2b2324ab437099e456e55e281234ca99` (egui 0.23.0).
>
> ```
> git diff \
>     5a0186fa2b2324ab437099e456e55e281234ca99:crates/egui-wgpu \
>     bd087ffb8d7467e0b5aa06d17dd600d511d6a5e8:crates/egui-wgpu |
>     git apply -3 --directory=crates/egui-wgpu
> ```
>
> Fix any merge conflicts, and then do `git commit`.

# egui-wgpu

[![Latest version](https://img.shields.io/crates/v/egui-wgpu.svg)](https://crates.io/crates/egui-wgpu)
[![Documentation](https://docs.rs/egui-wgpu/badge.svg)](https://docs.rs/egui-wgpu)
![MIT](https://img.shields.io/badge/license-MIT-blue.svg)
![Apache](https://img.shields.io/badge/license-Apache-blue.svg)

This crates provides bindings between [`egui`](https://github.com/emilk/egui) and [wgpu](https://crates.io/crates/wgpu).

This was originally hosted at https://github.com/hasenbanck/egui_wgpu_backend
