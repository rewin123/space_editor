use crate::{
    tab_name::{TabName, TabNameHolder},
    EditorUi,
};
/// This mod contains start layouts templates for tabs
/// It was created for easy customization of shown tabs on startup
use bevy::prelude::*;

pub trait StartLayout {
    fn build(&self) -> egui_dock::DockState<TabNameHolder>;
}

pub trait GroupLayout<G>: StartLayout {
    /// Add tab to group to end
    fn push<N: TabName>(&mut self, group: G, tab: N);
    /// Add tab to group to start
    fn push_front<N: TabName>(&mut self, group: G, tab: N);
}

pub enum DoublePanel {
    TopLeft,
    BottomLeft,
    Main,
}

#[derive(Default, Resource)]
pub struct DoublePanelGroup {
    pub top_panel: Vec<TabNameHolder>,
    pub bottom_panel: Vec<TabNameHolder>,
    pub main_panel: Vec<TabNameHolder>,
}

impl StartLayout for DoublePanelGroup {
    fn build(&self) -> egui_dock::DockState<TabNameHolder> {
        let mut state = egui_dock::DockState::new(self.main_panel.clone());

        let [_game, panels] = state.main_surface_mut().split_left(
            egui_dock::NodeIndex::root(),
            0.2,
            self.top_panel.clone(),
        );

        let [_, _] = state
            .main_surface_mut()
            .split_below(panels, 0.3, self.bottom_panel.clone());

        state
    }
}

impl GroupLayout<DoublePanel> for DoublePanelGroup {
    fn push<N: TabName>(&mut self, group: DoublePanel, tab: N) {
        match group {
            DoublePanel::TopLeft => self.top_panel.push(tab.into()),
            DoublePanel::BottomLeft => self.bottom_panel.push(tab.into()),
            DoublePanel::Main => self.main_panel.push(tab.into()),
        }
    }

    fn push_front<N: TabName>(&mut self, group: DoublePanel, tab: N) {
        match group {
            DoublePanel::TopLeft => self.top_panel.insert(0, tab.into()),
            DoublePanel::BottomLeft => self.bottom_panel.insert(0, tab.into()),
            DoublePanel::Main => self.main_panel.insert(0, tab.into()),
        }
    }
}

pub trait GroupLayoutExt {
    fn layout_push<R: GroupLayout<G> + Resource, N: TabName, G>(
        &mut self,
        group: G,
        tab: N,
    ) -> &mut Self;
    fn layout_push_front<R: GroupLayout<G> + Resource, N: TabName, G>(
        &mut self,
        group: G,
        tab: N,
    ) -> &mut Self;
    fn init_layout_group<R: GroupLayout<G> + Resource + Default, G>(&mut self) -> &mut Self;
}

impl GroupLayoutExt for App {
    fn layout_push<R: GroupLayout<G> + Resource, N: TabName, G>(
        &mut self,
        group: G,
        tab: N,
    ) -> &mut Self {
        if let Some(mut layout) = self.world_mut().get_resource_mut::<R>() {
            layout.push(group, tab)
        }
        self
    }

    fn layout_push_front<R: GroupLayout<G> + Resource, N: TabName, G>(
        &mut self,
        group: G,
        tab: N,
    ) -> &mut Self {
        if let Some(mut layout) = self.world_mut().get_resource_mut::<R>() {
            layout.push_front(group, tab)
        }
        self
    }

    fn init_layout_group<R: GroupLayout<G> + Resource + Default, G>(&mut self) -> &mut Self {
        self.init_resource::<R>();
        self.add_systems(Startup, |mut editor: ResMut<EditorUi>, layout: Res<R>| {
            editor.set_layout(layout.as_ref());
        })
    }
}
