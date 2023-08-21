use std::{marker::PhantomData, sync::Arc, any::Any};

use bevy::{prelude::*, reflect::{TypeRegistry, GetTypeRegistration}, ecs::{system::{EntityCommand, EntityCommands}, component::ComponentId, world::unsafe_world_cell::UnsafeWorldCell}, utils::{HashMap, HashSet}};
use bevy_egui::egui;
use bevy_inspector_egui::{reflect_inspector::InspectorUi, inspector_egui_impls::InspectorEguiImpl};
use std::any::TypeId;

use crate::PrefabMarker;

pub struct EditorRegistryPlugin;



impl Plugin for EditorRegistryPlugin {
    fn build(&self, app: &mut App) {
        
        app.init_resource::<EditorRegistry>();

        app.editor_registry::<Transform>();
        app.editor_registry::<Name>();
        app.editor_registry::<Visibility>();
        
        app.editor_clone_registry::<PrefabMarker>();
    }
}

#[derive(Clone)]
pub struct RemoveComponent {
    func : Arc<dyn Fn(&mut EntityCommands) + Send + Sync>
}

impl RemoveComponent {
    pub fn new<T : Clone + Component>() -> Self {
        Self {
            func : Arc::new(move |cmds| {
                cmds.remove::<T>();
            })
        }
    }
}

#[derive(Clone)]
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


#[derive(Default, Resource, Clone)]
pub struct EditorRegistry {
    pub registry : TypeRegistry,
    pub spawn_components : HashMap<TypeId, AddDefaultComponent>,
    pub clone_components : Vec<CloneComponent>,
    pub remove_components : HashMap<TypeId, RemoveComponent>,
    pub silent : HashSet<TypeId> //skip in inspector ui
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
        self.remove_components.insert(
            T::get_type_registration().type_id(),
            RemoveComponent::new::<T>()
        );
    }

    pub fn silent_register<T : Component + Default + Send + 'static + GetTypeRegistration + Clone>(&mut self) {
        self.registry.write().register::<T>();
        self.spawn_components.insert(
            T::get_type_registration().type_id(),
            AddDefaultComponent::new::<T>()
        );
        self.clone_components.push(
            CloneComponent::new::<T>()
        );
        self.silent.insert(T::get_type_registration().type_id());
        self.remove_components.insert(
            T::get_type_registration().type_id(),
            RemoveComponent::new::<T>()
        );
    }

    pub fn only_clone_register<T : Component + Default + Send + 'static + GetTypeRegistration + Clone>(&mut self) {
        self.clone_components.push(
            CloneComponent::new::<T>()
        );
    }

    pub fn get_spawn_command(&self, id : &TypeId) -> AddDefaultComponent {
        self.spawn_components.get(id).unwrap().clone()
    }

    pub fn remove_by_id(&self, cmds : &mut EntityCommands, id : &TypeId) {
        if let Some(rem) = self.remove_components.get(id) {
            (rem.func)(cmds);
        }
    }

    pub fn clone_entity_flat(&self, cmds : &mut EntityCommands, src : &EntityRef) {
        for t in &self.clone_components {
            (t.func)(cmds, src);
        }
    }
}

pub trait EditorRegistryExt {
    fn editor_registry<T : Component + Default + Send + 'static + GetTypeRegistration + Clone>(&mut self);
    fn editor_silent_registry<T : Component + Default + Send + 'static + GetTypeRegistration + Clone>(&mut self);
    fn editor_clone_registry<T : Component + Default + Send + 'static + GetTypeRegistration + Clone>(&mut self);

    fn editor_relation<T, Relation>(&mut self)
        where T : Component, Relation : Component + Default;
}

impl EditorRegistryExt for App {
    fn editor_registry<T : Component + Default + Send + 'static + GetTypeRegistration + Clone>(&mut self) {
        self.world.resource_mut::<EditorRegistry>().register::<T>();
        self.register_type::<T>();
    }

    fn editor_clone_registry<T : Component + Default + Send + 'static + GetTypeRegistration + Clone>(&mut self) {
        self.world.resource_mut::<EditorRegistry>().only_clone_register::<T>();
        self.register_type::<T>();
    }

    fn editor_silent_registry<T : Component + Default + Send + 'static + GetTypeRegistration + Clone>(&mut self) {
        self.world.resource_mut::<EditorRegistry>().silent_register::<T>();
        self.register_type::<T>();
    }

    fn editor_relation<T, Relation>(&mut self) 
        where T : Component, Relation : Component + Default {
        
        self.add_systems(Update, relation_system::<T, Relation>);
    }
}


fn relation_system<T : Component, Relation : Component + Default>(
    mut commands : Commands,
    query : Query<Entity, (Added<T>, Without<Relation>)>
) {
    for e in query.iter() {
        commands.entity(e).insert(Relation::default());
    }
}