use bevy::{
    core_pipeline::{
        core_3d::{Camera3dDepthTextureUsage, ScreenSpaceTransmissionQuality},
        tonemapping::{DebandDither, Tonemapping},
    },
    pbr::{CascadeShadowConfig, Cascades, CascadesVisibleEntities, CubemapVisibleEntities},
    prelude::*,
    render::{
        camera::{CameraMainTextureUsages, CameraRenderGraph, Exposure},
        primitives::{CascadesFrusta, CubemapFrusta, Frustum},
        view::{ColorGrading, VisibleEntities},
    },
};
use bevy_scene_hook::HookPlugin;
use space_shared::toast::ToastMessage;
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
    #[cfg(not(tarpaulin_include))]
    fn build(&self, app: &mut App) {
        app.init_state::<EditorState>();

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

        app.register_type::<Dir3>();
        app.register_type::<Dir2>();

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

        app.editor_registry::<MeshPrimitive3dPrefab>();
        app.editor_relation::<MeshPrimitive3dPrefab, Transform>();
        app.editor_relation::<MeshPrimitive3dPrefab, Visibility>();
        app.editor_relation::<MeshPrimitive3dPrefab, MaterialPrefab>();

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
        app.register_type::<PlanePrefab>();
        app.register_type::<Plane3dPrefab>();
        app.register_type::<PlaneMultiPointPrefab>();
        app.register_type::<RegularPolygonPrefab>();
        app.register_type::<TorusPrefab>();
        app.register_type::<EllipsePrefab>();
        app.register_type::<TrianglePrefab>();
        app.register_type::<Capsule2dPrefab>();

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
        app.editor_registry::<PlaymodeCamera>();

        app.register_type::<Camera3dDepthTextureUsage>();
        app.register_type::<ScreenSpaceTransmissionQuality>();

        app.editor_relation::<Camera2d, Camera>();
        app.editor_relation::<Camera2d, OrthographicProjection>();
        app.editor_relation::<Camera3d, Camera>();
        app.editor_relation::<Camera3d, Projection>();
        app.editor_relation::<Camera3d, ColorGrading>();
        app.editor_relation::<Camera3d, Exposure>();
        app.editor_relation::<Camera, VisibleEntities>();
        app.editor_relation::<Camera, Frustum>();
        app.editor_relation::<Camera, Transform>();
        app.editor_relation::<Camera, Tonemapping>();
        app.editor_relation::<Camera, DebandDither>();
        app.editor_relation::<Camera, CameraMainTextureUsages>();

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

        app.editor_registry::<PlaymodeLight>();

        app.add_event::<ToastMessage>();

        app.add_systems(OnEnter(EditorState::Game), spawn_player_start);

        app.add_systems(Update, spawn_scene.in_set(PrefabSet::PrefabLoad));
        app.add_systems(PreUpdate, create_child_path);

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
    query: Query<
        Entity,
        (
            With<Camera>,
            With<Camera3d>,
            With<PrefabMarker>,
            Without<CameraRenderGraph>,
        ),
    >,
) {
    for e in query.iter() {
        commands.entity(e).insert(CameraRenderGraph::new(
            bevy::core_pipeline::core_3d::graph::Core3d,
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
    query: Query<(Entity, &Visibility), Without<ViewVisibility>>,
) {
    for (e, vis) in query.iter() {
        let mut ent = commands.entity(e);
        if vis != Visibility::Hidden {
            ent.insert((InheritedVisibility::VISIBLE, ViewVisibility::default()));
        } else {
            ent.insert((InheritedVisibility::HIDDEN, ViewVisibility::HIDDEN));
        }
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

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn adds_camera_render_graph_to_prefab_camera() {
        let mut app = App::new();

        app.add_systems(Startup, |mut commands: Commands| {
            commands.spawn((Camera::default(), Camera3d::default(), PrefabMarker));
        })
        .add_systems(Update, camera_render_graph_creation);

        app.update();

        let mut query = app
            .world_mut()
            .query_filtered::<Entity, With<CameraRenderGraph>>();
        assert_eq!(query.iter(&app.world_mut()).count(), 1);
    }

    #[test]
    fn removes_global_transform_from_untransformed_entities() {
        let mut app = App::new();

        app.add_systems(Startup, |mut commands: Commands| {
            commands.spawn(GlobalTransform::default());
        })
        .add_systems(Update, remove_global_transform);

        app.update();

        let mut query = app
            .world_mut()
            .query_filtered::<Entity, (Without<Transform>, With<GlobalTransform>)>();
        assert_eq!(query.iter(&app.world_mut()).count(), 0);
    }

    #[test]
    fn adds_visibility_bundle() {
        let mut app = App::new();

        app.add_systems(Startup, |mut commands: Commands| {
            commands.spawn(Visibility::Visible);
            commands.spawn(Visibility::Hidden);
            commands.spawn((Visibility::Inherited, InheritedVisibility::VISIBLE));
        })
        .add_systems(Update, add_computed_visibility);

        app.update();

        let mut query = app
            .world_mut()
            .query_filtered::<Entity, With<ViewVisibility>>();
        assert_eq!(query.iter(&app.world_mut()).count(), 3);

        let mut query = app.world_mut().query::<&InheritedVisibility>();
        assert_eq!(
            query
                .iter(&app.world_mut())
                .filter(|v| v == &&InheritedVisibility::VISIBLE)
                .count(),
            2
        );
    }

    #[test]
    fn removes_computed_visibility() {
        let mut app = App::new();

        app.add_systems(Startup, |mut commands: Commands| {
            commands.spawn(ViewVisibility::default());
            commands.spawn(VisibilityBundle::default());
            commands.spawn((ViewVisibility::default(), InheritedVisibility::VISIBLE));
        })
        .add_systems(Update, remove_computed_visibility);

        app.update();

        let mut query = app
            .world_mut()
            .query_filtered::<Entity, With<ViewVisibility>>();
        assert_eq!(query.iter(&app.world_mut()).count(), 1);
    }

    #[test]
    fn adds_global_transforms_to_entities() {
        let mut app = App::new();

        app.add_systems(Startup, |mut commands: Commands| {
            commands.spawn(GlobalTransform::default());
            commands.spawn(Transform::default());
            let child = commands.spawn(Transform::default()).id();
            commands.spawn(TransformBundle::default()).add_child(child);
            commands.spawn(TransformBundle::default());
        })
        .add_systems(Update, (add_global_transform, remove_global_transform));

        app.update();

        let mut query = app
            .world_mut()
            .query_filtered::<Entity, (With<Transform>, With<GlobalTransform>)>();
        assert_eq!(query.iter(&app.world_mut()).count(), 4);
    }
}
