use bevy::ecs::change_detection::MutUntyped;
use bevy::prelude::*;
use bevy::reflect::GetTypeRegistration;
use bevy::utils::{HashMap, HashSet};

use super::AppPersistanceExt;

//TODO: I think this must be a derive macro in future
pub trait Hotkey : Send + Sync + Reflect + FromReflect + GetTypeRegistration + TypePath + PartialEq + Eq + Copy + std::hash::Hash + 'static {
    fn name<'a>(&self) -> String;
}

#[derive(Resource, Reflect, Deref)]
pub struct HotkeySet<T : Hotkey> {
    pub bindings : HashMap<T, Vec<KeyCode>>,
}

impl<T> Default for HotkeySet<T>
where T : Hotkey {
    fn default() -> Self {
        Self { bindings : HashMap::new() }
    }
}

#[derive(Resource, Default)]
pub struct AllHotkeys {
    pub mappers : Vec<Box<dyn Fn(&mut World, &mut dyn FnMut(&mut World, String, &mut Vec<KeyCode>)) + Send + Sync>>
}

impl AllHotkeys {
    pub fn map(&self, world : &mut World, map_fun : &mut dyn FnMut(&mut World, String, &mut Vec<KeyCode>)) {
        for mapper in &self.mappers {
            mapper(world, map_fun);
        }
    }
}

pub trait UntypedHotkeySet {
    fn get_flat_bindings(&mut self) -> Vec<(String, &mut Vec<KeyCode>)>;
}

impl<T : Hotkey> UntypedHotkeySet for HotkeySet<T> {
    fn get_flat_bindings(&mut self) -> Vec<(String, &mut Vec<KeyCode>)> {
        self.bindings.iter_mut().map(|(k, v)| (k.name(), v)).collect()
    }
}

pub trait HotkeyAppExt {
    fn editor_hotkey<T : Hotkey>(&mut self, key : T, binding : Vec<KeyCode>) -> &mut Self;
}

impl HotkeyAppExt for App {
    fn editor_hotkey<T : Hotkey>(&mut self, key : T, binding : Vec<KeyCode>) -> &mut Self {

        if !self.world.contains_resource::<AllHotkeys>() {
            self.insert_resource(AllHotkeys::default());
        }

        if !self.world.contains_resource::<HotkeySet<T>>() {
            self.insert_resource(HotkeySet::<T> { bindings : HashMap::new() });
            self.init_resource::<Input<T>>();
            self.persistance_resource::<HotkeySet<T>>();
            self.add_systems(PreUpdate, hotkey_mapper::<T>);
            self.register_type::<Vec<KeyCode>>();
            self.register_type::<HotkeySet<T>>();
            self.register_type::<HashMap<T, Vec<KeyCode>>>();
            self.register_type::<T>();
            self.world.resource_mut::<AllHotkeys>()
                .mappers
                .push(Box::new(|w, map_fun| {
                    w.resource_scope::<HotkeySet<T>, _>(|world, mut set| {
                        for (name, binding) in set.get_flat_bindings() {
                            map_fun(world, name, binding);
                        }
                    });
                }));
        }

        let mut set = self.world.get_resource_mut::<HotkeySet<T>>().unwrap();
        set.bindings.insert(key, binding);
        self
    }
}

fn hotkey_mapper<T>(
    bindings : Res<HotkeySet<T>>,
    mut hotkeys : ResMut<Input<T>>,
    input : Res<Input<KeyCode>>,
) where T : Hotkey {
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