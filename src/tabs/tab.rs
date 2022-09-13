use std::cell::RefCell;

use super::started::Started;
use crate::UpdateInfo;

pub type Tree = egui_dock::Tree<Box<dyn Tab>>;

/// Helper struct for tabs.
pub struct Tabs {
    tree: RefCell<Tree>,
}

impl Default for Tabs {
    fn default() -> Self {
        // Add the basic "get started" tab
        Self {
            tree: RefCell::new(Tree::new(vec![Box::new(Started::new())])),
        }
    }
}

impl Tabs {
    pub fn ui(&self, ui: &mut egui::Ui, info: &UpdateInfo<'_>) {
        ui.group(|ui| {
            egui_dock::DockArea::new(&mut self.tree.borrow_mut())
                .show_inside(ui, &mut TabViewer { info });
        });
    }

    pub fn add_tab<T>(&self, tab: T)
    where
        T: Tab + 'static,
    {
        self.tree.borrow_mut().push_to_focused_leaf(Box::new(tab));
    }

    pub fn clean_tabs(&self) {
        todo!()
    }
}

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

    fn requires_filesystem(&self) -> bool {
        false
    }
}
