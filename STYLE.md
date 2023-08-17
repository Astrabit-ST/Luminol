# Luminol code style guidelines

This document defines how Luminol's codebase should be structured, formatted and written in a ruleset format.

## Table of contents

1. [Unsafe code][unsafe]
2. [Mutability][mutability]
3. [Allocations][allocations]

## Guidelines

### [1][unsafe] **Unsafe Code**

This section defines where, when, and how to write unsafe code.

- [1.1][unsafe] DO NOT WRITE UNSAFE CODE.
- [1.2][unsafe] Unsafe code is permissible if
    1) A library has a poor constraint, or marks something as unsafe when it is safe
    2) The rust compiler does not understand something that would be otherwise safe
    3) You have no other way of doing it that would not serverely hinder code readability
    - Examples:
        - rodio/cpal's audio types are !Send and !Sync due to them not being thread safe on Android (we do not support Android), so it's okay to `impl Send` on types that contain it.
        ```rs
        // src/audio.rs
        pub struct Audio {
            inner: Mutex<Inner>, // Use a mutex so it's at least *slightly* more thread safe.
        }

        struct Inner {
            _output_stream: OutputStream,
            output_stream_handle: OutputStreamHandle,
            sinks: HashMap<Source, Sink>,
        }

        // We do not support android, and audio on android is the reason why OutputStream is not `Send`. This is okay.
        #[allow(unsafe_code)]
        unsafe impl Send for Inner {}
        ```
        - The rust compiler does not support self referential types. Unsafe code is required to use them properly with nested refcells.
        ```rs
        pub fn map<'a>(&'a self, id: usize) -> impl Deref<Target = rpg::Map> + DerefMut + 'a {
            struct Ref<'b> {
                _state: atomic_refcell::AtomicRef<'b, State>,
                map_ref: dashmap::mapref::one::RefMut<'b, usize, rpg::Map>,
            }
            impl<'b> Deref for Ref<'b> {
                type Target = rpg::Map;

                fn deref(&self) -> &Self::Target {
                    &self.map_ref
                }
            }
            impl<'b> DerefMut for Ref<'b> {
                fn deref_mut(&mut self) -> &mut Self::Target {
                    &mut self.map_ref
                }
            }

            let state = self.state.borrow();
            let State::Loaded { ref maps, .. } = &*state else {
                panic!("project not loaded")
            };
            //? # SAFETY
            // For starters, this has been tested against miri. Miri is okay with it.
            // Ref is self referential- map_ref borrows from _state. We need to store _state so it gets dropped at the same time as map_ref.
            // If it didn't, map_ref would invalidate the refcell. We could unload the project, changing State while map_ref is live. Storing _state prevents this.
            // Because the rust borrow checker isn't smart enough for this, we need to create an unbounded reference to maps to get a map out. We're not actually using this reference
            // for any longer than it would be valid for (notice the fact that we assign map_ref a lifetime of 'a, which is the lifetime it should have anyway) so this is okay.
            let map_ref: dashmap::mapref::one::RefMut<'a, _, _> = unsafe {
                let unsafe_maps_ref: &dashmap::DashMap<usize, rpg::Map> = &*(maps as *const _);
                unsafe_maps_ref
                    .entry(id)
                    .or_try_insert_with(|| {
                        state!()
                            .filesystem
                            .read_data(format!("Data/Map{id:0>3}.{}", self.rxdata_ext()))
                    })
                    .expect("failed to load map") // FIXME
            };

            Ref {
                _state: state,
                map_ref,
            }
        }
        ```
        - Using libraries like OpenGL is naturally unsafe. If they are a necessity, then unsafe code is okay, but PLEASE validate it as much as possible first.
- [1.3][unsafe] When writing unsafe code, use as little as possible. i.e. when using `unsafe impl Send`, avoid `unsafe impl Sync` and use a Mutex.
- [1.4][unsafe] Storing raw pointers is heavily discouraged. In general, keep pointers on the stack and do not store them.
- [1.5][unsafe] Impling `Send` and `Sync` is nice as a stopgap as Luminol generally does not use multithreaded coded.
    - [1.5.1][unsafe] You can use them to test things, but they are disallowed in pull requests.
    - [1.5.2][unsafe] If you can prove that they are actually safe, then they are allowed (see the rodio/cpal example.)

### [2][mutability] **Mutability rules **

This section outlines when to take `&mut self`.

- [1.1][mutability] Prefer taking `&mut self`.
    - [1.1.1] Allow the caller to determine how to handle mutability.
- [1.2][mutability] If it is not possible to use `&mut self` (i.e. shared ownership via an Arc is required) try and use atomics.
    - [1.2.1][mutability] Use crossbeam's `Atomic<T>` type when dealing with non-primitive types that are `Copy`.
- [1.3][mutability] If you can't use `Atomic<T>`, use `AtomicRefCell<T>`.
    - [1.3.1][mutability] `AtomicRefCell<T>` is preferred over `RwLock<T>`, which is preferred over `Mutex<T>`.
    - [1.3.2][mutability] Always use `Mutex<T>` when a type does not impl `Sync`, rather than using `unsafe impl Sync`. See [1.3][unsafe] of unsafe code guidelines.

### [3][allocations] **Allocations**

This section covers when and when not to allocate memory.

- [1.1][allocations] Avoid allocations where possible.
    - [1.1.1][allocations] The usage of `format!` and `vec!` is okay, though.
    - [1.1.2][allocations] When making allocations of collections, try and predict a size (i.e. using `String::with_capacity`)
- [1.2][allocations] If you are concerned about a potential allocation hotspot, try checking that!
- [1.3][allocations] If a resource is going to be loaded into memory several times, and it is easy and sensible to cache it (i.e. textures), write a cache module.

### [4][panics] **Panics**

This section outlines when to panic and when not to panic.

- [1.1] Unwrap is okay when there's certain code condition that you generally expect to be true.
- [1.2] Explicit panics are okay when an intangible/unintended state is achieved.
    ```rs
    pub fn map<'a>(&'a self, id: usize) -> impl Deref<Target = rpg::Map> + DerefMut + 'a {
        struct Ref<'b> {
            _state: atomic_refcell::AtomicRef<'b, State>,
            map_ref: dashmap::mapref::one::RefMut<'b, usize, rpg::Map>,
        }
        // ...

        // We expect to be loaded when this function is called. This function could theoretically be called anywhere, but is generally called by code that relies on the filesystem.
        // When a project is not loaded, that code is never called. It is a logic error for it to be called.
        let state = self.state.borrow();
        let State::Loaded { ref maps, .. } = &*state else {
            panic!("project not loaded")
        };

        // ...
        Ref {
            _state: state,
            map_ref,
        }
    }
    ```

### [5][errors] **Errors**

## Credits

- [**speak2erase**](https://github.com/speak2erase) for writing this guide.


[unsafe]: #1-unsafe-code
[mutability]: #2-mutability-rules
[allocations]: #3-allocations
[panics]: #4-panics
[errors]: #5-errors