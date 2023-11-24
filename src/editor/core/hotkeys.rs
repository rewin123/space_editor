use bevy::prelude::*;
use bevy::reflect::GetTypeRegistration;
use bevy::utils::HashMap;

use super::AppPersistanceExt;

//TODO: I think this must be a derive macro in future
pub trait Hotkey {
    fn name<'a>(&self) -> &'a str;
}

pub trait UntypedHotkeySet {
    
}

#[derive(Resource, Reflect)]
pub struct HotkeySet<T : PartialEq + Eq + std::hash::Hash + Reflect + FromReflect> {
    pub bindings : HashMap<T, Vec<KeyCode>>,
}

impl<T> Default for HotkeySet<T>
where T : PartialEq + Eq + std::hash::Hash + Reflect + FromReflect {

    fn default() -> Self {
        Self { bindings : HashMap::new() }
    }
}

pub trait HotkeyAppExt {
    fn editor_hotkey<T : Copy + PartialEq + Eq + std::hash::Hash + Send + Sync + Reflect + FromReflect + GetTypeRegistration + TypePath + 'static>(&mut self, key : T, binding : Vec<KeyCode>) -> &mut Self;
}

impl HotkeyAppExt for App {
    fn editor_hotkey<T : Copy + PartialEq + Eq + std::hash::Hash + Send + Sync + Reflect + FromReflect + GetTypeRegistration + TypePath + 'static>(&mut self, key : T, binding : Vec<KeyCode>) -> &mut Self {
        if !self.world.contains_resource::<HotkeySet<T>>() {
            self.insert_resource(HotkeySet::<T> { bindings : HashMap::new() });
            self.init_resource::<Input<T>>();
            self.persistance_resource::<HotkeySet<T>>();
            self.add_systems(PreUpdate, hotkey_mapper::<T>);
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
) where T : Copy + PartialEq + Eq + std::hash::Hash + Send + Sync + Reflect + FromReflect + GetTypeRegistration + TypePath + 'static {
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