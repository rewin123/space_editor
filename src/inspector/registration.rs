use std::{marker::PhantomData, sync::Arc, any::Any};

use bevy::{prelude::*, reflect::{TypeRegistry, GetTypeRegistration}, ecs::{system::EntityCommand, component::ComponentId, world::unsafe_world_cell::UnsafeWorldCell}, utils::HashMap};
use bevy_egui::egui;
use std::any::TypeId;


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
    pub custom_reflect : HashMap<TypeId, CustomReflect>
}

impl EditorRegistry {
    pub fn register<T : Component + Default + Send + 'static + GetTypeRegistration>(&mut self) {
        self.registry.write().register::<T>();
        self.spawn_components.insert(
            T::get_type_registration().type_id(),
            AddDefaultComponent::new::<T>()
            
        );
    }

    pub fn get_spawn_command(&self, id : &TypeId) -> AddDefaultComponent {
        self.spawn_components.get(id).unwrap().clone()
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
    fn editor_registry<T : Component + Default + Send + 'static + GetTypeRegistration>(&mut self);

    fn editor_custom_reflect<T, F>(&mut self, reflect_fun : F)
        where T : 'static + Reflect + GetTypeRegistration, F : Fn(&mut egui::Ui,
            &mut T,
            &str,
            &str,
            &mut dyn FnMut(),
            &mut UnsafeWorldCell) + 'static + Send + Sync;
}

impl EditorRegistryExt for App {
    fn editor_registry<T : Component + Default + Send + 'static + GetTypeRegistration>(&mut self) {
        self.world.resource_mut::<EditorRegistry>().register::<T>();
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