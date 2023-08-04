pub mod component;
pub mod spawn_system;
pub mod save;
pub mod load;

use bevy::prelude::*;
use bevy_egui::*;
use bevy_scene_hook::HookPlugin;

use crate::{asset_insector::AssetDetectorPlugin, inspector::{InspectorPlugin, registration::EditorRegistryExt}};

use self::{spawn_system::spawn_scene, component::{ScenePrefab, PrefabAutoChild}, save::{SaveState, SaveConfig, SaveLoadPrefabPlugin}};

pub struct PrefabPlugin;

impl Plugin for PrefabPlugin {
    fn build(&self, app: &mut App) {
        if !app.is_plugin_added::<AssetDetectorPlugin>() {
            app.add_plugins(AssetDetectorPlugin);
        }
        if !app.is_plugin_added::<HookPlugin>() {
            app.add_plugins(HookPlugin);
        }
        
        if app.is_plugin_added::<InspectorPlugin>() {
            app.editor_registry::<ScenePrefab>();
            app.editor_registry::<PrefabAutoChild>();
        }

        app.add_systems(Update, spawn_scene);

        app.add_plugins(SaveLoadPrefabPlugin);

    }
}