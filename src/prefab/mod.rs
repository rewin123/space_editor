pub mod component;
pub mod spawn_system;
pub mod save;
pub mod load;

use bevy::{prelude::*, core_pipeline::{core_3d::Camera3dDepthTextureUsage, tonemapping::{Tonemapping, DebandDither}}, render::{view::{VisibleEntities, ColorGrading}, primitives::Frustum, camera::CameraRenderGraph}};
use bevy_scene_hook::HookPlugin;

use crate::{editor_registry::EditorRegistryExt, prelude::EditorRegistryPlugin, EditorState, EditorSet, PrefabMarker, EditorCameraMarker};

use component::*;
use spawn_system::*;
use save::*;
use load::*;

pub struct PrefabPlugin;

impl Plugin for PrefabPlugin {
    fn build(&self, app: &mut App) {

        app.add_state::<EditorState>();

        if !app.is_plugin_added::<HookPlugin>() {
            app.add_plugins(HookPlugin);
        }

        if !app.is_plugin_added::<EditorRegistryPlugin>() {
            app.add_plugins(EditorRegistryPlugin);
        }

        app.configure_set(Update, EditorSet::Game.run_if(in_state(EditorState::Game)));
        app.configure_set(Update, EditorSet::Editor.run_if(in_state(EditorState::Editor)));
        
        app.editor_registry::<GltfPrefab>();
        app.editor_registry::<MaterialPrefab>();

        app.editor_registry::<MeshPrimitivePrefab>();
        app.editor_relation::<MeshPrimitivePrefab, Transform>();
        app.editor_relation::<MeshPrimitivePrefab, Visibility>();
        app.editor_relation::<MeshPrimitivePrefab, MaterialPrefab>();

        //shape registration
        app.register_type::<SpherePrefab>();
        app.register_type::<BoxPrefab>();
        app.register_type::<QuadPrefab>();
        app.register_type::<CapsulePrefab>();
        app.register_type::<CirclePrefab>();
        app.register_type::<CylinderPrefab>();
        app.register_type::<IcospherePrefab>();
        app.register_type::<PlanePrefab>();
        app.register_type::<RegularPoligonPrefab>();
        app.register_type::<TorusPrefab>();

        //material registration
        app.register_type::<Color>();
        app.register_type::<AlphaMode>();
        app.register_type::<ParallaxMappingMethod>();

        //camera
        app.editor_registry::<Camera>();
        app.editor_registry::<Camera3d>();
        app.editor_registry::<Projection>();
        app.editor_registry::<CameraPlay>();

        app.register_type::<Camera3dDepthTextureUsage>();


        app.editor_relation::<Camera3d, Camera>();
        app.editor_relation::<Camera, Projection>();
        app.editor_relation::<Camera, VisibleEntities>();
        app.editor_relation::<Camera, Frustum>();
        app.editor_relation::<Camera, Transform>();
        app.editor_relation::<Camera, Tonemapping>();
        app.editor_relation::<Camera, DebandDither>();
        app.editor_relation::<Camera, ColorGrading>();
        app.add_systems(Update, camera_render_graph_creation);
        
        app.editor_registry::<PlayerStart>();
        app.editor_relation::<PlayerStart, Transform>();
        app.editor_relation::<PlayerStart, GlobalTransform>();
        app.editor_relation::<PlayerStart, Visibility>();
        app.editor_relation::<PlayerStart, ComputedVisibility>();

        app.add_systems(OnEnter(EditorState::Game), spawn_player_start);

        app.add_systems(Update, spawn_scene);
        app.add_systems(Update, (add_global_transform, remove_global_transform, add_computed_visiblity, remove_computed_visiblity));

        app.add_systems(
            Update,
            (sync_mesh, sync_material)
        );

        app.add_systems(
            Update,
            (editor_remove_mesh).run_if(in_state(EditorState::Editor))
        );

        app.add_plugins(SavePrefabPlugin);
        app.add_plugins(LoadPlugin);

    }
}


fn draw_camera_gizmo(
    mut gizom : Gizmos,
    cameras : Query<(&GlobalTransform, &Projection), (With<Camera>, Without<EditorCameraMarker>)>
) {
    for (transform, projection) in cameras.iter() {
        let transform = transform.compute_transform();
        let cuboid_transform = transform.with_scale(Vec3::new(2.0, 1.0, 1.0));
        gizom.cuboid(cuboid_transform, Color::PINK);
    }
}

fn camera_render_graph_creation(
    mut commands : Commands,
    query : Query<Entity, (With<Camera>, With<PrefabMarker>, Without<CameraRenderGraph>)>
) {
    for e in query.iter() {
        commands.entity(e).insert( CameraRenderGraph::new(bevy::core_pipeline::core_3d::graph::NAME));
    }
}

pub fn add_global_transform(
    mut commands : Commands,
    mut query : Query<(Entity, &mut Transform, Option<&Parent>), (With<Transform>, Without<GlobalTransform>)>,
    globals : Query<&GlobalTransform>
) {
    for (e, mut tr, parent) in query.iter_mut() {
        if let Some(parent) = parent {
            if let Ok(parent_global) = globals.get(parent.get()) {
                commands.entity(e).insert(parent_global.mul_transform(tr.clone()));
            } else {
                commands.entity(e).insert(GlobalTransform::from(*tr));
            }
        } else {
            commands.entity(e).insert(GlobalTransform::from(*tr));

        }
        tr.set_changed();
    }
}

fn remove_global_transform(
    mut commands : Commands,
    query : Query<Entity, (Without<Transform>, With<GlobalTransform>)>
) {
    for e in query.iter() {
        commands.entity(e).remove::<GlobalTransform>();
    }
}

fn add_computed_visiblity(
    mut commands : Commands,
    query : Query<Entity, (With<Visibility>, Without<ComputedVisibility>)>
) {
    for e in query.iter() {
        commands.entity(e).insert(ComputedVisibility::default());
    }
}

fn remove_computed_visiblity(
    mut commands : Commands,
    query : Query<Entity, (Without<Visibility>, With<ComputedVisibility>)>
) {
    for e in query.iter() {
        commands.entity(e).remove::<ComputedVisibility>();
    }
}