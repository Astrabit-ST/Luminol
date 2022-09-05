/// A basic trait describing a window that can show itself.
/// A mutable bool is passed to it and is set to false if it is closed.
pub trait Window {
    fn show(
        &mut self,
        ctx: &egui::Context,
        open: &mut bool,
        data_cache: Option<&mut crate::filesystem::data_cache::DataCache>,
    );

    /// Required to prevent duplication.
    fn name(&self) -> String;

    ///  A function to determine if this window needs the data cache.
    fn requires_cache(&self) -> bool {
        false
    }
}
