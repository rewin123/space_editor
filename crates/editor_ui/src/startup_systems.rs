use bevy::prelude::*;
use bevy_egui::*;


fn install_image_loaders(mut ctx: EguiContexts) {
    egui_extras::install_image_loaders(ctx.ctx_mut());
}

pub struct StartupSystems;

impl Plugin for StartupSystems {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, install_image_loaders);
    }
}
