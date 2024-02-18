use anyhow::{Context, Result};
use std::{fs, io::Write};

use crate::gizmos::{
    CAMERA_GIZMO, DIRECTION_LIGHT_GIZMO, POINT_LIGHT_GIZMO, SPOT_LIGHT_GIZMO, UNKNOWN,
};

pub fn create_assets_filesystem() -> Result<()> {
    create_dir_if_doesnt_exist("assets")?;
    create_dir_if_doesnt_exist("assets/icons")?;
    create_dir_if_doesnt_exist("assets/models")?;
    create_dir_if_doesnt_exist("assets/scenes")?;
    create_dir_if_doesnt_exist("assets/textures")?;
    create_icon_config()?;
    create_camera_gizmo()?;
    create_unknown_gizmo()?;
    create_point_light_gizmo()?;
    create_dir_light_gizmo()?;
    create_spot_light_gizmo()?;
    Ok(())
}

fn create_dir_if_doesnt_exist(path: &str) -> Result<()> {
    if fs::read_dir(path).is_err() {
        fs::create_dir(path).context(format!("Failed to create dir at path: {}", path))?;
    }
    Ok(())
}

fn create_icon_config() -> Result<()> {
    if fs::metadata("assets/icons/editor.icons.ron").is_err() {
        let mut file = fs::OpenOptions::new()
            .create_new(true)
            .append(false)
            .write(true)
            .open("assets/icons/editor.icons.ron")?;
        file.write_all(ICONS_CONFIG_RON.as_bytes())?;
        file.flush()?;
    }
    Ok(())
}

fn create_camera_gizmo() -> Result<()> {
    if fs::metadata("assets/icons/CameraGizmo.png").is_err() {
        let mut file = fs::OpenOptions::new()
            .create_new(true)
            .append(false)
            .write(true)
            .open("assets/icons/CameraGizmo.png")?;
        file.write_all(CAMERA_GIZMO)?;
        file.flush()?;
    }
    Ok(())
}

fn create_unknown_gizmo() -> Result<()> {
    if fs::metadata("assets/icons/unknown.png").is_err() {
        let mut file = fs::OpenOptions::new()
            .create_new(true)
            .append(false)
            .write(true)
            .open("assets/icons/unknown.png")?;
        file.write_all(UNKNOWN)?;
        file.flush()?;
    }
    Ok(())
}

fn create_dir_light_gizmo() -> Result<()> {
    if fs::metadata("assets/icons/DirectionalLightGizmo.png").is_err() {
        let mut file = fs::OpenOptions::new()
            .create_new(true)
            .append(false)
            .write(true)
            .open("assets/icons/DirectionalLightGizmo.png")?;
        file.write_all(DIRECTION_LIGHT_GIZMO)?;
        file.flush()?;
    }
    Ok(())
}
fn create_spot_light_gizmo() -> Result<()> {
    if fs::metadata("assets/icons/SpotLightGizmo.png").is_err() {
        let mut file = fs::OpenOptions::new()
            .create_new(true)
            .append(false)
            .write(true)
            .open("assets/icons/SpotLightGizmo.png")?;
        file.write_all(SPOT_LIGHT_GIZMO)?;
        file.flush()?;
    }
    Ok(())
}
fn create_point_light_gizmo() -> Result<()> {
    if fs::metadata("assets/icons/PointLightGizmo.png").is_err() {
        let mut file = fs::OpenOptions::new()
            .create_new(true)
            .append(false)
            .write(true)
            .open("assets/icons/PointLightGizmo.png")?;
        file.write_all(POINT_LIGHT_GIZMO)?;
        file.flush()?;
    }
    Ok(())
}

const ICONS_CONFIG_RON: &str = r#"
({
	"sphere": Sphere (
		radius: 0.75,
		
	),
	"square": Quad (
		size: (
			2.0,
			2.0
		)
	),
	"unknown": Image (
		path: "icons/unknown.png"
	),
	"directional" : Image (
		path: "icons/DirectionalLightGizmo.png"
	),
	"spot" : Image (
		path: "icons/SpotLightGizmo.png"
	),
	"point" : Image (
		path: "icons/PointLightGizmo.png"
	),
	"camera" : Image (
		path: "icons/CameraGizmo.png"
	)
})
"#;
