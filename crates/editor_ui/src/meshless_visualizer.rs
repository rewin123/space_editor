use bevy::{prelude::*, render::view::RenderLayers};
use bevy_mod_billboard::{
    prelude::BillboardPlugin, BillboardTextureBundle, BillboardTextureHandle,
};
use bevy_mod_picking::backends::raycast::{
    bevy_mod_raycast::prelude::RaycastVisibility, RaycastBackendSettings,
};
use space_shared::*;

#[derive(Default)]
pub struct MeshlessVisualizerPlugin;

impl Plugin for MeshlessVisualizerPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<LightIcons>()
            .init_resource::<CameraIcon>()
            .init_resource::<IconMesh>()
            .insert_resource(RaycastBackendSettings {
                raycast_visibility: RaycastVisibility::Ignore,
                ..Default::default()
            })
            .add_plugins(BillboardPlugin)
            .add_systems(Startup, load_light_icons.in_set(EditorSet::Editor))
            .add_systems(Update, visualize_meshless.in_set(EditorSet::Editor));
    }
}

/// Marks the applied entity as needing to be visualized. This is added via users to make any custom entities have a mesh that they can control
/// also comes with the visual that will be used for the object in question.
#[derive(Component, Default)]
pub struct CustomMeshless {
    // /// Visual that will be used to show the entity or object
    pub visual: MatMesh,
}

pub struct MatMesh {
    // TODO: figure out what to put with the material so that the compiler doesn't cry
    // material: Box<dyn Material>,
    material: StandardMaterial,
    mesh: Mesh,
}

impl Default for MatMesh {
    fn default() -> Self {
        Self {
            material: StandardMaterial::default(),
            mesh: shape::UVSphere {
                radius: 0.5,
                ..default()
            }
            .into(),
        }
    }
}

// definitely want to use bevy_asset_loader
#[derive(Resource, Default)]
pub struct LightIcons {
    pub directional: Handle<Image>,
    pub point: Handle<Image>,
    pub spot: Handle<Image>,
}

#[derive(Resource, Default)]
pub struct CameraIcon {
    pub camera: Handle<Image>,
}

#[derive(Resource, Default)]
pub struct IconMesh {
    pub mesh: Handle<Mesh>,
    pub sphere: Handle<Mesh>,
}

pub fn visualize_meshless(
    mut commands: Commands,
    lights: Query<(
        Entity,
        &Transform,
        Option<&Children>,
        AnyOf<(&DirectionalLight, &SpotLight, &PointLight)>,
    )>,
    cams: Query<
        (Entity, &Transform, Option<&Children>),
        (
            With<Camera>,
            Without<EditorCameraMarker>,
            With<PrefabMarker>,
        ),
    >,
    light_icons: Res<LightIcons>,
    camera_icon: Res<CameraIcon>,
    icon_mesh: Res<IconMesh>,
) {
    for (parent, _trans, children, light_type) in &lights {
        if children.is_none() {
            let image = match light_type {
                (Some(_directional), _, _) => light_icons.directional.clone(),
                (_, Some(_spot), _) => light_icons.spot.clone(),
                (_, _, Some(_point)) => light_icons.point.clone(),
                _ => unreachable!(),
            };
            // creates a mesh for the icon, as well as a clickable sphere that can be selected to interact with the grandparent, being the actual entity in question
            let child = commands
                .spawn((
                    BillboardTextureBundle {
                        mesh: bevy_mod_billboard::BillboardMeshHandle(icon_mesh.mesh.clone()),
                        texture: BillboardTextureHandle(image.clone()),
                        transform: Transform::default(),
                        ..default()
                    },
                    RenderLayers::layer((RenderLayers::TOTAL_LAYERS - 1) as u8),
                ))
                .with_children(|adult| {
                    adult.spawn((
                        MaterialMeshBundle::<StandardMaterial> {
                            mesh: icon_mesh.sphere.clone(),
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
    for (parent, _trans, children) in &cams {
        if children.is_none() {
            let child = commands
                .spawn((
                    BillboardTextureBundle {
                        mesh: bevy_mod_billboard::BillboardMeshHandle(icon_mesh.mesh.clone()),
                        texture: BillboardTextureHandle(camera_icon.camera.clone()),
                        transform: Transform::default(),
                        ..default()
                    },
                    RenderLayers::layer((RenderLayers::TOTAL_LAYERS - 1) as u8),
                ))
                .with_children(|adult| {
                    adult.spawn((
                        MaterialMeshBundle::<StandardMaterial> {
                            mesh: icon_mesh.sphere.clone(),
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

// TODO: update this to follow the new method so that either a mesh or a 3d sprite can be added to whatever
// a user wants

/// This will create a way to have any entity with CustomMeshlessMarker have a way to be visualized by the user
/// Additionally, the user can either choose their own mesh and material to use or default to the white sphere
pub fn visualize_custom_meshless(
    mut commands: Commands,
    ass: Res<AssetServer>,
    objects: Query<(
        Entity,
        &Transform,
        &CustomMeshless,
        Option<&Handle<Mesh>>,
        Option<&Handle<StandardMaterial>>,
    )>,
) {
    /* NOTES: Maybe this should instead of a struct that is a component, there should be a trait that
            can be impl'd such that the user can pair up different Components and meshes that this function then handles overall.
            An example could be that `objects` is (Entity, &Transform) With<impl CustomMeshless> so then anything that impls it by the user can
            be visualized via their impl, otherwise should be defaulted (or derived default if need be).
    */

    for (entity, transform, custom, mesh, mat) in objects.iter() {
        match (mesh, mat) {
            (Some(_), Some(_)) => {}
            _ => {
                commands.entity(entity).insert((
                    // NOTE: 2d case is not currently covered
                    MaterialMeshBundle {
                        mesh: ass.add(custom.visual.mesh.clone()),
                        material: ass.add(custom.visual.material.clone()),
                        transform: *transform,
                        ..default()
                    },
                    RenderLayers::layer((RenderLayers::TOTAL_LAYERS - 1) as u8),
                ));
            }
        }
    }
}

/// loads the icons for the different types of lights and camera
pub fn load_light_icons(
    ass: Res<AssetServer>,
    mut lights: ResMut<LightIcons>,
    mut cams: ResMut<CameraIcon>,
    mut icon_mesh: ResMut<IconMesh>,
) {
    lights.directional = ass.load("icons/DirectionalLightGizmo.png");
    lights.spot = ass.load("icons/SpotLightGizmo.png");
    lights.point = ass.load("icons/PointLightGizmo.png");
    cams.camera = ass.load("icons/CameraGizmo.png");
    icon_mesh.mesh = ass.add(shape::Quad::new(Vec2::splat(2.)).into());
    icon_mesh.sphere = ass.add(
        shape::Icosphere {
            radius: 0.75,
            ..default()
        }
        .try_into()
        .unwrap(),
    );
}

// this removes the meshes and entities for them when moving to the game state
pub fn clean_meshless() {}
