# space_editor
 bevy prefab editor

> cargo run 


For use prefab spawn system just add plugin to your app
```
App::default()
    .add_plugins(DefaultPlugins)
    .add_plugins(PrefabPlugin)
```

for spawn use PrefabBundle
```
 commands.spawn(PrefabBundle::new("tile.scn.ron"))
        .insert(Name::new("Prefab"));
```


Editor is build for implement to your game by adding plugin to your app

Minimal example for editor:

```
fn main() {
    App::default()
        .add_plugins(DefaultPlugins)
        .add_plugins(SpaceEditorPlugin::default())
        .add_systems(Startup, setup)
        .run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    
    // light
    commands.spawn(PointLightBundle {
        point_light: PointLight {
            intensity: 1500.0,
            shadows_enabled: true,
            ..default()
        },
        transform: Transform::from_xyz(4.0, 8.0, 4.0),
        ..default()
    });
    // camera
    commands.spawn(Camera3dBundle {
        transform: Transform::from_xyz(-2.0, 2.5, 5.0).looking_at(Vec3::ZERO, Vec3::Y),
        ..default()
    }).insert(PanOrbitCamera::default());
}
```

Custom types can be added to editor and prefab editor with single line

```
app.editor_registry::<Name>();
```

Also, representation of component in editor ui can be customized 
```
app.editor_custom_reflect(refl_impl::reflect_name);

...............

pub fn reflect_name(
    ui :  &mut egui::Ui,
    name : &mut Name,
    hash : &str,
    label : &str,
    setup_updated : &mut dyn FnMut(),
    world : &mut UnsafeWorldCell
) {
    ui.horizontal(|ui| {
        name.mutate(|s| {
            ui.label(label);
            ui.add(egui::TextEdit::singleline(s));
        });
    });
}
```

