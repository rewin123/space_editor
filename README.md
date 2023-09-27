# space_editor: The Bevy Prefab Editor
License: MIT 

![Editor screenshot](https://github.com/rewin123/space_editor/blob/main/showcase.png)

space_editor is usefull tool for scene/prefab/prototyping for bevy.  Its allow to create/modify levels/scenes/prefabs in fast gui based way.
## Main features

- **Intuitive Scene and Prefab Management**: Space Editor allows you to prepare and save scenes and prefabs with an intuitive user interface. You can easily reuse them in your game development workflow. 
- **bevy_xpbd_3d compatibility**: Space Editor supports bevy_xpbd_3d, including all editor features. 
- **Gizmo-Based manipulations**: Manipulate entity positions, rotations, and scales using gizmos. 
- **Component values editing**: Easily edit component parameters within the editor UI 
- **Seamless Editor-Game switching**: Switch between the editor UI and the game effortlessly for fast prototyping and testing. 
- **Prefab Reusability**: Prefabs can be nested within other prefabs, improving reusability and organization in your projects. 
- **Many custom components**: Space Editor implements various custom components to seamlessly integrate its saving system with the standard Bevy scene format. 
- **Easy API for customization**: Customize or register your own components within the editor with ease, tailoring it to your specific project needs.
- **API for adding tabs**: Extend the functionality of the editor by easily adding new tabs, enhancing your workflow. 

Getting Started
To run the editor, use the following command:
> cargo run

To run platformer example, use the following command:
> cargo run run --example platformer --features bevy_xpbd_3d


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
        .add_systems(Startup, simple_editor_setup)
        .run();
}
```

(Code from main.rs)

## Customization

**More detailed information in docs/README.md**

Custom types can be added to the editor gui and prefab spawn system with just a single line:

```
app.editor_registry::<Name>();
```
The representation of components in the editor UI can also be customized by bevy_inspector_egui library.

### Prefab
A prefab is simply a Bevy scene serialized to a readable and editable RON format. However, it needs to be spawned through PrefabBundle to activate custom logic such as adding global transforms to an object.

### Contributing
Any request for adding functionality to the editor is welcome. Open an issue on the [issue tracker](https://github.com/rewin123/space_editor/issues).
Any pull request is welcome too:) 

### License
MIT - https://choosealicense.com/licenses/mit/

### Roadmap 🗺️


| Feature                          | Description                                                                                                              | Status    |
|----------------------------------|--------------------------------------------------------------------------------------------------------------------------|:---------:|
| Save/Load                        | Capability to load and save prefabs in the editor by name.                                                               | ✅ Done             |
| Interact with Object             | Functionality to place, rotate, and scale objects using a gizmo.                                                         | ✅ Done             |
| Component Inspector              | Functionality to view and modify component values.                                                                       | ✅ Done             |
| Modify Components                | Ability to add or remove components through the GUI.                                                                     | ✅ Done             |
| GLTF Loader                      | Support for loading GLTF in prefab.                                                                                      | ✅ Done             |
| Prefab Loader                    | Support for loading another prefab within a prefab and hide any technical entities.                                      | ✅ Done             |
| Separate Editor Registration     | Ability to select types which will be shown, saved, and loaded in the editor.                                            | ✅ Done             |
| Customizable UI                  | Feature to customize the inspector UI as per user preference.                                                            | ✅ Done             |
| Asset Inspector                  | Viewer for all project assets to easily drag and drop for adding.                                                        | ❌ Planned          |
| Play/Editor States               | Ability to add state to run the game in the editor window, save the prefab at play state start, and reload after end.    | ✅ Done             |
| Player Start Component           | A component to load a prefab only in Play state.                                                                         | ✅ Done          |
| Search and Add Assets            | Support for searching existing assets by typing the name in a field.                                                     | ❌ Planned          |
| Edit Nested Prefabs              | If a prefab opened in the editor contains another prefab, allow changes to internal state and apply to all prefabs.      | ❌ Planned          |
| Individual Prefab Parameters     | Feature to change some parameters in a unique way, independent of other prefabs.                                         | ❌ Planned          |
| Mesh Component                   | Support for using primitives in the prefab editor.                                                                       | ✅ Done |
| Material Component               | Support for setting up material in prefab.                                                                               | ✅ Done |
| Bevy_rapier Support              | Support for adding collider/other components from the `bevy_rapier` crate to the editor.                                 | ❌ Planned          |
| Bevy_xpcb Support                | Support for adding collider/other components from the `bevy_xpcb` crate to the editor.                                   | ✅ Mostly done      |
| bevy_proto Support                | Support for commonly used text-based prefab system                                                                       | ❌ Planned          |
| bevy_mod_picking Support         | Support for mouse select and deselect of entities                                                                        | ✅ Done             |
| bevy_inspector_egui Support      | Support for commonly used inspector library                                                                              | ✅ Done             |
| Multiple Select Support          | Feature to manipulate multiple objects simultaneously.                                                                   | 🛠️ Work in progress |
| Tests                            | Useful tests for make crate stable                                                                                       | ❌ Planned          |
| Drink tea after                  |                                                                                                                          | ❌ Planned          |

Remember, Rome wasn't built in a day. And neither is `space_editor`. Your feedback and suggestions are always welcome.

