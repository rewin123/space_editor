use std::sync::Arc;

use bevy::{
    ecs::system::{EntityCommand, EntityCommands},
    prelude::*,
    reflect::{GetTypeRegistration, TypePath, TypeRegistryArc},
    utils::{HashMap, HashSet},
};
use space_shared::*;

use space_undo::AppAutoUndo;
use std::any::TypeId;

use crate::{component::AutoStruct, save::SaveState, PrefabSet};

/// Plugin to activate custom registry
pub struct EditorRegistryPlugin;

impl Plugin for EditorRegistryPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<EditorRegistry>();

        app.editor_clone_registry::<PrefabMarker>();
    }
}

/// Container struct for function to remove component in untyped style
#[derive(Clone)]
pub struct RemoveComponent {
    func: Arc<dyn Fn(&mut EntityCommands) + Send + Sync>,
}

impl RemoveComponent {
    pub fn new<T: Component>() -> Self {
        Self {
            func: Arc::new(move |cmds| {
                cmds.remove::<T>();
            }),
        }
    }
}

/// Container struct for function to clone component in untyped style
#[derive(Clone)]
pub struct CloneComponent {
    pub func: Arc<dyn Fn(&mut EntityCommands, &EntityRef) + Send + Sync>,
}

impl CloneComponent {
    pub fn new<T: Component + Reflect + FromReflect>() -> Self {
        Self {
            func: Arc::new(move |cmds, src| {
                if let Some(c) = src.get::<T>() {
                    let cloned = c.clone_value();
                    <T as FromReflect>::from_reflect(&*cloned).map_or_else(
                        || {
                            error!("Failed to clone component");
                        },
                        |taken| {
                            cmds.insert(taken);
                        },
                    );
                }
            }),
        }
    }
}

/// Container struct for function to add default component in untyped style
#[derive(Clone)]
pub struct AddDefaultComponent {
    func: Arc<dyn Fn(Entity, &mut World) + Send + Sync>,
}

impl EntityCommand for AddDefaultComponent {
    fn apply(self, id: Entity, world: &mut World) {
        (self.func)(id, world);
    }
}

impl AddDefaultComponent {
    pub fn new<T: Default + Component>() -> Self {
        Self {
            func: Arc::new(move |id, world| {
                world.entity_mut(id).insert(T::default());
            }),
        }
    }
}

/// Container struct for function to send default event
#[derive(Clone)]
pub struct SendEvent {
    name: String,
    path: String,
    pub type_id: TypeId,
    func: Arc<dyn Fn(&mut World) + Send + Sync>,
}

impl SendEvent {
    pub fn new<T: Default + Event + Resource + Clone>() -> Self {
        let path = std::any::type_name::<T>().to_string();
        let name = path.split("::").last().unwrap_or("UnnamedEvent").into();
        let type_id = TypeId::of::<T>();
        Self {
            name,
            path,
            type_id,
            func: Arc::new(move |world| {
                let event = world.resource::<T>().clone();
                world.send_event(event);
            }),
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn path(&self) -> &str {
        &self.path
    }

    pub fn send(&self, world: &mut World) {
        (self.func)(world);
    }
}

/// Resource, which contains all custom editor registry
#[derive(Default, Resource, Clone)]
pub struct EditorRegistry {
    pub registry: TypeRegistryArc,
    pub spawn_components: HashMap<TypeId, AddDefaultComponent>,
    pub clone_components: Vec<CloneComponent>,
    pub remove_components: HashMap<TypeId, RemoveComponent>,
    pub send_events: Vec<SendEvent>,
    pub silent: HashSet<TypeId>, //skip in inspector ui
}

impl EditorRegistry {
    /// Register new component, which will be shown in editor UI and saved in prefab
    pub fn register<
        T: Component + Reflect + FromReflect + Default + Send + 'static + GetTypeRegistration,
    >(
        &mut self,
    ) {
        self.registry.write().register::<T>();
        self.spawn_components.insert(
            T::get_type_registration().type_id(),
            AddDefaultComponent::new::<T>(),
        );
        self.clone_components.push(CloneComponent::new::<T>());
        self.remove_components.insert(
            T::get_type_registration().type_id(),
            RemoveComponent::new::<T>(),
        );
    }

    /// Register new component, which will be hidden in editor UI and saved in prefab
    pub fn silent_register<
        T: Component + Reflect + FromReflect + Default + Send + 'static + GetTypeRegistration,
    >(
        &mut self,
    ) {
        self.registry.write().register::<T>();
        self.spawn_components.insert(
            T::get_type_registration().type_id(),
            AddDefaultComponent::new::<T>(),
        );
        self.clone_components.push(CloneComponent::new::<T>());
        self.silent.insert(T::get_type_registration().type_id());
        self.remove_components.insert(
            T::get_type_registration().type_id(),
            RemoveComponent::new::<T>(),
        );
    }

    /// Register new component, which will be cloned with editor ui clone event
    pub fn only_clone_register<
        T: Component + Reflect + FromReflect + Default + Send + 'static + GetTypeRegistration,
    >(
        &mut self,
    ) {
        self.clone_components.push(CloneComponent::new::<T>());
    }

    /// Get spawn function for this component type
    pub fn get_spawn_command(&self, id: &TypeId) -> AddDefaultComponent {
        self.spawn_components.get(id).unwrap().clone()
    }

    /// Get remove function for this component type
    pub fn remove_by_id(&self, cmds: &mut EntityCommands, id: &TypeId) {
        if let Some(rem) = self.remove_components.get(id) {
            (rem.func)(cmds);
        }
    }

    /// Get clone function for this component type
    pub fn clone_entity_flat(&self, cmds: &mut EntityCommands, src: &EntityRef) {
        for t in &self.clone_components {
            (t.func)(cmds, src);
        }
    }

    /// Register new event, which will be shown in editor UI and can be sent
    pub fn event_register<
        T: Event + Default + Resource + Reflect + Send + Clone + 'static + GetTypeRegistration,
    >(
        &mut self,
    ) {
        self.send_events.push(SendEvent::new::<T>());
        self.send_events.sort_unstable_by_key(|send_event| {
            (send_event.name().to_owned(), send_event.path().to_owned())
        });
    }
}

pub trait EditorRegistryExt {
    /// register new component in editor UI and prefab systems
    fn editor_registry<
        T: Component + Default + Send + 'static + GetTypeRegistration + Reflect + FromReflect,
    >(
        &mut self,
    ) -> &mut Self;
    /// register new component inly in prefab systems (will be no shown in editor UI)
    fn editor_silent_registry<
        T: Component + Reflect + FromReflect + Default + Send + 'static + GetTypeRegistration,
    >(
        &mut self,
    ) -> &mut Self;

    fn editor_clone_registry<
        T: Component + Default + Reflect + FromReflect + Send + 'static + GetTypeRegistration,
    >(
        &mut self,
    ) -> &mut Self;

    /// Mark that if T component spawned, then Relation must be spawned too
    fn editor_relation<T, Relation>(&mut self) -> &mut Self
    where
        T: Component,
        Relation: Component + Default;

    /// Simple sync between prefab struct and bevy struct
    fn editor_into_sync<T, Target>(&mut self) -> &mut Self
    where
        T: Component + Clone + Into<Target>,
        Target: Component;

    /// Not used yet
    fn editor_auto_struct<T>(&mut self) -> &mut Self
    where
        T: Component
            + Reflect
            + FromReflect
            + Default
            + Clone
            + 'static
            + GetTypeRegistration
            + TypePath;

    /// register new event in editor UI
    fn editor_registry_event<
        T: Event + Default + Resource + Reflect + Send + Clone + 'static + GetTypeRegistration,
    >(
        &mut self,
    ) -> &mut Self;
}

impl EditorRegistryExt for App {
    fn editor_registry<
        T: Component + Default + Send + 'static + GetTypeRegistration + Reflect + FromReflect,
    >(
        &mut self,
    ) -> &mut Self {
        self.world.resource_mut::<EditorRegistry>().register::<T>();
        self.world.init_component::<T>();
        self.register_type::<T>();
        self.auto_reflected_undo::<T>();
        self
    }

    fn editor_clone_registry<
        T: Component + Reflect + FromReflect + Default + Send + 'static + GetTypeRegistration,
    >(
        &mut self,
    ) -> &mut Self {
        self.world
            .resource_mut::<EditorRegistry>()
            .only_clone_register::<T>();
        self.register_type::<T>();
        self
    }

    fn editor_silent_registry<
        T: Component + Reflect + FromReflect + Default + Send + 'static + GetTypeRegistration,
    >(
        &mut self,
    ) -> &mut Self {
        self.world
            .resource_mut::<EditorRegistry>()
            .silent_register::<T>();
        self.register_type::<T>();
        self
    }

    fn editor_relation<T, Relation>(&mut self) -> &mut Self
    where
        T: Component,
        Relation: Component + Default,
    {
        self.add_systems(
            Update,
            relation_system::<T, Relation>.in_set(PrefabSet::Relation),
        );
        self
    }

    //Not used now
    fn editor_auto_struct<T>(&mut self) -> &mut Self
    where
        T: Component
            + Reflect
            + FromReflect
            + Default
            + Clone
            + 'static
            + GetTypeRegistration
            + TypePath,
    {
        self.editor_silent_registry::<AutoStruct<T>>();
        self.editor_registry::<T>();

        self.add_systems(OnEnter(SaveState::Save), generate_auto_structs::<T>);
        self.add_systems(Update, despawn_auto_structs::<T>);
        self
    }

    fn editor_into_sync<T, Target>(&mut self) -> &mut Self
    where
        T: Component + Clone + Into<Target>,
        Target: Component,
    {
        self.add_systems(Update, into_sync_system::<T, Target>);

        self
    }

    fn editor_registry_event<
        T: Event + Default + Resource + Reflect + Send + Clone + 'static + GetTypeRegistration,
    >(
        &mut self,
    ) -> &mut Self {
        #[cfg(not(feature = "no_event_registration"))]
        {
            self.register_type::<T>();
            self.world.init_resource::<T>();
        }
        self.world
            .resource_mut::<EditorRegistry>()
            .event_register::<T>();
        self
    }
}

fn into_sync_system<T: Component + Clone + Into<Target>, Target: Component>(
    mut commands: Commands,
    query: Query<(Entity, &T), Changed<T>>,
) {
    for (e, t) in query.iter() {
        let new_target: Target = t.clone().into();
        commands.entity(e).insert(new_target);
    }
}

/// Not used
fn generate_auto_structs<T: Component + Reflect + FromReflect + Default + Clone>(
    mut commands: Commands,
    query: Query<(Entity, &T)>,
    assets: Res<AssetServer>,
) {
    for (e, data) in query.iter() {
        commands.entity(e).insert(AutoStruct::new(data, &assets));
    }
}

/// Not used
fn despawn_auto_structs<T: Component + Reflect + FromReflect + Default + Clone>(
    mut commands: Commands,
    query: Query<(Entity, &AutoStruct<T>)>,
    assets: Res<AssetServer>,
) {
    for (e, auto_data) in query.iter() {
        let data = auto_data.get_data(&assets);
        commands.entity(e).insert(data).remove::<AutoStruct<T>>();
    }
}

fn relation_system<T: Component, Relation: Component + Default>(
    mut commands: Commands,
    query: Query<Entity, (Added<T>, Without<Relation>)>,
) {
    for e in query.iter() {
        commands.entity(e).insert(Relation::default());
    }
}

#[cfg(test)]
mod tests {
    use bevy::{ecs::system::CommandQueue, prelude::*};

    use crate::prelude::{EditorRegistry, EditorRegistryExt, EditorRegistryPlugin};

    /// Test for clone logic in editor registry
    #[test]
    fn clone_entity_test() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_plugins(EditorRegistryPlugin);
        app.editor_registry::<Name>();

        let name = "name";
        let e = app.world.spawn(Name::new(name)).id();

        let new_e_id;
        {
            let mut command_queue = CommandQueue::default();
            let mut cmds = Commands::new(&mut command_queue, &app.world);

            let mut new_e = cmds.spawn_empty();
            new_e_id = new_e.id();

            app.world
                .resource::<EditorRegistry>()
                .clone_entity_flat(&mut new_e, &app.world.entity(e));
            command_queue.apply(&mut app.world);
        }

        assert_eq!(
            app.world.entity(new_e_id).get::<Name>().unwrap().as_str(),
            name
        );
    }
}
