use bevy::{
    pbr::{LightEntity, Mesh3d},
    prelude::*,
    render::view::RenderLayers,
};
use shared::*;

#[derive(Default)]
pub struct MeshlessVisualizerPlugin;

impl Plugin for MeshlessVisualizerPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<MeshlessMeshMat>()
            .add_systems(Startup, setup_meshless_mesh.in_set(EditorSet::Editor))
            .add_systems(Update, visualize_meshless.in_set(EditorSet::Editor));
    }
}

#[derive(Component)]
pub struct EditorViewOnly;

#[derive(Resource, Default)]
pub struct MeshlessMeshMat {
    pub mesh: Handle<Mesh>,
    pub mat: Handle<StandardMaterial>,
}

pub fn visualize_meshless(
    mut commands: Commands,
    cams: Query<(Entity, &Transform), (With<Camera>, Without<EditorCameraMarker>)>,
    lights: Query<
        (Entity, &Transform),
        Or<(With<DirectionalLight>, With<SpotLight>, With<PointLight>)>,
    >,
    handle: Res<MeshlessMeshMat>,
) {
    for (entity, transform) in &cams {
        commands
            .entity(entity)
            .insert(MaterialMeshBundle {
                mesh: handle.mesh.clone(),
                material: handle.mat.clone(),
                transform: *transform,
                ..default()
            })
            .insert(RenderLayers::layer((RenderLayers::TOTAL_LAYERS - 1) as u8));
    }

    for (entity, transform) in &lights {
        commands
            .entity(entity)
            .insert(MaterialMeshBundle {
                mesh: handle.mesh.clone(),
                material: handle.mat.clone(),
                transform: *transform,
                ..default()
            })
            .insert(RenderLayers::layer((RenderLayers::TOTAL_LAYERS - 1) as u8));
    }
}

/// Creates the default mesh to use
pub fn setup_meshless_mesh(
    mut meshes: ResMut<Assets<Mesh>>,
    mut handle: ResMut<MeshlessMeshMat>,
    mut mats: ResMut<Assets<StandardMaterial>>,
) {
    handle.mesh = meshes.add(
        shape::UVSphere {
            radius: 0.5,
            ..default()
        }
        .into(),
    );
    handle.mat = mats.add(StandardMaterial::default());
}
