
/// A basic trait describing a window that can show itself.
/// A mutable bool is passed to it and is set to false if it is closed.
pub trait Window {
    fn show(&mut self, ctx: &egui::Context, open: &mut bool);

    fn name(&self) -> String;
}