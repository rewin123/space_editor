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

### Roadmap 🗺️


| Feature                          | Description                                                                                                              | Status    |
|----------------------------------|--------------------------------------------------------------------------------------------------------------------------|:---------:|
| Save/Load                        | Capability to load and save prefabs in the editor by name.                                                               | ✅ Done    |
| Interact with Object             | Functionality to place, rotate, and scale objects using a gizmo.                                                         | ✅ Done    |
| Component Inspector              | Functionality to view and modify component values.                                                                       | ✅ Done    |
| Modify Components                | Ability to add or remove components through the GUI.                                                                     | ✅ Done    |
| GLTF Loader                      | Support for loading GLTF in prefab.                                                                                      | ✅ Done    |
| Prefab Loader                    | Support for loading another prefab within a prefab and hide any technical entities.                                      | ✅ Done    |
| Separate Editor Registration     | Ability to select types which will be shown, saved, and loaded in the editor.                                            | ✅ Done    |
| Customizable UI                  | Feature to customize the inspector UI as per user preference.                                                            | ✅ Done    |
| Asset Inspector                  | Viewer for all project assets to easily drag and drop for adding.                                                        | ❌ Planned |
| Play/Editor States               | Ability to add state to run the game in the editor window, save the prefab at play state start, and reload after end.    | ❌ Planned |
| Player Start Component           | A component to load a prefab only in Play state.                                                                         | ❌ Planned |
| Search and Add Assets            | Support for searching existing assets by typing the name in a field.                                                     | ❌ Planned |
| Edit Nested Prefabs              | If a prefab opened in the editor contains another prefab, allow changes to internal state and apply to all prefabs.      | ❌ Planned |
| Individual Prefab Parameters     | Feature to change some parameters in a unique way, independent of other prefabs.                                         | ❌ Planned |
| Mesh Component                   | Support for using primitives in the prefab editor.                                                                       | ❌ Planned |
| Material Component               | Support for setting up material in prefab.                                                                               | ❌ Planned |
| Bevy_rapier Support              | Support for adding collider/other components from the `bevy_rapier` crate to the editor.                                 | ❌ Planned |
| Bevy_xpcb Support                | Support for adding collider/other components from the `bevy_xpcb` crate to the editor.                                   | ❌ Planned |
| Multiple Select Support          | Feature to manipulate multiple objects simultaneously.                                                                   | ❌ Planned |
| Drink tea after                  |                                                                                                                          | ❌ Planned |

Remember, Rome wasn't built in a day. And neither is `space_editor`. Your feedback and suggestions are always welcome.

