use anyhow::anyhow;
use bevy::{
    math::primitives as math_shapes, prelude::*, render::view::RenderLayers, utils::HashMap,
};
use bevy_asset_loader::{
    asset_collection::AssetCollection,
    dynamic_asset::{DynamicAsset, DynamicAssetCollection},
    prelude::DynamicAssetType,
};
use bevy_mod_billboard::{
    prelude::BillboardPlugin, BillboardMeshHandle, BillboardTextureBundle, BillboardTextureHandle,
};
use bevy_mod_picking::backends::raycast::{
    bevy_mod_raycast::prelude::RaycastVisibility, RaycastBackendSettings,
};
use space_prefab::editor_registry::EditorRegistryExt;
use space_shared::*;

use crate::{EditorGizmo, LAST_RENDER_LAYER};
use space_editor_core::selected::Selected;

#[derive(Default)]
pub struct MeshlessVisualizerPlugin;

impl Plugin for MeshlessVisualizerPlugin {
    #[cfg(not(tarpaulin_include))]
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(EditorState::Editor), register_assets)
            .add_systems(
                Startup,
                |mut next_editor_state: ResMut<NextState<EditorState>>| {
                    next_editor_state.set(EditorState::Editor);
                },
            )
            .insert_resource(RaycastBackendSettings {
                raycast_visibility: RaycastVisibility::Ignore,
                ..Default::default()
            })
            .add_plugins(BillboardPlugin)
            .add_systems(
                Update,
                (visualize_meshless, visualize_custom_meshless).in_set(EditorSet::Editor),
            )
            .editor_registry::<CustomMeshless>();
    }
}

/// Gives the entity some mesh and material to display within the editor
/// Default is a billboard with a quad mesh and question mark icon
#[derive(Component, Clone, Default, Reflect)]
#[reflect(Component, Default)]
pub struct CustomMeshless {
    /// Visual that will be used to show the entity or object
    pub visual: MeshlessModel,
}

/// This determines what a custom entity should use as its editor interactable model if it doesn't
/// have a mesh associated with it.
#[derive(Clone, Reflect)]
pub enum MeshlessModel {
    Billboard {
        mesh: Option<Handle<Mesh>>,     // Default: Quad::new(Vec2::splat(2.))
        texture: Option<Handle<Image>>, // Default: assets/icons/unknown.png
    },
    Object {
        mesh: Option<Handle<Mesh>>, // Default: Icosphere { radius: 0.75, ..default }
        material: Option<Handle<StandardMaterial>>, // Default: StandardMaterial {unlit: true, ..default }
    },
}

impl Default for MeshlessModel {
    fn default() -> Self {
        Self::Billboard {
            mesh: None,
            texture: None,
        }
    }
}

/// Assets to be loaded on app startup
#[derive(AssetCollection, Resource)]
pub struct EditorIconAssets {
    /// Image to be used as a backup
    #[asset(key = "unknown")]
    pub unknown: Handle<Image>,
    /// Image for a directional light
    #[asset(key = "directional")]
    pub directional: Handle<Image>,
    /// Image for a point light
    #[asset(key = "point")]
    pub point: Handle<Image>,
    /// Image for a spot light
    #[asset(key = "spot")]
    pub spot: Handle<Image>,
    /// Image for a camera
    #[asset(key = "camera")]
    pub camera: Handle<Image>,
    /// Mesh that images are put onto
    #[asset(key = "square")]
    pub square: Handle<Mesh>,
    /// Mesh that allows the images to be clickable
    #[asset(key = "sphere")]
    pub sphere: Handle<Mesh>,
}

fn register_assets(mut commands: Commands, asset_server: Res<AssetServer>) {
    use space_shared::asset_fs::*;
    let assets = EditorIconAssets {
        // Unwraps are logged
        unknown: asset_server.add(
            create_unknown_image()
                .inspect_err(|err| error!("failed to load image `Unknown`: {err}"))
                .unwrap(),
        ),
        directional: asset_server.add(
            create_dir_light_image()
                .inspect_err(|err| error!("failed to load image `DirectionalLight`: {err}"))
                .unwrap(),
        ),
        point: asset_server.add(
            create_point_light_image()
                .inspect_err(|err| error!("failed to load image `PointLight`: {err}"))
                .unwrap(),
        ),
        spot: asset_server.add(
            create_spot_light_image()
                .inspect_err(|err| error!("failed to load image `SpotLight`: {err}"))
                .unwrap(),
        ),
        camera: asset_server.add(
            create_camera_image()
                .inspect_err(|err| error!("failed to load image `Camera`: {err}"))
                .unwrap(),
        ),
        square: asset_server.add(math_shapes::Rectangle::new(2., 2.).into()),
        sphere: asset_server.add(Mesh::from(math_shapes::Sphere { radius: 0.75 })),
    };
    commands.insert_resource(assets);
}

#[derive(serde::Deserialize, Asset, TypePath)]
pub struct EditorIconAssetCollection(HashMap<String, EditorIconAssetType>);

impl DynamicAssetCollection for EditorIconAssetCollection {
    fn register(&self, dynamic_assets: &mut bevy_asset_loader::dynamic_asset::DynamicAssets) {
        for (k, ass) in self.0.iter() {
            dynamic_assets.register_asset(k, Box::new(ass.clone()));
        }
    }
}

/// Supported types of icons within the editor to be loaded in
#[derive(serde::Deserialize, Debug, Clone)]
enum EditorIconAssetType {
    /// PNG images for cameras, lights, and audio
    Image { path: String },
    /// Quad mesh for putting images onto
    Quad { size: Vec2 },
    /// Icosphere mesh to make an icon clickable
    Sphere { radius: f32 },
}

impl DynamicAsset for EditorIconAssetType {
    fn load(&self, asset_server: &AssetServer) -> Vec<UntypedHandle> {
        match self {
            Self::Image { path } => vec![asset_server.load::<Image>(path).untyped()],
            _ => vec![],
        }
    }
    fn build(
        &self,
        world: &mut World,
    ) -> Result<bevy_asset_loader::dynamic_asset::DynamicAssetType, anyhow::Error> {
        let asset_server = world
            .get_resource::<AssetServer>()
            .ok_or_else(|| anyhow!("Failed to get the AssetServer"))?;
        match self {
            Self::Image { path } => {
                let handle = asset_server.load::<Image>(path);
                Ok(DynamicAssetType::Single(handle.untyped()))
            }
            Self::Quad { size } => {
                let mut meshes = world
                    .get_resource_mut::<Assets<Mesh>>()
                    .ok_or_else(|| anyhow!("Failed to get Mesh Assets"))?;
                let handle = meshes
                    .add(Mesh::from(math_shapes::Rectangle {
                        half_size: *size * 0.5,
                    }))
                    .untyped();
                Ok(DynamicAssetType::Single(handle))
            }
            Self::Sphere { radius } => {
                let mut meshes = world
                    .get_resource_mut::<Assets<Mesh>>()
                    .ok_or_else(|| anyhow!("Failed to get Mesh Assets"))?;
                let handle = meshes
                    .add(Mesh::from(math_shapes::Sphere { radius: *radius }))
                    .untyped();
                Ok(DynamicAssetType::Single(handle))
            }
        }
    }
}

pub fn visualize_meshless(
    mut commands: Commands,
    lights: Query<
        (
            Entity,
            Option<&Children>,
            AnyOf<(&DirectionalLight, &SpotLight, &PointLight)>,
        ),
        (With<PrefabMarker>, With<Transform>, With<Visibility>),
    >,
    cams: Query<
        (Entity, Option<&Children>),
        (
            With<Camera>,
            With<PrefabMarker>,
            With<Transform>,
            With<Visibility>,
            Without<EditorCameraMarker>,
        ),
    >,
    visualized: Query<&BillboardMeshHandle>,
    editor_icons: Res<EditorIconAssets>,
) {
    for (parent, children, light_type) in &lights {
        // change is none to doesn't contain
        // this then covers the case that lights could have children other than these
        if children.is_none()
            || children.is_some_and(|children| {
                children.iter().all(|child| visualized.get(*child).is_err())
            })
        {
            let image = match light_type {
                (Some(_directional), _, _) => editor_icons.directional.clone(),
                (_, Some(_spot), _) => editor_icons.spot.clone(),
                (_, _, Some(_point)) => editor_icons.point.clone(),
                _ => unreachable!(),
            };
            // creates a mesh for the icon, as well as a clickable sphere that can be selected to interact with the grandparent, being the actual entity in question
            let child = commands
                .spawn((
                    BillboardTextureBundle {
                        mesh: bevy_mod_billboard::BillboardMeshHandle(editor_icons.square.clone()),
                        texture: BillboardTextureHandle(image.clone()),
                        ..default()
                    },
                    RenderLayers::layer(LAST_RENDER_LAYER.into()),
                    Name::from("Billboard Texture"),
                ))
                .with_children(|adult| {
                    adult.spawn((
                        MaterialMeshBundle::<StandardMaterial> {
                            mesh: editor_icons.sphere.clone(),
                            visibility: Visibility::Hidden,
                            ..default()
                        },
                        SelectParent { parent },
                        Name::from("Billboard Mesh"),
                    ));
                })
                .id();
            commands.entity(parent).add_child(child);
        }
    }
    for (parent, children) in &cams {
        if children.is_none()
            || children.is_some_and(|children| {
                children.iter().all(|child| visualized.get(*child).is_err())
            })
        {
            let child = commands
                .spawn((
                    BillboardTextureBundle {
                        mesh: bevy_mod_billboard::BillboardMeshHandle(editor_icons.square.clone()),
                        texture: BillboardTextureHandle(editor_icons.camera.clone()),
                        ..default()
                    },
                    RenderLayers::layer(LAST_RENDER_LAYER.into()),
                    Name::from("Billboard Texture"),
                ))
                .with_children(|adult| {
                    adult.spawn((
                        MaterialMeshBundle::<StandardMaterial> {
                            mesh: editor_icons.sphere.clone(),
                            visibility: Visibility::Hidden,
                            ..default()
                        },
                        SelectParent { parent },
                        Name::from("Billboard Mesh"),
                    ));
                })
                .id();
            commands.entity(parent).add_child(child);
        }
    }
}

/// This will create a way to have any entity with CustomMeshlessMarker have a way to be visualized by the user
/// Additionally, the user can either choose their own mesh and material to use or default to the white sphere
pub fn visualize_custom_meshless(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    objects: Query<(Entity, &CustomMeshless, Option<&Children>)>,
    editor_icons: Res<EditorIconAssets>,
    visualized: Query<&BillboardMeshHandle>,
) {
    for (entity, meshless, children) in objects.iter() {
        if children.is_none()
            || children.is_some_and(|children| {
                children.iter().all(|child| visualized.get(*child).is_err())
            })
        {
            let child = match &meshless.visual {
                MeshlessModel::Billboard {
                    ref mesh,
                    ref texture,
                } => {
                    let Some(texture) = texture.clone() else {
                        return;
                    };
                    commands
                        .spawn((
                            BillboardTextureBundle {
                                mesh: BillboardMeshHandle(mesh.clone().unwrap_or_else(|| {
                                    asset_server.add(math_shapes::Rectangle::new(2., 2.).into())
                                })),
                                texture: BillboardTextureHandle(texture),
                                ..default()
                            },
                            Name::from("Billboard Texture"),
                            RenderLayers::layer(LAST_RENDER_LAYER.into()),
                        ))
                        .with_children(|adult| {
                            adult.spawn((
                                MaterialMeshBundle::<StandardMaterial> {
                                    mesh: editor_icons.sphere.clone(),
                                    visibility: Visibility::Hidden,
                                    ..default()
                                },
                                SelectParent { parent: entity },
                                Name::from("Billboard Mesh"),
                            ));
                        })
                        .id()
                }
                MeshlessModel::Object { mesh, material } => commands
                    .spawn((
                        MaterialMeshBundle {
                            mesh: mesh.clone().unwrap_or_else(|| editor_icons.sphere.clone()),
                            material: material.clone().unwrap_or_else(|| {
                                asset_server.add(StandardMaterial {
                                    unlit: true,
                                    ..default()
                                })
                            }),
                            ..default()
                        },
                        SelectParent { parent: entity },
                        RenderLayers::layer(LAST_RENDER_LAYER.into()),
                        Name::from("Meshless Object"),
                    ))
                    .id(),
            };
            commands.entity(entity).add_child(child);
        }
    }
}

pub fn clean_meshless(
    mut commands: Commands,
    // this covers all entities that are the children of the lights and Cameras
    // this can be extended to cover the custom children as well
    objects: Query<
        Entity,
        (
            Or<(With<BillboardTextureHandle>, With<BillboardMeshHandle>)>,
            With<Parent>,
        ),
    >,
) {
    for entity in objects.iter() {
        commands.entity(entity).despawn_recursive();
    }
}

pub fn draw_light_gizmo(
    mut gizmos: Gizmos<EditorGizmo>,
    // make the gizmos only show up when the light is selected or toggled?
    lights: Query<(
        &GlobalTransform,
        &LightAreaToggle,
        Option<&Selected>,
        AnyOf<(&DirectionalLight, &SpotLight, &PointLight)>,
    )>,
    // access a global setting for showing all lights areas
    // settings: Res<EditorLightSettings>,
) {
    for (transform, toggled, selected, light_type) in lights.iter() {
        if selected.is_some() || toggled.0 {
            let transform = transform.compute_transform();
            match light_type {
                (Some(directional), _, _) => {
                    // draw an arrow in the direction of the light
                    let dir = transform.forward().normalize();

                    // base
                    gizmos.ray(
                        transform.translation,
                        dir * 3.5,
                        directional.color.with_alpha(1.0),
                    );
                    let dirs = vec![
                        (transform.up().normalize(), transform.down().normalize()),
                        (transform.down().normalize(), transform.up().normalize()),
                        (transform.right().normalize(), transform.left().normalize()),
                        (transform.left().normalize(), transform.right().normalize()),
                    ];
                    for (a, b) in dirs.into_iter() {
                        // vertical
                        gizmos.ray(
                            transform.translation + dir * 3.5,
                            a,
                            directional.color.with_alpha(1.0),
                        );
                        // angle
                        gizmos.ray(
                            transform.translation + dir * 3.5 + a,
                            dir * 1.5 + b,
                            directional.color.with_alpha(1.0),
                        );
                    }
                }
                (_, Some(spot), _) => {
                    // range is the max distance the light will travel in the direction that the light is pointing
                    let range = transform.forward().normalize() * spot.range;

                    // center of the light direction
                    gizmos.ray(transform.translation, range, spot.color.with_alpha(1.0));

                    let outer_rad = range.length() * spot.outer_angle.tan();
                    let inner_rad = range.length() * spot.inner_angle.tan();

                    // circle at the end of the light range at both angles
                    gizmos.circle(
                        transform.translation + range,
                        Dir3::new_unchecked(transform.back().normalize()),
                        outer_rad,
                        spot.color.with_alpha(1.0),
                    );
                    gizmos.circle(
                        transform.translation + range,
                        Dir3::new_unchecked(transform.back().normalize()),
                        inner_rad,
                        spot.color.with_alpha(1.0),
                    );

                    // amount of lines to draw around the "cone" that the light creates
                    let num_segments = 8;
                    for i in 0..num_segments {
                        let angle_outer =
                            i as f32 * 2.0 * std::f32::consts::PI / num_segments as f32;
                        let angle_inner =
                            i as f32 * 2.0 * std::f32::consts::PI / num_segments as f32;

                        let outer_point = transform.translation
                            + range
                            + outer_rad
                                * (transform.right().normalize() * angle_outer.cos()
                                    + transform.up().normalize() * angle_outer.sin());
                        let inner_point = transform.translation
                            + range
                            + inner_rad
                                * (transform.right().normalize() * angle_inner.cos()
                                    + transform.up().normalize() * angle_inner.sin());

                        gizmos.line(
                            transform.translation,
                            outer_point,
                            spot.color.with_alpha(1.0),
                        );
                        gizmos.line(
                            transform.translation,
                            inner_point,
                            spot.color.with_alpha(1.0),
                        );
                    }
                }
                (_, _, Some(point)) => {
                    gizmos.sphere(
                        transform.translation,
                        Quat::IDENTITY,
                        point.range,
                        point.color.with_alpha(1.0),
                    );
                }
                _ => unreachable!(),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn register_assets_as_resource() {
        let mut app = App::new();

        app.add_plugins((
            MinimalPlugins,
            AssetPlugin::default(),
            ImagePlugin::default(),
        ));
        app.init_asset::<bevy::render::mesh::Mesh>();
        app.add_systems(PreUpdate, register_assets);
        app.update();

        let icons = app.world().get_resource::<EditorIconAssets>();

        assert!(icons.is_some());
    }

    #[test]
    fn clears_objects_with_billboard_handles() {
        let mut app = App::new();
        app.add_systems(Startup, |mut commands: Commands| {
            commands.spawn_empty();
            commands.spawn_empty().with_children(|cb| {
                cb.spawn(BillboardTextureBundle::default());
            });
        });
        app.add_systems(Update, clean_meshless);
        app.update();

        let mut query = app.world_mut().query::<Entity>();

        assert_eq!(query.iter(&app.world()).count(), 2);
    }
}
