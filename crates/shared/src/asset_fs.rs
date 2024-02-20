use anyhow::Result;
use bevy::prelude::*;
use image;

use crate::gizmos::{
    CAMERA_GIZMO, DIRECTION_LIGHT_GIZMO, POINT_LIGHT_GIZMO, SPOT_LIGHT_GIZMO, UNKNOWN,
};

pub fn create_camera_image() -> Result<Image> {
    let image = image::load_from_memory_with_format(CAMERA_GIZMO, image::ImageFormat::Png)?;
    let image = Image::from_dynamic(image, false);
    Ok(image)
}

pub fn create_unknown_image() -> Result<Image> {
    let image = image::load_from_memory_with_format(UNKNOWN, image::ImageFormat::Png)?;
    let image = Image::from_dynamic(image, false);
    Ok(image)
}

pub fn create_dir_light_image() -> Result<Image> {
    let image =
        image::load_from_memory_with_format(DIRECTION_LIGHT_GIZMO, image::ImageFormat::Png)?;
    let image = Image::from_dynamic(image, false);
    Ok(image)
}
pub fn create_spot_light_image() -> Result<Image> {
    let image = image::load_from_memory_with_format(SPOT_LIGHT_GIZMO, image::ImageFormat::Png)?;
    let image = Image::from_dynamic(image, false);
    Ok(image)
}
pub fn create_point_light_image() -> Result<Image> {
    let image = image::load_from_memory_with_format(POINT_LIGHT_GIZMO, image::ImageFormat::Png)?;
    let image = Image::from_dynamic(image, false);
    Ok(image)
}
