use bevy::{
    core_pipeline::{
        core_3d::{Camera3dDepthTextureUsage, ScreenSpaceTransmissionQuality},
        tonemapping::{DebandDither, Tonemapping},
    },
    pbr::{CascadeShadowConfig, Cascades, CascadesVisibleEntities, CubemapVisibleEntities},
    prelude::*,
    render::{
        camera::CameraRenderGraph,
        primitives::{CascadesFrusta, CubemapFrusta, Frustum},
        view::{ColorGrading, VisibleEntities},
    },
};
use bevy_scene_hook::HookPlugin;
use space_shared::{LightAreaToggle, PrefabMarker};

use crate::{
    component, editor_registry::EditorRegistryExt, load, prelude::EditorRegistryPlugin, save,
    spawn_system, EditorState, PrefabSet,
};

use component::*;
use load::*;
use save::*;
use spawn_system::*;

/// This plugin contains all components and logic of prefabs
pub struct PrefabPlugin;

impl Plugin for PrefabPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(BasePrefabPlugin);
    }
}

/// This plugin contains all components and logic of prefabs without optional dependencies
pub struct BasePrefabPlugin;

impl Plugin for BasePrefabPlugin {
    fn build(&self, app: &mut App) {
        app.add_state::<EditorState>();

        if !app.is_plugin_added::<HookPlugin>() {
            app.add_plugins(HookPlugin);
        }

        if !app.is_plugin_added::<EditorRegistryPlugin>() {
            app.add_plugins(EditorRegistryPlugin);
        }

        app.configure_sets(
            Update,
            (
                PrefabSet::PrefabLoad,
                PrefabSet::Relation,
                PrefabSet::RelationApply,
                PrefabSet::DetectPrefabChange,
                PrefabSet::PrefabChangeApply,
            )
                .chain(),
        );

        app.add_systems(Update, apply_deferred.in_set(PrefabSet::RelationApply));
        app.add_systems(Update, apply_deferred.in_set(PrefabSet::PrefabChangeApply));

        app.register_type::<EntityLink>();

        app.editor_registry::<Transform>();
        app.editor_registry::<Name>();
        app.editor_registry::<Visibility>();

        app.editor_registry::<GltfPrefab>();
        app.editor_registry::<MaterialPrefab>();
        app.editor_registry::<ColorMaterialPrefab>();

        app.editor_registry::<Sprite>();
        app.editor_registry::<SpriteTexture>();
        app.editor_relation::<SpriteTexture, Transform>();
        app.editor_relation::<SpriteTexture, Visibility>();

        // Spritesheet bundle
        app.editor_registry::<SpritesheetTexture>();
        app.editor_relation::<SpritesheetTexture, Transform>();
        app.editor_relation::<SpritesheetTexture, Visibility>();
        app.editor_registry::<AnimationIndicesSpriteSheet>();
        app.editor_registry::<AnimationClipName>();
        app.editor_registry::<AvailableAnimationClips>();
        app.editor_registry::<AnimationIndicesSpriteSheet>();
        app.editor_registry::<AnimationTimerSpriteSheet>();
        app.editor_registry::<TextureAtlasPrefab>();

        app.editor_registry::<MeshPrimitivePrefab>();
        app.editor_relation::<MeshPrimitivePrefab, Transform>();
        app.editor_relation::<MeshPrimitivePrefab, Visibility>();
        app.editor_relation::<MeshPrimitivePrefab, MaterialPrefab>();

        app.editor_registry::<MeshPrimitive2dPrefab>();
        app.editor_relation::<MeshPrimitive2dPrefab, Transform>();
        app.editor_relation::<MeshPrimitive2dPrefab, Visibility>();
        app.editor_relation::<MeshPrimitive2dPrefab, ColorMaterialPrefab>();

        //shape registration
        app.register_type::<SpherePrefab>();
        app.register_type::<BoxPrefab>();
        app.register_type::<QuadPrefab>();
        app.register_type::<CapsulePrefab>();
        app.register_type::<CirclePrefab>();
        app.register_type::<CylinderPrefab>();
        app.register_type::<IcospherePrefab>();
        app.register_type::<PlanePrefab>();
        app.register_type::<RegularPolygonPrefab>();
        app.register_type::<TorusPrefab>();

        app.editor_registry::<AssetMesh>();
        app.add_systems(
            Update,
            sync_asset_mesh.in_set(PrefabSet::DetectPrefabChange),
        );

        app.editor_registry::<AssetMaterial>();
        app.add_systems(
            Update,
            sync_asset_material.in_set(PrefabSet::DetectPrefabChange),
        );

        //material registration
        app.register_type::<Color>();
        app.register_type::<AlphaMode>();
        app.register_type::<ParallaxMappingMethod>();

        //camera
        app.editor_registry::<Camera>();
        app.editor_registry::<Camera3d>();
        app.editor_registry::<Camera2d>();
        app.editor_registry::<Projection>();
        app.editor_registry::<OrthographicProjection>();
        app.editor_registry::<CameraPlay>();

        app.register_type::<Camera3dDepthTextureUsage>();
        app.register_type::<ScreenSpaceTransmissionQuality>();

        app.editor_relation::<Camera2d, Camera>();
        app.editor_relation::<Camera2d, OrthographicProjection>();
        app.editor_relation::<Camera3d, Camera>();
        app.editor_relation::<Camera3d, Projection>();
        app.editor_relation::<Camera3d, ColorGrading>();
        app.editor_relation::<Camera, VisibleEntities>();
        app.editor_relation::<Camera, Frustum>();
        app.editor_relation::<Camera, Transform>();
        app.editor_relation::<Camera, Tonemapping>();
        app.editor_relation::<Camera, DebandDither>();

        app.add_systems(Update, camera_render_graph_creation);

        app.editor_registry::<PlayerStart>();
        app.editor_relation::<PlayerStart, Transform>();
        app.editor_relation::<PlayerStart, GlobalTransform>();
        app.editor_relation::<PlayerStart, Visibility>();
        app.editor_relation::<PlayerStart, ViewVisibility>();
        app.editor_relation::<PlayerStart, InheritedVisibility>();

        app.editor_relation::<Transform, GlobalTransform>();

        //Light
        app.editor_registry::<LightAreaToggle>();

        app.editor_registry::<PointLight>();
        app.editor_relation::<PointLight, CubemapVisibleEntities>();
        app.editor_relation::<PointLight, CubemapFrusta>();
        app.editor_relation::<PointLight, Transform>();
        app.editor_relation::<PointLight, Visibility>();

        app.editor_registry::<DirectionalLight>();
        app.editor_relation::<DirectionalLight, CascadesFrusta>();
        app.editor_relation::<DirectionalLight, Cascades>();
        app.editor_relation::<DirectionalLight, CascadeShadowConfig>();
        app.editor_relation::<DirectionalLight, CascadesVisibleEntities>();
        app.editor_relation::<DirectionalLight, Transform>();
        app.editor_relation::<DirectionalLight, Visibility>();

        app.editor_registry::<SpotLight>();
        app.editor_relation::<SpotLight, VisibleEntities>();
        app.editor_relation::<SpotLight, Frustum>();
        app.editor_relation::<SpotLight, Transform>();
        app.editor_relation::<SpotLight, Visibility>();

        app.add_systems(OnEnter(EditorState::Game), spawn_player_start);

        app.add_systems(Update, spawn_scene.in_set(PrefabSet::PrefabLoad));
        app.add_systems(
            Update,
            (
                add_global_transform,
                remove_global_transform,
                add_computed_visibility,
                remove_computed_visibility,
            )
                .in_set(PrefabSet::Relation),
        );

        app.add_systems(
            Update,
            (sync_mesh, sync_material).in_set(PrefabSet::DetectPrefabChange),
        );
        app.add_systems(
            Update,
            (
                sync_2d_mesh,
                sync_2d_material,
                sync_sprite_texture,
                sync_spritesheet,
            )
                .in_set(PrefabSet::DetectPrefabChange),
        );

        app.add_systems(
            Update,
            (editor_remove_mesh, editor_remove_mesh_2d).run_if(in_state(EditorState::Editor)),
        );
        app.add_systems(Update, animate_sprite);

        app.add_plugins(SavePrefabPlugin);
        app.add_plugins(LoadPlugin);
        app.add_plugins(crate::sub_scene::SceneUnpackPlugin);
    }
}

fn camera_render_graph_creation(
    mut commands: Commands,
    query: Query<Entity, (With<Camera>, With<PrefabMarker>, Without<CameraRenderGraph>)>,
) {
    for e in query.iter() {
        commands.entity(e).insert(CameraRenderGraph::new(
            bevy::core_pipeline::core_3d::graph::NAME,
        ));
    }
}

/// This systems automatically adds the global transform to entities that don't have it
pub fn add_global_transform(
    mut commands: Commands,
    mut query: Query<
        (Entity, &mut Transform, Option<&Parent>),
        (With<Transform>, Without<GlobalTransform>),
    >,
    globals: Query<&GlobalTransform>,
) {
    for (e, mut tr, parent) in query.iter_mut() {
        if let Some(parent) = parent {
            if let Ok(parent_global) = globals.get(parent.get()) {
                commands.entity(e).insert(parent_global.mul_transform(*tr));
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
    mut commands: Commands,
    query: Query<Entity, (Without<Transform>, With<GlobalTransform>)>,
) {
    for e in query.iter() {
        commands.entity(e).remove::<GlobalTransform>();
    }
}

fn add_computed_visibility(
    mut commands: Commands,
    query: Query<Entity, (With<Visibility>, Without<ViewVisibility>)>,
) {
    for e in query.iter() {
        commands
            .entity(e)
            .insert((InheritedVisibility::VISIBLE, ViewVisibility::default()));
    }
}

fn remove_computed_visibility(
    mut commands: Commands,
    query: Query<Entity, (Without<Visibility>, With<ViewVisibility>)>,
) {
    for e in query.iter() {
        commands
            .entity(e)
            .remove::<ViewVisibility>()
            .remove::<InheritedVisibility>();
    }
}

fn sync_asset_mesh(
    mut commands: Commands,
    changed: Query<(Entity, &AssetMesh), Changed<AssetMesh>>,
    mut deleted: RemovedComponents<AssetMesh>,
    assets: Res<AssetServer>,
) {
    for (e, mesh) in changed.iter() {
        commands.entity(e).insert(assets.load::<Mesh>(&mesh.path));
    }

    for e in deleted.read() {
        if let Some(mut cmd) = commands.get_entity(e) {
            cmd.remove::<Handle<Mesh>>();
            info!("Removed mesh handle for {:?}", e);
        }
    }
}

fn sync_asset_material(
    mut commands: Commands,
    changed: Query<(Entity, &AssetMaterial), Changed<AssetMaterial>>,
    mut deleted: RemovedComponents<AssetMaterial>,
    assets: Res<AssetServer>,
) {
    for (e, material) in changed.iter() {
        commands
            .entity(e)
            .insert(assets.load::<StandardMaterial>(&material.path));
    }

    for e in deleted.read() {
        if let Some(mut cmd) = commands.get_entity(e) {
            cmd.remove::<Handle<StandardMaterial>>();
        }
    }
}
