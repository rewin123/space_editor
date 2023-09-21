use std::sync::Arc;

use bevy::{prelude::*, reflect::{TypeRegistry, GetTypeRegistration, TypePath}, ecs::system::{EntityCommand, EntityCommands}, utils::{HashMap, HashSet}};

use std::any::TypeId;

use crate::{PrefabMarker, prefab::{component::AutoStruct, save::SaveState}, PrefabSet};

/// Plugin to activate custom registry 
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

/// Contains function to remove component in untyped style
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

/// Contains function to clone component in untyped style
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

/// Contains function to add default component in untyped style
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


/// Resource, which contains all custom editor registry
#[derive(Default, Resource, Clone)]
pub struct EditorRegistry {
    pub registry : TypeRegistry,
    pub spawn_components : HashMap<TypeId, AddDefaultComponent>,
    pub clone_components : Vec<CloneComponent>,
    pub remove_components : HashMap<TypeId, RemoveComponent>,
    pub silent : HashSet<TypeId> //skip in inspector ui
}

impl EditorRegistry {
    /// Register new component, which will be shown in editor UI and saved in prefab
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

    /// Register new component, which will be hidden in editor UI and saved in prefab
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

    /// Register new component, which will be cloned with editor ui clone event
    pub fn only_clone_register<T : Component + Default + Send + 'static + GetTypeRegistration + Clone>(&mut self) {
        self.clone_components.push(
            CloneComponent::new::<T>()
        );
    }

    /// Get spawn function for this component type
    pub fn get_spawn_command(&self, id : &TypeId) -> AddDefaultComponent {
        self.spawn_components.get(id).unwrap().clone()
    }

    /// Get remove function for this component type
    pub fn remove_by_id(&self, cmds : &mut EntityCommands, id : &TypeId) {
        if let Some(rem) = self.remove_components.get(id) {
            (rem.func)(cmds);
        }
    }

    /// Get clone function for this component type
    pub fn clone_entity_flat(&self, cmds : &mut EntityCommands, src : &EntityRef) {
        for t in &self.clone_components {
            (t.func)(cmds, src);
        }
    }
}

pub trait EditorRegistryExt {
    /// refister new component in editor UI and prefab systems
    fn editor_registry<T : Component + Default + Send + 'static + GetTypeRegistration + Clone>(&mut self) -> &mut Self;
    /// register new component inly in prefab systems (will be no shown in editor UI)
    fn editor_silent_registry<T : Component + Default + Send + 'static + GetTypeRegistration + Clone>(&mut self) -> &mut Self;
    
    fn editor_clone_registry<T : Component + Default + Send + 'static + GetTypeRegistration + Clone>(&mut self) -> &mut Self;

    /// Mark that if T component spawned, then Relation must be spawned too
    fn editor_relation<T, Relation>(&mut self) -> &mut Self
        where T : Component, Relation : Component + Default;
    
    /// Not used yet
    fn editor_auto_struct<T>(&mut self) -> &mut Self
        where  T : Component + Reflect + FromReflect + Default + Clone + 'static + GetTypeRegistration + TypePath;
}

impl EditorRegistryExt for App {
    fn editor_registry<T : Component + Default + Send + 'static + GetTypeRegistration + Clone>(&mut self) -> &mut Self {
        self.world.resource_mut::<EditorRegistry>().register::<T>();
        self.world.init_component::<T>();
        self.register_type::<T>();
        self
    }

    fn editor_clone_registry<T : Component + Default + Send + 'static + GetTypeRegistration + Clone>(&mut self) -> &mut Self {
        self.world.resource_mut::<EditorRegistry>().only_clone_register::<T>();
        self.register_type::<T>();
        self
    }

    fn editor_silent_registry<T : Component + Default + Send + 'static + GetTypeRegistration + Clone>(&mut self) -> &mut Self {
        self.world.resource_mut::<EditorRegistry>().silent_register::<T>();
        self.register_type::<T>();
        self
    }

    fn editor_relation<T, Relation>(&mut self) -> &mut Self
        where T : Component, Relation : Component + Default {
        
        self.add_systems(Update, relation_system::<T, Relation>.in_set(PrefabSet::Relation));
        self
    }

    
    fn editor_auto_struct<T>(&mut self) -> &mut Self
        where T : Component + Reflect + FromReflect + Default + Clone + 'static + GetTypeRegistration + TypePath {
        self.editor_silent_registry::<AutoStruct<T>>();
        self.editor_registry::<T>();

        self.add_systems(OnEnter(SaveState::Save), generate_auto_structs::<T>);
        self.add_systems(Update, despawn_auto_structs::<T>);
        self
    }
}


/// Not used
fn generate_auto_structs<T : Component + Reflect + FromReflect + Default + Clone>(
    mut commands : Commands,
    query : Query<(Entity, &T)>,
    assets : Res<AssetServer>
) {
    for (e, data) in query.iter() {
        commands.entity(e).insert(AutoStruct::new(data, &assets));
    }
}

/// Not used
fn despawn_auto_structs<T : Component + Reflect + FromReflect + Default + Clone>(
    mut commands : Commands,
    query : Query<(Entity, &AutoStruct<T>)>,
    assets : Res<AssetServer>
) {
    for (e, auto_data) in query.iter() {
        let data = auto_data.get_data(&assets);
        commands.entity(e).insert(data).remove::<AutoStruct<T>>();
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