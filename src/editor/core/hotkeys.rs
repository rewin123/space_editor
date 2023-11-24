use bevy::prelude::*;
use bevy::reflect::GetTypeRegistration;
use bevy::utils::HashMap;

use super::AppPersistanceExt;


#[derive(Resource, Reflect)]
pub struct HotkeySet<T : PartialEq + Eq + std::hash::Hash + Reflect + FromReflect> {
    pub bindings : HashMap<T, Vec<KeyCode>>,
}

impl<T> Default for HotkeySet<T> {
    fn default() -> Self {
        Self { bindings : HashMap::new() }
    }
}

pub trait HotkeyAppExt {
    fn editor_hotkey<T : PartialEq + Eq + std::hash::Hash + Send + Sync + Reflect + FromReflect + GetTypeRegistration + TypePath + 'static>(&mut self, key : T, binding : Vec<KeyCode>) -> &mut Self;
}

impl HotkeyAppExt for App {
    fn editor_hotkey<T : PartialEq + Eq + std::hash::Hash + Send + Sync + Reflect + FromReflect + GetTypeRegistration + TypePath + 'static>(&mut self, key : T, binding : Vec<KeyCode>) -> &mut Self {
        if !self.world.contains_resource::<HotkeySet<T>>() {
            self.insert_resource(HotkeySet::<T> { bindings : HashMap::new() });
            self.persistance_resource::<HotkeySet<T>>();
        }

        let mut set = self.world.get_resource_mut::<HotkeySet<T>>().unwrap();
        set.bindings.insert(key, binding);
        self
    }
}