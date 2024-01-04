use bevy::{
    prelude::*,
    render::view::RenderLayers,
    sprite::{Material2d, MaterialMesh2dBundle, Mesh2d, Mesh2dHandle},
};
use space_shared::*;

#[derive(Default)]
pub struct MeshlessVisualizerPlugin;

impl Plugin for MeshlessVisualizerPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<MeshlessMeshMat>()
            .add_systems(Startup, setup_meshless_mesh.in_set(EditorSet::Editor))
            .add_systems(Update, visualize_meshless.in_set(EditorSet::Editor));
    }
}

/// Marks the applied entity as needing to be visualized. This is added via users to make any custom entities have a mesh that they can control
/// also comes with the visual that will be used for the object in question.
// #[derive(Bundle)]
#[derive(Component)]
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

#[derive(Resource, Default)]
pub struct MeshlessMeshMat {
    pub mesh: Handle<Mesh>,
    pub mat: Handle<StandardMaterial>,
}

pub fn visualize_meshless(
    mut commands: Commands,
    objects: Query<
        (
            Entity,
            &Transform,
            Option<&Handle<Mesh>>,
            Option<&Handle<StandardMaterial>>,
        ),
        Or<(
            (With<Camera>, Without<EditorCameraMarker>),
            Or<(With<DirectionalLight>, With<SpotLight>, With<PointLight>)>,
        )>,
    >,
    handle: Res<MeshlessMeshMat>,
) {
    for (entity, transform, mesh, mat) in &objects {
        match (mesh, mat) {
            (Some(_), Some(_)) => {}
            _ => {
                commands.entity(entity).insert((
                    MaterialMeshBundle {
                        mesh: handle.mesh.clone(),
                        material: handle.mat.clone(),
                        transform: *transform,
                        ..default()
                    },
                    RenderLayers::layer((RenderLayers::TOTAL_LAYERS - 1) as u8),
                ));
            }
        }
    }
}

/// This will create a way to have any entity with CustomMeshlessMarker have a way to be visualized by the user
/// Additionally, the user can either choose their own mesh and material to use or default to the white sphere
pub fn visualize_custom_meshless(
    mut commands: Commands,
    ass: AssetServer,
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
    handle.mat = mats.add(StandardMaterial {
        unlit: true,
        ..default()
    });
}
