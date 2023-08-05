pub mod component;
pub mod spawn_system;
pub mod save;
pub mod load;

use bevy::prelude::*;
use bevy_scene_hook::HookPlugin;

use crate::{asset_insector::AssetDetectorPlugin, inspector::{InspectorPlugin, registration::EditorRegistryExt}};

use component::*;
use spawn_system::*;
use save::*;
use load::*;

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
            app.editor_registry::<GltfPrefab>();
        }

        app.add_systems(Update, spawn_scene);

        app.add_plugins(SavePrefabPlugin);
        app.add_plugins(LoadPlugin);

    }
}