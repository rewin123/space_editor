# space_editor: The Bevy Prefab Editor
License: MIT 

![sEditor screenshot](https://github.com/rewin123/space_editor/blob/main/showcase.png)

Welcome to space_editor, a Bevy Prefab Editor built for seamless integration into your game applications. Its design principle is straightforwardness - it's meant to be easy to use and highly customizable.

Getting Started
To run the editor, use the following command:
> cargo run 


## Usage
### Prefab spawn system
To utilize the prefab spawn system, simply add the plugin to your application as follows:
```
App::default()
    .add_plugins(DefaultPlugins)
    .add_plugins(PrefabPlugin)
```

For spawning, use the PrefabBundle:
```
 commands.spawn(PrefabBundle::new("tile.scn.ron"))
        .insert(Name::new("Prefab"));
```

(More code at bin/spawn_prefab.rs)


### Editor integration
The editor is built for easy implementation into your game by adding a plugin to your app. Here's a minimal example of how to do this:

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

(Code from main.rs)

## Customization
Custom types can be added to the editor gui and prefab spawn system with just a single line:

```
app.editor_registry::<Name>();
```
The representation of components in the editor UI can also be customized:
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
(Code from src/editor/inspect/refl_impl.rs)

### Prefab
A prefab is simply a Bevy scene serialized to a readable and editable RON format. However, it needs to be spawned through PrefabBundle to activate custom logic such as adding global transforms to an object.

### Contributing
Pull requests are welcome. For major changes, please open an issue first to discuss what you would like to change.

### License
MIT - https://choosealicense.com/licenses/mit/

### Roadmap
| Feature                          | Description                                                                                                              | Status    |
|----------------------------------|--------------------------------------------------------------------------------------------------------------------------|-----------|
| Save/load                        | Load and save prefabs in editor by name                                                                                  | ✅ Done    |
| Place/rotate/scale               | Allow to interact with object by gizmo                                                                                   | ✅ Done    |
| Component inspector              | Allow to view and change calues of components                                                                            | ✅ Done    |
| Add/Remove components            | Add/remove component in gui                                                                                              | ✅ Done    |
| GLTF loader                      | Load gltf in prefab                                                                                                      | ✅ Done    |
| Prefab loader                    | Load another prefab in prefab, hide any technical entities                                                               | ✅ Done    |
| Separate editor registration     | Select types, which will be shown/saved/loaded in editor                                                                 | ✅ Done    |
| Custom ui in editroe             | Allow to customize inspector ui                                                                                          | ✅ Done    |
| Asset inspector                  | Viewer of all assets in project to easly dran'n'drop add                                                                 | ❌ Planned |
| Play/Editor states               | Add state to run game in editor window, save prefab at play state start an reload after end                              | ❌ Planned |
| Player start component           | Component,  which load prefab only in Play state                                                                         | ❌ Planned |
| Add asset by name and search     | Adding support to search exist assets by taped name in field                                                             | ❌ Planned |
| Allow to edit prefabs in prefabs | If prefab, opened in editor, contains prefab it must be allowed to change internal state and apply to all prefabs        | ❌ Planned |
| Individual prefab parameters     | Allow to change some of parameters in unique way, undependet to another prefabs (for exampe upgrade health to one enemy) | ❌ Planned |
| Mesh component                   | Allow to use primitives in prefab editor                                                                                 | ❌ Planned |
| Material component               | Allow to setup material in prefab                                                                                        | ❌ Planned |
| bevy_rapier support              | Add collider/another components from crate to editor                                                                     | ❌ Planned |
| bevy_xpcb support                | Add collider/another components from crate to editor                                                                     | ❌ Planned |
| Multiple select support          | Allow to manipulate many objects simultenious                                                                            | ❌ Planned |
| Drink tea after                  |                                                                                                                          | ❌ Planned |
