use crate::UpdateInfo;

pub struct TabViewer<'a> {
    pub info: &'a UpdateInfo<'a>,
}

impl<'a> egui_dock::TabViewer for TabViewer<'a> {
    type Tab = Box<dyn Tab>;

    fn title(&mut self, tab: &mut Self::Tab) -> egui::WidgetText {
        tab.name().into()
    }

    fn ui(&mut self, ui: &mut egui::Ui, tab: &mut Self::Tab) {
        tab.show(ui, self.info);
    }
}

pub trait Tab {
    fn name(&self) -> String;

    fn show(&mut self, ui: &mut egui::Ui, info: &UpdateInfo<'_>);
}

pub type Tree = egui_dock::Tree<Box<dyn Tab>>;
