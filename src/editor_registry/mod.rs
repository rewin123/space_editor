use std::{marker::PhantomData, sync::Arc, any::Any};

use bevy::{prelude::*, reflect::{TypeRegistry, GetTypeRegistration}, ecs::{system::{EntityCommand, EntityCommands}, component::ComponentId, world::unsafe_world_cell::UnsafeWorldCell}, utils::HashMap};
use bevy_egui::egui;
use std::any::TypeId;

use crate::PrefabMarker;

pub struct EditorRegistryPlugin;

impl Plugin for EditorRegistryPlugin {
    fn build(&self, app: &mut App) {
        
        app.init_resource::<EditorRegistry>();

        app.editor_registry::<Transform>();
        app.editor_registry::<Name>();
        app.editor_registry::<Visibility>();
        app.editor_registry::<PrefabMarker>();
    }
}

pub struct CloneComponent {
    func : Arc<dyn Fn(&mut EntityCommands, &EntityRef) + Send + Sync>
}

impl CloneComponent {
    pub fn new<T : Clone + Component>() -> Self {
        Self {
            func : Arc::new(move |cmds, src| {
                if let Some(c) = src.get::<T>() {
                    cmds.insert(c.clone());
                }
            })
        }
    }
}

#[derive(Clone)]
pub struct AddDefaultComponent {
    func : Arc<dyn Fn(Entity, &mut World) + Send + Sync>
}

impl EntityCommand for AddDefaultComponent {
    fn apply(self, id: Entity, world: &mut World) {
        (self.func)(id, world);
    }
}

impl AddDefaultComponent {
    pub fn new<T : Default + Component>() -> Self {
        Self { 
            func : Arc::new(move |id, world| {
                world.entity_mut(id).insert(T::default());
            })
        }
    }
}


#[derive(Default, Resource)]
pub struct EditorRegistry {
    pub registry : TypeRegistry,
    pub spawn_components : HashMap<TypeId, AddDefaultComponent>,
    pub custom_reflect : HashMap<TypeId, CustomReflect>,
    pub clone_components : Vec<CloneComponent>
}

impl EditorRegistry {
    pub fn register<T : Component + Default + Send + 'static + GetTypeRegistration + Clone>(&mut self) {
        self.registry.write().register::<T>();
        self.spawn_components.insert(
            T::get_type_registration().type_id(),
            AddDefaultComponent::new::<T>()
        );
        self.clone_components.push(
            CloneComponent::new::<T>()
        );
    }

    pub fn get_spawn_command(&self, id : &TypeId) -> AddDefaultComponent {
        self.spawn_components.get(id).unwrap().clone()
    }

    pub fn clone_entity_flat(&self, cmds : &mut EntityCommands, src : &EntityRef) {
        for t in &self.clone_components {
            (t.func)(cmds, src);
        }
    }
}

pub struct CustomReflect {
    pub reflect : Box<dyn Fn(&mut egui::Ui,
        &mut dyn Reflect,
        &str,
        &str,
        &mut dyn FnMut(),
        &mut UnsafeWorldCell) + 'static + Send + Sync>,
}

pub trait EditorRegistryExt {
    fn editor_registry<T : Component + Default + Send + 'static + GetTypeRegistration + Clone>(&mut self);

    fn editor_custom_reflect<T, F>(&mut self, reflect_fun : F)
        where T : 'static + Reflect + GetTypeRegistration, F : Fn(&mut egui::Ui,
            &mut T,
            &str,
            &str,
            &mut dyn FnMut(),
            &mut UnsafeWorldCell) + 'static + Send + Sync;
}

impl EditorRegistryExt for App {
    fn editor_registry<T : Component + Default + Send + 'static + GetTypeRegistration + Clone>(&mut self) {
        self.world.resource_mut::<EditorRegistry>().register::<T>();
        self.register_type::<T>();
    }

    fn editor_custom_reflect<T, F>(&mut self, reflect_fun : F )
    where T : 'static + Reflect + GetTypeRegistration, F : Fn(&mut egui::Ui,
        &mut T,
        &str,
        &str,
        &mut dyn FnMut(),
        &mut UnsafeWorldCell) + 'static + Send + Sync {
        let box_fun = Box::new(move |
                ui : &mut egui::Ui, 
                r : &mut dyn Reflect, 
                hash : &str,
                name : &str,
                set_changed : &mut dyn FnMut(),
                world : &mut UnsafeWorldCell| {
            unsafe {
                if let Some(t) = r.downcast_mut::<T>() {
                    reflect_fun(ui, t, hash, name, set_changed, world);
                } else {
                    ui.label(format!("Error to custom reflect"));
                }
            }
        });
        
        let custom = CustomReflect {
            reflect : box_fun,
        };

        let reg = T::get_type_registration();
        
        self.world.resource_mut::<EditorRegistry>().custom_reflect.insert(reg.type_id(), custom);
    }
}