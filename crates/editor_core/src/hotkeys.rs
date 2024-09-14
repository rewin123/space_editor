#![allow(clippy::type_complexity)]
#![allow(clippy::too_many_arguments)]

use bevy::prelude::*;
use bevy::reflect::GetTypeRegistration;
use bevy::utils::HashMap;

#[cfg(feature = "persistence_editor")]
use space_persistence::AppPersistenceExt;
use space_persistence::PersistenceRegistry;

pub trait Hotkey:
    Send
    + Sync
    + Reflect
    + FromReflect
    + GetTypeRegistration
    + TypePath
    + PartialEq
    + Eq
    + Copy
    + std::hash::Hash
    + 'static
{
    fn name(&self) -> String;
}

#[derive(Resource, Reflect)]
pub struct HotkeySet<T: Hotkey> {
    pub bindings: HashMap<T, Vec<KeyCode>>,
    pub name: String,
}

impl<T> Default for HotkeySet<T>
where
    T: Hotkey,
{
    fn default() -> Self {
        Self {
            bindings: HashMap::new(),
            name: T::short_type_path().to_string(),
        }
    }
}

#[derive(Resource, Default)]
pub struct AllHotkeys {
    pub mappers: Vec<
        Box<
            dyn Fn(&mut World, &mut dyn FnMut(&mut World, String, &mut Vec<KeyCode>)) + Send + Sync,
        >,
    >,
    pub global_mapper: Vec<
        Box<
            dyn Fn(&mut World, &mut dyn FnMut(&mut World, &mut dyn UntypedHotkeySet)) + Send + Sync,
        >,
    >,
}

impl AllHotkeys {
    pub fn map(
        &self,
        world: &mut World,
        map_fun: &mut dyn FnMut(&mut World, String, &mut Vec<KeyCode>),
    ) {
        for mapper in &self.mappers {
            mapper(world, map_fun);
        }
    }

    pub fn global_map(
        &self,
        world: &mut World,
        map_fun: &mut dyn FnMut(&mut World, &mut dyn UntypedHotkeySet),
    ) {
        for mapper in &self.global_mapper {
            mapper(world, map_fun);
        }
    }
}

pub trait UntypedHotkeySet {
    fn get_flat_bindings(&mut self) -> Vec<(String, &mut Vec<KeyCode>)>;
    fn get_name(&self) -> &str;
}

impl<T: Hotkey> UntypedHotkeySet for HotkeySet<T> {
    fn get_flat_bindings(&mut self) -> Vec<(String, &mut Vec<KeyCode>)> {
        let mut res = self
            .bindings
            .iter_mut()
            .map(|(k, v)| (k.name(), v))
            .collect::<Vec<_>>();

        res.sort_by(|a, b| a.0.cmp(&b.0));
        res
    }

    fn get_name(&self) -> &str {
        &self.name
    }
}

pub trait HotkeyAppExt {
    fn editor_hotkey<T: Hotkey>(&mut self, key: T, binding: Vec<KeyCode>) -> &mut Self;
}

impl HotkeyAppExt for App {
    fn editor_hotkey<T: Hotkey>(&mut self, key: T, binding: Vec<KeyCode>) -> &mut Self {
        if !self.world().contains_resource::<AllHotkeys>() {
            self.insert_resource(AllHotkeys::default());
        }

        if !self.world().contains_resource::<HotkeySet<T>>() {
            self.insert_resource(HotkeySet::<T>::default());
            self.init_resource::<ButtonInput<T>>();
            #[cfg(feature = "persistence_editor")]
            {
                if self.world().contains_resource::<PersistenceRegistry>() {
                    self.persistence_resource_with_fn::<HotkeySet<T>>(Box::new(
                        |dst: &mut HotkeySet<T>, src: HotkeySet<T>| {
                            dst.bindings.extend(src.bindings);
                        },
                    ));
                }
            }
            self.add_systems(PreUpdate, hotkey_mapper::<T>);
            self.register_type::<Vec<KeyCode>>();
            self.register_type::<HotkeySet<T>>();
            self.register_type::<HashMap<T, Vec<KeyCode>>>();
            self.register_type::<T>();
            self.world_mut()
                // Safe, was injected in this function
                .resource_mut::<AllHotkeys>()
                .mappers
                .push(Box::new(|w, map_fun| {
                    w.resource_scope::<HotkeySet<T>, _>(|world, mut set| {
                        for (name, binding) in set.get_flat_bindings() {
                            map_fun(world, name, binding);
                        }
                    });
                }));

            self.world_mut()
                // Safe, was injected in this function
                .resource_mut::<AllHotkeys>()
                .global_mapper
                .push(Box::new(|w, map_fun| {
                    w.resource_scope::<HotkeySet<T>, _>(|world, mut set| {
                        map_fun(world, set.as_mut());
                    })
                }))
        }

        let Some(mut set) = self.world_mut().get_resource_mut::<HotkeySet<T>>() else {
            return self;
        };
        set.bindings.insert(key, binding);
        self
    }
}

fn hotkey_mapper<T>(
    bindings: Res<HotkeySet<T>>,
    mut hotkeys: ResMut<ButtonInput<T>>,
    input: Res<ButtonInput<KeyCode>>,
) where
    T: Hotkey,
{
    hotkeys.clear();
    for (key, binding) in bindings.bindings.iter() {
        let mut pressed = true;
        for code in binding {
            if !input.pressed(*code) {
                pressed = false;
            }
        }
        if pressed {
            hotkeys.press(*key);
        } else {
            hotkeys.release(*key);
        }
    }
}

#[cfg(test)]
mod tests {
    use bevy::input::InputPlugin;

    use super::*;

    #[derive(Reflect, Hash, PartialEq, Eq, Clone, Copy, Debug)]
    enum TestKey {
        A,
        B,
    }

    impl Hotkey for TestKey {
        fn name(&self) -> String {
            format!("{:?}", self)
        }
    }

    #[test]
    fn hotkey_tester() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_plugins(InputPlugin);

        app.editor_hotkey(TestKey::A, vec![KeyCode::KeyA]);
        app.editor_hotkey(TestKey::B, vec![KeyCode::KeyB]);
        assert_eq!(TestKey::A.name(), "A");

        app.update();
        {
            let mut input = app.world_mut().resource_mut::<ButtonInput<KeyCode>>();
            input.press(KeyCode::KeyA);
            assert!(input.pressed(KeyCode::KeyA));
        }
        app.update();

        {
            let input = app.world().resource::<ButtonInput<TestKey>>();
            assert_eq!(input.pressed(TestKey::A), true);
        }
    }
}
