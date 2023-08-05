use std::fs;
use std::path::Path;
use bevy::prelude::*;
use bevy_egui::*;

pub struct EditorAsset {
    pub path: String,
    pub ext: String,
}

#[derive(Resource, Default)]
pub struct DetectedAssets {
    pub assets : Vec<EditorAsset>
}

pub struct AssetDetectorPlugin;

impl Plugin for AssetDetectorPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<DetectedAssets>();

        app.add_systems(Startup, detect_assets);
    }
}

fn detect_assets(
    mut assets : ResMut<DetectedAssets>
) {
    get_assets_in_directory(&Path::new("assets"), &mut assets.assets);
}

fn get_assets_in_directory(dir_path: &Path, assets: &mut Vec<EditorAsset>) {
    if let Ok(entries) = fs::read_dir(dir_path) {
        for entry in entries {
            if let Ok(entry) = entry {
                let path = entry.path();
                if path.is_file() {
                    if let Some(ext) = path.extension() {
                        let editor_asset = EditorAsset {
                            path: path.to_str().unwrap().to_string(),
                            ext: ext.to_str().unwrap().to_string(),
                        };
                        assets.push(editor_asset);
                    }
                } else if path.is_dir() {
                    get_assets_in_directory(&path, assets);
                }
            }
        }
    }
}