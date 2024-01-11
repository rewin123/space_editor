use bevy::{prelude::*, render::view::RenderLayers, utils::HashMap};
use bevy_asset_loader::{
    asset_collection::AssetCollection,
    dynamic_asset::{DynamicAsset, DynamicAssetCollection},
    loading_state::{config::ConfigureLoadingState, LoadingState, LoadingStateAppExt},
    prelude::DynamicAssetType,
};
use bevy_common_assets::ron::RonAssetPlugin;
use bevy_mod_billboard::{
    prelude::BillboardPlugin, BillboardMeshHandle, BillboardTextureBundle, BillboardTextureHandle,
};
use bevy_mod_picking::backends::raycast::{
    bevy_mod_raycast::prelude::RaycastVisibility, RaycastBackendSettings,
};
use space_prefab::editor_registry::EditorRegistryExt;
use space_shared::*;

use crate::LAST_RENDER_LAYER;

#[derive(Default)]
pub struct MeshlessVisualizerPlugin;

impl Plugin for MeshlessVisualizerPlugin {
    fn build(&self, app: &mut App) {
        app.add_loading_state(
            LoadingState::new(EditorState::Loading)
                .continue_to_state(EditorState::Editor)
                .load_collection::<EditorIconAssets>()
                .register_dynamic_asset_collection::<EditorIconAssetCollection>()
                .with_dynamic_assets_file::<EditorIconAssetCollection>("icons/editor.icons.ron"),
        )
        .insert_resource(RaycastBackendSettings {
            raycast_visibility: RaycastVisibility::Ignore,
            ..Default::default()
        })
        .add_plugins(BillboardPlugin)
        .add_plugins(RonAssetPlugin::<EditorIconAssetCollection>::new(&[
            "icons.ron",
        ]))
        // .add_systems(Startup, load_light_icons.in_set(EditorSet::Editor))
        // runs every frame within the editor set, when the game transitions to the game state, it stops running
        // then resumes when the editor comes back to the editor state
        .add_systems(OnEnter(EditorState::Editor), show_assets)
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

pub fn show_assets(icons: Res<EditorIconAssets>) {
    // for (k, v) in icons.icons.iter() {
    //     info!("Key: {k:?}, Val: {v:?}");
    // }
}

#[derive(AssetCollection, Resource)]
pub struct EditorIconAssets {
    // make this a custom dynamic asset that is a map of names to use in here "directional", "spot", "point", "camera", etc.
    // best case is that I can do editor_icons.icons.get("directional").unwrap() and ignore all pathing other than just the name
    // #[asset(key = "icons", collection(typed, mapped))]
    // pub icons: HashMap<String, Handle<Image>>,
    // for this check the custom_dynamic_assets.rs example in asset loader
    // Usage: editor_icons.meshes.get("sphere").unwrap()
    // pub meshes: HashMap<String, Handle<Mesh>>,
    #[asset(key = "unknown")]
    pub unknown: Handle<Image>,
    #[asset(key = "directional")]
    pub directional: Handle<Image>,
    #[asset(key = "point")]
    pub point: Handle<Image>,
    #[asset(key = "spot")]
    pub spot: Handle<Image>,
    #[asset(key = "camera")]
    pub camera: Handle<Image>,
    #[asset(key = "square")]
    pub square: Handle<Mesh>,
    #[asset(key = "sphere")]
    pub sphere: Handle<Mesh>,
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
            EditorIconAssetType::Image { path } => vec![asset_server.load::<Image>(path).untyped()],
            _ => vec![],
        }
    }
    fn build(
        &self,
        world: &mut World,
    ) -> Result<bevy_asset_loader::dynamic_asset::DynamicAssetType, anyhow::Error> {
        let cell = world.cell();
        let asset_server = cell
            .get_resource::<AssetServer>()
            .expect("Failed to get the AssetServer");
        match self {
            EditorIconAssetType::Image { path } => {
                let handle = asset_server.load::<Image>(path);
                Ok(DynamicAssetType::Single(handle.untyped()))
            }
            EditorIconAssetType::Quad { size } => {
                let mut meshes = cell
                    .get_resource_mut::<Assets<Mesh>>()
                    .expect("Failed to get Mesh Assets");
                let handle = meshes
                    .add(Mesh::from(shape::Quad {
                        size: *size,
                        ..default()
                    }))
                    .untyped();
                Ok(DynamicAssetType::Single(handle))
            }
            EditorIconAssetType::Sphere { radius } => {
                let mut meshes = cell
                    .get_resource_mut::<Assets<Mesh>>()
                    .expect("Failed to get Mesh Assets");
                let handle = meshes
                    .add(
                        Mesh::try_from(shape::Icosphere {
                            radius: *radius,
                            ..default()
                        })
                        // in case the provided value is bad, defaults to a value that has been tested as good enough
                        .unwrap_or(
                            shape::Icosphere {
                                radius: 0.75,
                                ..default()
                            }
                            .try_into()
                            .unwrap(),
                        ),
                    )
                    .untyped();
                Ok(DynamicAssetType::Single(handle))
            }
        }
    }
}

// fn usage(editor_icons: Res<EditorIconAssets>) {
//     match editor_icons.meshes.get("sphere") {
//         Some(a) => info!("{a:?}"),
//         None => info!("nothing"),
//     }
// }

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
    // light_icons: Res<LightIcons>,
    // camera_icon: Res<CameraIcon>,
    // icon_mesh: Res<IconMesh>,
    editor_icons: Res<EditorIconAssets>,
) {
    for (parent, children, light_type) in &lights {
        // change is none to doesn't contain
        // this then covers the case that lights could have children other than these
        if children.is_none() {
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
                    RenderLayers::layer(LAST_RENDER_LAYER),
                ))
                .with_children(|adult| {
                    adult.spawn((
                        MaterialMeshBundle::<StandardMaterial> {
                            mesh: editor_icons.sphere.clone(),
                            visibility: Visibility::Hidden,
                            ..default()
                        },
                        SelectParent { parent },
                    ));
                })
                .id();
            commands.entity(parent).add_child(child);
        }
    }
    for (parent, children) in &cams {
        if children.is_none() {
            let child = commands
                .spawn((
                    BillboardTextureBundle {
                        mesh: bevy_mod_billboard::BillboardMeshHandle(editor_icons.square.clone()),
                        texture: BillboardTextureHandle(editor_icons.camera.clone()),
                        ..default()
                    },
                    RenderLayers::layer(LAST_RENDER_LAYER),
                ))
                .with_children(|adult| {
                    adult.spawn((
                        MaterialMeshBundle::<StandardMaterial> {
                            mesh: editor_icons.sphere.clone(),
                            visibility: Visibility::Hidden,
                            ..default()
                        },
                        SelectParent { parent },
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
    ass: Res<AssetServer>,
    objects: Query<(Entity, &CustomMeshless, Option<&Children>)>,
    editor_icons: Res<EditorIconAssets>,
) {
    // TODO(MickHarrigan): LONGTERM - Convert from standard material to anything that impl's Material

    for (entity, meshless, children) in objects.iter() {
        if children.is_none() {
            let child = match &meshless.visual {
                MeshlessModel::Billboard {
                    ref mesh,
                    ref texture,
                } => commands
                    .spawn((
                        BillboardTextureBundle {
                            mesh: BillboardMeshHandle(
                                mesh.clone()
                                    .unwrap_or(ass.add(shape::Quad::new(Vec2::splat(2.)).into())),
                            ),
                            texture: BillboardTextureHandle(
                                texture.clone().unwrap_or(ass.load("icons/unknown.png")),
                            ),
                            ..default()
                        },
                        RenderLayers::layer(LAST_RENDER_LAYER),
                    ))
                    .with_children(|adult| {
                        adult.spawn((
                            MaterialMeshBundle::<StandardMaterial> {
                                mesh: editor_icons.sphere.clone(),
                                visibility: Visibility::Hidden,
                                ..default()
                            },
                            SelectParent { parent: entity },
                        ));
                    })
                    .id(),
                MeshlessModel::Object { mesh, material } => commands
                    .spawn((
                        MaterialMeshBundle {
                            mesh: mesh.clone().unwrap_or(editor_icons.sphere.clone()),
                            material: material.clone().unwrap_or(ass.add(StandardMaterial {
                                unlit: true,
                                ..default()
                            })),
                            ..default()
                        },
                        SelectParent { parent: entity },
                        RenderLayers::layer(LAST_RENDER_LAYER),
                    ))
                    .id(),
            };
            commands.entity(entity).add_child(child);
        }
    }
}

/// loads the icons for the different types of lights and camera
// pub fn load_light_icons(
//     ass: Res<AssetServer>,
//     mut lights: ResMut<LightIcons>,
//     mut cams: ResMut<CameraIcon>,
//     mut icon_mesh: ResMut<IconMesh>,
// ) {
//     lights.directional = ass.load("icons/DirectionalLightGizmo.png");
//     lights.spot = ass.load("icons/SpotLightGizmo.png");
//     lights.point = ass.load("icons/PointLightGizmo.png");
//     cams.camera = ass.load("icons/CameraGizmo.png");
//     icon_mesh.square = ass.add(shape::Quad::new(Vec2::splat(2.)).into());
//     icon_mesh.sphere = ass.add(
//         shape::Icosphere {
//             radius: 0.75,
//             ..default()
//         }
//         .try_into()
//         .unwrap(),
//     );
// }

// this removes the meshes and entities for them when moving to the game state
pub fn clean_meshless(
    mut commands: Commands,
    // this covers all entities that are the children of the lights
    // this can be extended to cover the custom children as well
    objects: Query<Entity, Or<(With<BillboardTextureHandle>, With<BillboardMeshHandle>)>>,
) {
    for entity in objects.iter() {
        commands.entity(entity).despawn_recursive();
    }
}
