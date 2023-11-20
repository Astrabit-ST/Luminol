/// Custom implementation of `eframe::Frame` for Luminol.
/// We need this because the normal `eframe::App` uses a struct with private fields in its
/// definition of `update()`, and that prevents us from implementing custom app runners.
pub struct CustomFrame<'a>(
    #[cfg(not(target_arch = "wasm32"))] pub &'a mut eframe::Frame,
    #[cfg(target_arch = "wasm32")] pub std::marker::PhantomData<&'a ()>,
);

#[cfg(not(target_arch = "wasm32"))]
impl std::ops::Deref for CustomFrame<'_> {
    type Target = eframe::Frame;
    fn deref(&self) -> &Self::Target {
        self.0
    }
}

#[cfg(not(target_arch = "wasm32"))]
impl std::ops::DerefMut for CustomFrame<'_> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.0
    }
}

/// Custom implementation of `eframe::App` for Luminol.
/// We need this because the normal `eframe::App` uses a struct with private fields in its
/// definition of `update()`, and that prevents us from implementing custom app runners.
pub trait CustomApp
where
    Self: eframe::App,
{
    fn custom_update(&mut self, ctx: &egui::Context, frame: &mut CustomFrame<'_>);
}

#[macro_export]
macro_rules! app_use_custom_update {
    () => {
        fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
            #[cfg(not(target_arch = "wasm32"))]
            self.custom_update(ctx, &mut CustomFrame(frame))
        }
    };
}
