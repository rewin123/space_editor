use std::collections::BTreeMap;

use bevy::ecs::system::EntityCommands;

use kcg_prefab::{component::*, ext::*};
use kcg_shared::{LightAreaToggle, PrefabMarker};

/// Resource with bundles to spawn
#[derive(Resource, Default)]
pub struct BundleReg {
    pub bundles: BTreeMap<String, BTreeMap<String, EditorBundleUntyped>>,
}

impl BundleReg {
    pub fn add_bundle<T: Bundle + Clone>(&mut self, bundle: EditorBundle<T>) {
        let dyn_bundle = EditorBundleUntyped::new(bundle.data.clone(), bundle.name.clone());

        self.bundles
            .entry(bundle.category)
            .or_default()
            .insert(bundle.name, dyn_bundle);
    }
}

/// Contains all info to display and spawn editor bundle
pub struct EditorBundle<T: Bundle + Clone> {
    pub data: T,
    pub category: String,
    pub name: String,
}

/// Untyped editor bundle
pub struct EditorBundleUntyped {
    pub data: Box<dyn Fn(&mut EntityCommands) + Send + Sync>,
    pub name: String,
}

impl EditorBundleUntyped {
    /// Create new untyped editor bundle
    pub fn new<T: Bundle + Clone>(data: T, name: String) -> Self {
        Self {
            data: Box::new(move |cmds| {
                cmds.insert(data.clone());
            }),
            name,
        }
    }

    /// Spawn in world untyped editor bundle and mark entity as part of prefab
    pub fn spawn(&self, commands: &mut Commands) -> Entity {
        let mut cmds = commands.spawn_empty();
        (self.data)(&mut cmds);
        cmds.insert(PrefabMarker);
        cmds.id()
    }
}

/// Trait to add `editor_bundle(..)` to App
pub trait EditorUiExt {
    /// Register new bundle in editor ui
    fn editor_bundle<T: Bundle + Clone>(&mut self, category: &str, name: &str, bundle: T);
}

impl EditorUiExt for App {
    fn editor_bundle<T: Bundle + Clone>(&mut self, category: &str, name: &str, bundle: T) {
        let mut reg = if let Some(reg) = self.world.get_resource_mut::<BundleReg>() {
            reg
        } else {
            self.init_resource::<BundleReg>();
            self.world.get_resource_mut::<BundleReg>().unwrap()
        };

        reg.add_bundle(EditorBundle {
            data: bundle,
            category: category.to_string(),
            name: name.to_string(),
        });
    }
}

pub fn register_light_editor_bundles(app: &mut App) {
    app.editor_bundle(
        "🔆 Light",
        "Point light",
        (
            Name::new("Point light"),
            PointLight::default(),
            LightAreaToggle::default(),
        ),
    );

    app.editor_bundle(
        "🔆 Light",
        "Directional light",
        (
            Name::new("Directional light"),
            DirectionalLight::default(),
            LightAreaToggle::default(),
        ),
    );

    app.editor_bundle(
        "🔆 Light",
        "Spot light",
        (
            Name::new("Spot light"),
            SpotLight::default(),
            LightAreaToggle::default(),
        ),
    );
}

/// Register meshs
pub fn register_mesh_editor_bundles(app: &mut App) {
    app.editor_bundle(
        "Mesh",
        "3D Cube",
        (
            MeshPrimitivePrefab::Cube(1.0),
            Name::new("Cube".to_string()),
            Transform::default(),
            Visibility::default(),
        ),
    );
    app.editor_bundle(
        "Mesh",
        "3D Box",
        (
            MeshPrimitivePrefab::Box(BoxPrefab::default()),
            Name::new("Box".to_string()),
            Transform::default(),
            Visibility::default(),
        ),
    );
    app.editor_bundle(
        "Mesh",
        "3D Sphere",
        (
            MeshPrimitivePrefab::Sphere(SpherePrefab::default()),
            Name::new("UVSphere".to_string()),
            Transform::default(),
            Visibility::default(),
        ),
    );
    app.editor_bundle(
        "Mesh",
        "3D Quad",
        (
            MeshPrimitivePrefab::Quad(QuadPrefab::default()),
            Name::new("Quad".to_string()),
            Transform::default(),
            Visibility::default(),
        ),
    );
    app.editor_bundle(
        "Mesh",
        "3D Capsule",
        (
            MeshPrimitivePrefab::Capsule(CapsulePrefab::default()),
            Name::new("Capsule"),
            Transform::default(),
            Visibility::default(),
        ),
    );
    app.editor_bundle(
        "Mesh",
        "3D Circle",
        (
            MeshPrimitivePrefab::Circle(CirclePrefab::default()),
            Name::new("Circle".to_string()),
            Transform::default(),
            Visibility::default(),
        ),
    );
    app.editor_bundle(
        "Mesh",
        "3D Cylinder",
        (
            MeshPrimitivePrefab::Cylinder(CylinderPrefab::default()),
            Name::new("Cylinder".to_string()),
            Transform::default(),
            Visibility::default(),
        ),
    );
    app.editor_bundle(
        "Mesh",
        "3D Icosphere",
        (
            MeshPrimitivePrefab::Icosphere(IcospherePrefab::default()),
            Name::new("Icosphere".to_string()),
            Transform::default(),
            Visibility::default(),
        ),
    );
    app.editor_bundle(
        "Mesh",
        "3D Plane",
        (
            MeshPrimitivePrefab::Plane(PlanePrefab::default()),
            Name::new("Plane".to_string()),
            Transform::default(),
            Visibility::default(),
        ),
    );
    app.editor_bundle(
        "Mesh",
        "3D Regular Polygon",
        (
            MeshPrimitivePrefab::RegularPolygon(RegularPolygonPrefab::default()),
            Name::new("Regular Polygon".to_string()),
            Transform::default(),
            Visibility::default(),
        ),
    );
    app.editor_bundle(
        "Mesh",
        "3D Torus",
        (
            MeshPrimitivePrefab::Torus(TorusPrefab::default()),
            Name::new("Torus".to_string()),
            Transform::default(),
            Visibility::default(),
        ),
    );

    app.editor_bundle(
        "Mesh",
        "2D Quad",
        (
            MeshPrimitive2dPrefab::Quad(QuadPrefab::default()),
            Name::new("2D Quad".to_string()),
            Transform::default(),
            Visibility::default(),
        ),
    );

    app.editor_bundle(
        "Mesh",
        "2D Circle",
        (
            MeshPrimitive2dPrefab::Circle(CirclePrefab::default()),
            Name::new("2D Circle".to_string()),
            Transform::default(),
            Visibility::default(),
        ),
    );

    app.editor_bundle(
        "Mesh",
        "2D Plane",
        (
            MeshPrimitive2dPrefab::Plane(PlanePrefab::default()),
            Name::new("2D Plane".to_string()),
            Transform::default(),
            Visibility::default(),
        ),
    );

    app.editor_bundle(
        "Mesh",
        "2D Regular Polygon",
        (
            MeshPrimitive2dPrefab::RegularPolygon(RegularPolygonPrefab::default()),
            Name::new("2D Regular Polygon".to_string()),
            Transform::default(),
            Visibility::default(),
        ),
    );

    app.editor_bundle(
        "Camera",
        "3D Playmode Camera",
        (
            Camera3d::default(),
            Name::new("Camera3d".to_string()),
            Transform::default(),
            Visibility::default(),
            CameraPlay::default(),
        ),
    );

    app.editor_bundle(
        "Camera",
        "2D Playmode Camera",
        (
            Camera2d::default(),
            Name::new("Camera2d".to_string()),
            Transform::default(),
            Visibility::default(),
            CameraPlay::default(),
        ),
    );

    app.editor_bundle(
        "Sprite",
        "Blank Sprite",
        (
            SpriteBundle {
                sprite: Sprite {
                    color: Color::BLUE,
                    custom_size: Some(Vec2::new(100.0, 100.0)),
                    ..default()
                },
                ..default()
            },
            Name::new("Sprite".to_string()),
        ),
    );

    app.editor_bundle(
        "Sprite",
        "Texture Sprite",
        (
            SpriteTexture {
                texture: "branding/bevy_bird_dark.png".to_string(),
            },
            Name::new("Texture Sprite".to_string()),
        ),
    );

    app.editor_bundle(
        "Sprite",
        "Sprite Sheet",
        (
            SpritesheetTexture {
                texture: String::from("textures/gabe-idle-run.png"),
            },
            Name::from("Spritesheet"),
            AnimationIndicesSpriteSheet::default(),
            AnimationClipName::default(),
            AvailableAnimationClips::default(),
            AnimationTimerSpriteSheet::default(),
            TextureAtlasPrefab::default(),
        ),
    )
}
