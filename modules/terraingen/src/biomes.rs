use bevy::prelude::*;
use lerp::Lerp;

use crate::resources::mesh::NoiseValues;

#[derive(Default, Debug, Clone, Component, Reflect)]
pub enum Biomes {
    // Water
    DeepSea,
    Sea,
    River,
    // Sand
    Beach,
    Desert,
    TemperateDesert,
    // Forest
    #[default]
    Grassland,
    Forest,
    Jungle,
    RainForest,
    Thundra,
    // Landscape
    Mountain,
    SnowyMountain,
    MushyMountain,
    // Winter
    Snowy,
    Permafrost,
}

impl Biomes {
    pub fn get_biome(
        noise: NoiseValues,
        y: u32,
        map_height: u32,
        min_terrain_level: f64,
        max_terrain_level: f64,
        min_temp: f64,
        max_temp: f64,
    ) -> Self {
        let map_height_diff = max_terrain_level - min_terrain_level;
        let water_level = if min_terrain_level > 0. {
            -1.
        } else {
            (0. - min_terrain_level) / map_height_diff
        };

        let mountain_level = min_terrain_level.lerp(max_terrain_level, 0.9) / map_height_diff;
        let temp_lerp = min_temp.lerp(max_temp, noise.temperature);
        let below_zero_temp_lerp = if min_temp >= 0. {
            None
        } else {
            Some(min_temp.lerp(max_temp, 0.0))
        };
        let map_lat = y as f64 / map_height as f64;

        match noise {
            // Winter
            NoiseValues {
                height, moisture, ..
            } if height > water_level
                && moisture > 0.6
                && below_zero_temp_lerp.map_or(false, |t| temp_lerp < t) =>
            {
                Self::Thundra
            }
            NoiseValues {
                height, moisture, ..
            } if height > water_level
                && moisture < 0.2
                && below_zero_temp_lerp.map_or(false, |t| temp_lerp < t) =>
            {
                Self::TemperateDesert
            }
            NoiseValues {
                height,
                temperature,
                ..
            } if height > water_level
                && (map_lat > 0.9 + temperature / 10. || map_lat < 0.1 - temperature / 10.) =>
            {
                Self::Thundra
            }
            NoiseValues { height, .. }
                if height > water_level
                    && below_zero_temp_lerp.map_or(false, |t| temp_lerp < t) =>
            {
                Self::Snowy
            }

            // Mountains
            NoiseValues {
                height, moisture, ..
            } if height > mountain_level * 1.1 && moisture > 0.8 => Self::SnowyMountain,
            NoiseValues {
                height, moisture, ..
            } if height > mountain_level * 1.1
                && moisture > 0.6
                && below_zero_temp_lerp.map_or(true, |t| temp_lerp >= t) =>
            {
                Self::MushyMountain
            }
            NoiseValues { height, .. }
                if height > mountain_level * 1.1
                    && below_zero_temp_lerp.map_or(true, |t| temp_lerp >= t) =>
            {
                Self::Mountain
            }

            // Land
            NoiseValues {
                height, moisture, ..
            } if height > water_level * 1.1
                && moisture > 0.8
                && below_zero_temp_lerp.map_or(true, |t| temp_lerp >= t) =>
            {
                Self::RainForest
            }
            NoiseValues {
                height, moisture, ..
            } if height > water_level * 1.1
                && moisture > 0.6
                && below_zero_temp_lerp.map_or(true, |t| temp_lerp >= t) =>
            {
                Self::Jungle
            }
            NoiseValues {
                height, moisture, ..
            } if height > water_level * 1.1
                && moisture > 0.45
                && below_zero_temp_lerp.map_or(true, |t| temp_lerp >= t) =>
            {
                Self::Forest
            }
            NoiseValues {
                height, moisture, ..
            } if height > water_level
                && moisture < 0.25
                && below_zero_temp_lerp.map_or(true, |t| temp_lerp >= t) =>
            {
                Self::Desert
            }
            NoiseValues { height, .. }
                if height >= water_level
                    && height <= water_level * 1.1
                    && below_zero_temp_lerp.map_or(true, |t| temp_lerp >= t) =>
            {
                Self::Beach
            }
            // Else grassland
            NoiseValues {
                height, moisture, ..
            } if height < water_level
                && moisture > 0.6
                && below_zero_temp_lerp.map_or(true, |t| temp_lerp >= t) =>
            {
                Self::DeepSea
            }
            NoiseValues { height, .. }
                if height < water_level
                    && below_zero_temp_lerp.map_or(true, |t| temp_lerp >= t) =>
            {
                Self::Sea
            }
            NoiseValues { height, .. }
                if height < water_level
                    && (below_zero_temp_lerp.map_or(false, |t| temp_lerp <= t)
                        || map_lat > 0.9
                        || map_lat < 0.1) =>
            {
                Self::Permafrost
            }
            _ => Self::Grassland,
        }
    }

    pub fn color(&self) -> Color {
        match self {
            Biomes::DeepSea => Color::MIDNIGHT_BLUE,
            Biomes::Sea => Color::BLUE,
            Biomes::River => Color::ALICE_BLUE,
            Biomes::Beach => Color::BEIGE,
            Biomes::Desert => Color::BISQUE,
            Biomes::TemperateDesert => Color::YELLOW_GREEN,
            Biomes::Grassland => Color::LIME_GREEN,
            Biomes::Forest => Color::DARK_GREEN,
            Biomes::Jungle => Color::GREEN,
            Biomes::RainForest => Color::rgb(0.0, 0.5, 0.2),
            Biomes::Thundra => Color::rgb(0.0, 0.3, 0.1),
            Biomes::Mountain => Color::GRAY,
            Biomes::SnowyMountain => todo!(),
            Biomes::MushyMountain => Color::MAROON,
            Biomes::Snowy => Color::WHITE,
            Biomes::Permafrost => Color::rgb(0.9, 0.9, 0.95),
        }
    }
}
