// Remove after update to newer rust version
#![allow(clippy::type_complexity)]
#[cfg(test)]
mod tests;

use std::sync::Arc;

use bevy::{prelude::*, utils::HashMap};

const MAX_REFLECT_RECURSION: i32 = 10;
const AUTO_UNDO_LATENCY: i32 = 2;

#[derive(Default)]
pub struct UndoPlugin;

/// Components with this marker will be used for undo
#[derive(Component)]
pub struct UndoMarker;

impl Plugin for UndoPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ChangeChain>();
        app.init_resource::<UndoIgnoreStorage>();
        app.init_resource::<ChangeChainSettings>();

        app.add_event::<NewChange>();
        app.add_event::<UndoRedo>();

        app.configure_sets(
            PostUpdate,
            (UndoSet::PerType, UndoSet::UpdateAll, UndoSet::Remapping)
                .chain()
                .in_set(UndoSet::Global),
        );

        app.add_systems(
            PostUpdate,
            (
                clear_one_frame_ignore,
                update_change_chain,
                undo_redo_logic,
                undo_ignore_tick,
            )
                .chain()
                .in_set(UndoSet::UpdateAll),
        );
    }
}

/// Allows to make UndoMarker attached to another marker M so that
/// if there is an entity with marker M, then UndoMarker will be added to that entity,
/// and likewise, if there is an entity with UndoMarker but without marker M, then UndoMarker will be removed
#[derive(Default)]
pub struct SyncUndoMarkersPlugin<M: Component> {
    _phantom: std::marker::PhantomData<M>,
}

impl<M: Component> Plugin for SyncUndoMarkersPlugin<M> {
    fn build(&self, app: &mut App) {
        app.add_systems(PostUpdate, sync_system::<M>);
    }
}

fn sync_system<M: Component>(
    mut commands: Commands,
    add_undo: Query<Entity, (With<M>, Without<UndoMarker>)>,
    remove_undo: Query<Entity, (Without<M>, With<UndoMarker>)>,
) {
    for e in add_undo.iter() {
        commands.entity(e).insert(UndoMarker);
    }

    for e in remove_undo.iter() {
        commands.entity(e).remove::<UndoMarker>();
    }
}

#[derive(SystemSet, Hash, PartialEq, Eq, Debug, Clone)]
pub enum UndoSet {
    /// Per component systems
    PerType,
    /// System which working with change chain and global logic
    UpdateAll,
    /// Remap entities
    Remapping,
    ///Contains all undo sets
    Global,
}

#[derive(Event)]
pub struct UndoRedoApplied<T> {
    pub entity: Entity,
    _phantom: std::marker::PhantomData<T>,
}

#[derive(Component)]
pub struct OneFrameUndoIgnore {
    pub counter: i32,
}

impl Default for OneFrameUndoIgnore {
    fn default() -> Self {
        Self { counter: 10 }
    }
}

fn update_change_chain(
    mut buffer: Local<Vec<NewChange>>, //Buffer will use for chain reaction changes and collecting them together
    settings: Res<ChangeChainSettings>,
    mut change_chain: ResMut<ChangeChain>,
    mut events: EventReader<NewChange>,
) {
    //collect buffer
    let mut events_on_current_frame = 0;
    for event in events.read() {
        buffer.push(event.clone());
        events_on_current_frame += 1;
    }

    if events_on_current_frame > 0 {
        return;
    }

    if buffer.is_empty() {
        return;
    }

    //Drop buffer to vec of arc
    let mut new_changes = vec![];
    for b in buffer.iter() {
        new_changes.push(b.change.clone());
        change_chain.changes_for_redo.clear();
    }

    //Clear buffer
    buffer.clear();

    match new_changes.len().cmp(&1) {
        std::cmp::Ordering::Less => {}
        std::cmp::Ordering::Equal => {
            change_chain.changes.push(new_changes[0].clone());
        }
        std::cmp::Ordering::Greater => {
            change_chain.changes.push(Arc::new(ManyChanges {
                changes: new_changes,
            }));
        }
    };

    if change_chain.changes.len() > settings.max_change_chain_size {
        let count = change_chain.changes.len() - settings.max_change_chain_size;
        change_chain.changes.drain(0..count);
    }
}

fn clear_one_frame_ignore(
    mut commands: Commands,
    mut query: Query<(Entity, &mut OneFrameUndoIgnore)>,
) {
    for (e, mut ignore) in query.iter_mut() {
        ignore.counter -= 1;
        if ignore.counter <= 0 {
            commands.entity(e).remove::<OneFrameUndoIgnore>();
        }
    }
}

fn undo_redo_logic(world: &mut World) {
    world.resource_scope::<Events<UndoRedo>, _>(|world, mut events| {
        world.resource_scope::<ChangeChain, _>(|world, mut change_chain| {
            {
                let mut reader = events.get_reader();
                for event in reader.read(&events) {
                    match event {
                        UndoRedo::Undo => {
                            if let Some(change) = change_chain.changes.pop() {
                                let res = change.revert(world, &change_chain.entity_remap).unwrap();
                                if let ChangeResult::SuccessWithRemap(remap) = res {
                                    change_chain.entity_remap.extend(remap);
                                }
                                change_chain.changes_for_redo.push(change);
                            }
                        }
                        UndoRedo::Redo => {
                            if let Some(change) = change_chain.changes_for_redo.pop() {
                                let inverse_change = change.get_inverse();
                                let res = inverse_change
                                    .revert(world, &change_chain.entity_remap)
                                    .unwrap();
                                if let ChangeResult::SuccessWithRemap(remap) = res {
                                    change_chain.entity_remap.extend(remap);
                                }
                                change_chain.changes.push(change);
                            }
                        }
                    }
                }
            }
            events.clear();
        });
    });
}

#[derive(Resource, Default)]
pub struct ChangeChain {
    pub changes: Vec<Arc<dyn EditorChange + Send + Sync>>,
    pub changes_for_redo: Vec<Arc<dyn EditorChange + Send + Sync>>,
    entity_remap: HashMap<Entity, Entity>,
}

#[derive(Resource, Reflect)]
#[reflect(Resource, Default)]
pub struct ChangeChainSettings {
    pub max_change_chain_size: usize,
}

impl Default for ChangeChainSettings {
    fn default() -> Self {
        Self {
            max_change_chain_size: 200,
        }
    }
}

pub fn get_entity_with_remap(entity: Entity, entity_remap: &HashMap<Entity, Entity>) -> Entity {
    *entity_remap.get(&entity).unwrap_or(&entity)
}

pub trait EditorChange {
    fn revert(
        &self,
        world: &mut World,
        entity_remap: &HashMap<Entity, Entity>,
    ) -> Result<ChangeResult, String>;

    fn debug_text(&self) -> String;

    fn get_inverse(&self) -> Arc<dyn EditorChange + Send + Sync>;
}

pub enum ChangeResult {
    Success,
    SuccessWithRemap(Vec<(Entity, Entity)>),
}

#[derive(Event)]
pub enum UndoRedo {
    Undo,
    Redo,
}

#[derive(Event, Clone)]
pub struct NewChange {
    pub change: Arc<dyn EditorChange + Send + Sync>,
}

pub struct AddedEntity {
    pub entity: Entity,
}

impl EditorChange for AddedEntity {
    fn revert(
        &self,
        world: &mut World,
        entity_remap: &HashMap<Entity, Entity>,
    ) -> Result<ChangeResult, String> {
        let e = get_entity_with_remap(self.entity, entity_remap);
        world.entity_mut(e).despawn_recursive();
        world
            .resource_mut::<UndoIgnoreStorage>()
            .storage
            .insert(e, OneFrameUndoIgnore::default());
        info!("Removed Entity: {}", e.index());
        Ok(ChangeResult::Success)
    }

    fn debug_text(&self) -> String {
        format!("Added Entity: {}", self.entity.index())
    }

    fn get_inverse(&self) -> Arc<dyn EditorChange + Send + Sync> {
        Arc::new(RemovedEntity {
            entity: self.entity,
        })
    }
}

pub struct RemovedEntity {
    pub entity: Entity,
}

impl EditorChange for RemovedEntity {
    fn revert(
        &self,
        world: &mut World,
        remap: &HashMap<Entity, Entity>,
    ) -> Result<ChangeResult, String> {
        if let Some(e) = remap.get(&self.entity) {
            if world.get_entity(*e).is_none() {
                let id = world
                    .spawn_empty()
                    .insert((UndoMarker, OneFrameUndoIgnore::default()))
                    .id();
                info!("Reverted Removed Entity: {}", e.index());
                Ok(ChangeResult::SuccessWithRemap(vec![(self.entity, id)]))
            } else {
                info!("Reverted Removed Entity: {}", e.index());
                Ok(ChangeResult::Success)
            }
        } else {
            let id = world
                .spawn_empty()
                .insert((UndoMarker, OneFrameUndoIgnore::default()))
                .id();
            info!("Reverted Removed Entity: {}", self.entity.index());
            Ok(ChangeResult::SuccessWithRemap(vec![(self.entity, id)]))
        }
    }

    fn debug_text(&self) -> String {
        format!("Removed Entity: {}", self.entity.index())
    }

    fn get_inverse(&self) -> Arc<dyn EditorChange + Send + Sync> {
        Arc::new(AddedEntity {
            entity: self.entity,
        })
    }
}

pub struct ComponentChange<T: Component> {
    old_value: T,
    new_value: T,
    entity: Entity,
}

impl<T: Component + Clone> EditorChange for ComponentChange<T> {
    fn revert(
        &self,
        world: &mut World,
        entity_remap: &HashMap<Entity, Entity>,
    ) -> Result<ChangeResult, String> {
        let e = get_entity_with_remap(self.entity, entity_remap);

        world
            .entity_mut(e)
            .insert((self.old_value.clone(), OneFrameUndoIgnore::default()));
        info!("Reverted ComponentChange for entity: {}", e.index());
        Ok(ChangeResult::Success)
    }

    fn debug_text(&self) -> String {
        format!("ComponentChange for entity {:?}", self.entity)
    }

    fn get_inverse(&self) -> Arc<dyn EditorChange + Send + Sync> {
        Arc::new(Self {
            old_value: self.new_value.clone(),
            new_value: self.old_value.clone(),
            entity: self.entity,
        })
    }
}

pub struct ReflectedComponentChange<T: Component + Reflect + FromReflect> {
    old_value: T,
    new_value: T,
    entity: Entity,
}

impl<T: Component + Reflect + FromReflect> EditorChange for ReflectedComponentChange<T> {
    fn revert(
        &self,
        world: &mut World,
        entity_remap: &HashMap<Entity, Entity>,
    ) -> Result<ChangeResult, String> {
        let e = get_entity_with_remap(self.entity, entity_remap);

        world.entity_mut(e).insert((
            <T as FromReflect>::from_reflect(&self.old_value)
                .ok_or(format!("Failed to revert reflected entity `{:?}`", e))?,
            OneFrameUndoIgnore::default(),
        ));
        world.send_event(UndoRedoApplied::<T> {
            entity: e,
            _phantom: std::marker::PhantomData,
        });

        info!(
            "Reverted ReflectedComponentChange for entity: {}",
            e.index()
        );
        Ok(ChangeResult::Success)
    }

    fn debug_text(&self) -> String {
        format!(
            "{:?} changed for entity {:?}",
            pretty_type_name::pretty_type_name::<T>(),
            self.entity
        )
    }

    fn get_inverse(&self) -> Arc<dyn EditorChange + Send + Sync> {
        Arc::new(Self {
            old_value: <T as FromReflect>::from_reflect(&self.new_value).unwrap(),
            new_value: <T as FromReflect>::from_reflect(&self.old_value).unwrap(),
            entity: self.entity,
        })
    }
}

pub struct AddedComponent<T: Component> {
    new_value: T,
    entity: Entity,
}

impl<T: Component + Clone> EditorChange for AddedComponent<T> {
    fn revert(
        &self,
        world: &mut World,
        entity_remap: &HashMap<Entity, Entity>,
    ) -> Result<ChangeResult, String> {
        let e = get_entity_with_remap(self.entity, entity_remap);
        let mut add_to_ignore = false;
        if let Some(mut e) = world.get_entity_mut(e) {
            e.remove::<T>().insert(OneFrameUndoIgnore::default());
            add_to_ignore = true;
        }
        if add_to_ignore {
            world
                .resource_mut::<UndoIgnoreStorage>()
                .storage
                .insert(e, OneFrameUndoIgnore::default());
        }

        info!("Reverted AddedComponent for entity: {}", e.index());

        Ok(ChangeResult::Success)
    }

    fn debug_text(&self) -> String {
        format!("AddedComponent for entity {:?}", self.entity)
    }

    fn get_inverse(&self) -> Arc<dyn EditorChange + Send + Sync> {
        Arc::new(RemovedComponent {
            entity: self.entity,
            old_value: self.new_value.clone(),
        })
    }
}

pub struct ReflectedAddedComponent<T: Component + Reflect + FromReflect> {
    new_value: T,
    entity: Entity,
}

impl<T: Component + Reflect + FromReflect> EditorChange for ReflectedAddedComponent<T> {
    fn revert(
        &self,
        world: &mut World,
        entity_remap: &HashMap<Entity, Entity>,
    ) -> Result<ChangeResult, String> {
        let dst = entity_remap
            .get(&self.entity)
            .map_or(self.entity, |remapped| *remapped);
        if let Some(mut e) = world.get_entity_mut(dst) {
            e.remove::<T>().insert(OneFrameUndoIgnore::default());
        }
        world
            .resource_mut::<UndoIgnoreStorage>()
            .storage
            .insert(dst, OneFrameUndoIgnore::default());
        world.send_event(UndoRedoApplied::<T> {
            entity: dst,
            _phantom: std::marker::PhantomData,
        });

        info!(
            "Reverted ReflectedAddedComponent for entity: {}",
            dst.index()
        );

        Ok(ChangeResult::Success)
    }

    fn debug_text(&self) -> String {
        format!("ReflectedAddedComponent for entity {:?}", self.entity)
    }

    fn get_inverse(&self) -> Arc<dyn EditorChange + Send + Sync> {
        Arc::new(ReflectedRemovedComponent {
            old_value: <T as FromReflect>::from_reflect(&self.new_value).unwrap(),
            entity: self.entity,
        })
    }
}

pub struct RemovedComponent<T: Component + Clone> {
    old_value: T,
    entity: Entity,
}

impl<T: Component + Clone> EditorChange for RemovedComponent<T> {
    fn revert(
        &self,
        world: &mut World,
        entity_remap: &HashMap<Entity, Entity>,
    ) -> Result<ChangeResult, String> {
        let mut remap = vec![];
        let dst = entity_remap.get(&self.entity).map_or_else(
            || {
                if world.get_entity(self.entity).is_some() {
                    self.entity
                } else {
                    let id = world.spawn_empty().id();
                    remap.push((self.entity, id));
                    id
                }
            },
            |remapped| *remapped,
        );

        world
            .entity_mut(dst)
            .insert((self.old_value.clone(), OneFrameUndoIgnore::default()));

        info!("Reverted RemovedComponent for entity: {}", dst.index());

        Ok(ChangeResult::SuccessWithRemap(remap))
    }

    fn debug_text(&self) -> String {
        format!("RemovedComponent for entity {:?}", self.entity)
    }

    fn get_inverse(&self) -> Arc<dyn EditorChange + Send + Sync> {
        Arc::new(AddedComponent {
            new_value: self.old_value.clone(),
            entity: self.entity,
        })
    }
}

pub struct ReflectedRemovedComponent<T: Component + Reflect> {
    old_value: T,
    entity: Entity,
}

impl<T: Component + Reflect + FromReflect> EditorChange for ReflectedRemovedComponent<T> {
    fn revert(
        &self,
        world: &mut World,
        entity_remap: &HashMap<Entity, Entity>,
    ) -> Result<ChangeResult, String> {
        let mut remap = vec![];
        let dst = entity_remap.get(&self.entity).map_or_else(
            || {
                if world.get_entity(self.entity).is_some() {
                    self.entity
                } else {
                    let id = world.spawn_empty().id();
                    remap.push((self.entity, id));
                    id
                }
            },
            |remapped| *remapped,
        );

        world.entity_mut(dst).insert((
            <T as FromReflect>::from_reflect(&self.old_value).ok_or(format!(
                "Failed to revert to destination entity `{:?}`",
                dst
            ))?,
            OneFrameUndoIgnore::default(),
        ));
        world.send_event(UndoRedoApplied::<T> {
            entity: dst,
            _phantom: std::marker::PhantomData,
        });

        info!(
            "Reverted ReflectedRemovedComponent for entity: {}",
            dst.index()
        );

        Ok(ChangeResult::SuccessWithRemap(remap))
    }

    fn debug_text(&self) -> String {
        format!("ReflectedRemovedComponent for entity {:?}", self.entity)
    }

    fn get_inverse(&self) -> Arc<dyn EditorChange + Send + Sync> {
        Arc::new(ReflectedAddedComponent {
            new_value: <T as FromReflect>::from_reflect(&self.old_value).unwrap(),
            entity: self.entity,
        })
    }
}

pub struct ManyChanges {
    changes: Vec<Arc<dyn EditorChange + Send + Sync>>,
}

impl EditorChange for ManyChanges {
    fn revert(
        &self,
        world: &mut World,
        entity_remap: &HashMap<Entity, Entity>,
    ) -> Result<ChangeResult, String> {
        let mut remap = entity_remap.clone();
        for change in self.changes.iter() {
            let res = change.revert(world, &remap)?;
            match res {
                ChangeResult::Success => {}
                ChangeResult::SuccessWithRemap(new_remap) => {
                    remap.extend(new_remap);
                }
            }
        }

        info!("Reverted ManyChanges");

        Ok(ChangeResult::SuccessWithRemap(
            remap.iter().map(|(key, value)| (*key, *value)).collect(),
        ))
    }

    fn debug_text(&self) -> String {
        "ManyChanges".to_string()
    }

    fn get_inverse(&self) -> Arc<dyn EditorChange + Send + Sync> {
        let mut old_changes = self.changes.clone();
        old_changes.reverse();
        let changes = old_changes
            .iter()
            .map(|change| change.get_inverse())
            .collect::<Vec<_>>();

        Arc::new(Self { changes })
    }
}

#[derive(Component)]
pub struct ChangedMarker<T> {
    latency: i32,
    _phantom: std::marker::PhantomData<T>,
}

impl<T> Default for ChangedMarker<T> {
    fn default() -> Self {
        Self {
            latency: AUTO_UNDO_LATENCY, //2 frame latency
            _phantom: std::marker::PhantomData,
        }
    }
}

#[derive(Resource, Default)]
pub struct UndoIgnoreStorage {
    pub storage: HashMap<Entity, OneFrameUndoIgnore>,
}

#[derive(Resource)]
pub struct AutoUndoStorage<T: Component> {
    pub storage: HashMap<Entity, T>,
}

impl<T: Component> Default for AutoUndoStorage<T> {
    fn default() -> Self {
        Self {
            storage: HashMap::new(),
        }
    }
}

pub trait AppAutoUndo {
    fn auto_undo<T: Component + Clone>(&mut self) -> &mut Self;

    //Allow more complex undo and auto entity remapping
    fn auto_reflected_undo<T: Component + Reflect + FromReflect>(&mut self) -> &mut Self;
}

impl AppAutoUndo for App {
    fn auto_undo<T: Component + Clone>(&mut self) -> &mut Self {
        if !self.world_mut().contains_resource::<ChangeChain>() {
            return self;
        }

        self.world_mut()
            .insert_resource(AutoUndoStorage::<T>::default());
        self.add_event::<UndoRedoApplied<T>>();

        self.add_systems(
            PostUpdate,
            (
                auto_undo_update_cache::<T>,
                auto_undo_add_init::<T>,
                auto_undo_remove_detect::<T>,
                apply_deferred,
                auto_undo_system_changed::<T>,
                auto_undo_system::<T>,
            )
                .chain()
                .in_set(UndoSet::PerType),
        );

        self
    }

    fn auto_reflected_undo<T: Component + Reflect + FromReflect>(&mut self) -> &mut Self {
        if !self.world_mut().contains_resource::<ChangeChain>() {
            return self;
        }

        self.world_mut()
            .insert_resource(AutoUndoStorage::<T>::default());
        self.add_event::<UndoRedoApplied<T>>();

        self.add_systems(
            PostUpdate,
            (
                auto_undo_reflected_update_cache::<T>,
                auto_undo_reflected_add_init::<T>,
                auto_undo_reflected_remove_detect::<T>,
                apply_deferred,
                auto_undo_system_changed::<T>,
                auto_undo_reflected_system::<T>,
            )
                .chain()
                .in_set(UndoSet::PerType),
        );

        self.add_systems(
            PostUpdate,
            auto_remap_undo_redo::<T>.in_set(UndoSet::Remapping),
        );

        self
    }
}

fn apply_for_every_typed_field<D: Reflect>(
    value: &mut dyn Reflect,
    applyer: &dyn Fn(&mut D),
    max_recursion: i32,
) {
    if max_recursion < 0 {
        return;
    }
    #[allow(clippy::option_if_let_else)]
    if let Some(v) = value.as_any_mut().downcast_mut::<D>() {
        applyer(v);
    } else {
        match value.reflect_mut() {
            bevy::reflect::ReflectMut::Struct(s) => {
                for field_idx in 0..s.field_len() {
                    apply_for_every_typed_field(
                        s.field_at_mut(field_idx).unwrap(),
                        applyer,
                        max_recursion - 1,
                    );
                }
            }
            bevy::reflect::ReflectMut::TupleStruct(s) => {
                for field_idx in 0..s.field_len() {
                    apply_for_every_typed_field(
                        s.field_mut(field_idx).unwrap(),
                        applyer,
                        max_recursion - 1,
                    );
                }
            }
            bevy::reflect::ReflectMut::Tuple(s) => {
                for field_idx in 0..s.field_len() {
                    apply_for_every_typed_field(
                        s.field_mut(field_idx).unwrap(),
                        applyer,
                        max_recursion - 1,
                    );
                }
            }
            bevy::reflect::ReflectMut::List(s) => {
                for field_idx in 0..s.len() {
                    apply_for_every_typed_field(
                        s.get_mut(field_idx).unwrap(),
                        applyer,
                        max_recursion - 1,
                    )
                }
            }
            bevy::reflect::ReflectMut::Array(s) => {
                for field_idx in 0..s.len() {
                    apply_for_every_typed_field(
                        s.get_mut(field_idx).unwrap(),
                        applyer,
                        max_recursion - 1,
                    );
                }
            }
            bevy::reflect::ReflectMut::Map(s) => {
                for field_idx in 0..s.len() {
                    let (_key, value) = s.get_at_mut(field_idx).unwrap();
                    apply_for_every_typed_field(value, applyer, max_recursion - 1);
                }
            }
            bevy::reflect::ReflectMut::Enum(s) => {
                for field_idx in 0..s.field_len() {
                    apply_for_every_typed_field(
                        s.field_at_mut(field_idx).unwrap(),
                        applyer,
                        max_recursion - 1,
                    );
                }
            }
            bevy::reflect::ReflectMut::Value(_v) => {
                //do nothing. Value was checked before
            }
        }
    }
}

fn auto_remap_undo_redo<T: Component + Reflect>(
    change_chain: Res<ChangeChain>,
    mut query: Query<&mut T>,
    mut undoredo_applied: EventReader<UndoRedoApplied<T>>,
) {
    for event in undoredo_applied.read() {
        println!("remapping {:?}", event.entity);
        if let Ok(mut data) = query.get_mut(event.entity) {
            let reflect = data.as_reflect_mut();

            apply_for_every_typed_field::<Entity>(
                reflect,
                &|v| {
                    if let Some(e) = change_chain.entity_remap.get(v) {
                        println!("remap {:?} to {:?}", v, e);
                        *v = *e;
                    }
                },
                MAX_REFLECT_RECURSION,
            );
        }
    }
}

fn auto_undo_update_cache<T: Component + Clone>(
    mut storage: ResMut<AutoUndoStorage<T>>,
    ignored_query: Query<(Entity, &T), With<OneFrameUndoIgnore>>,
) {
    for (e, data) in ignored_query.iter() {
        storage.storage.insert(e, data.clone());
    }
}

fn auto_undo_reflected_update_cache<T: Component + Reflect + FromReflect>(
    mut storage: ResMut<AutoUndoStorage<T>>,
    ignored_query: Query<(Entity, &T), With<OneFrameUndoIgnore>>,
) {
    for (e, data) in ignored_query.iter() {
        storage
            .storage
            .insert(e, <T as FromReflect>::from_reflect(data).unwrap());
    }
}

fn auto_undo_add_init<T: Component + Clone>(
    mut commands: Commands,
    mut storage: ResMut<AutoUndoStorage<T>>,
    query: Query<(Entity, &T), (With<UndoMarker>, Added<T>, Without<OneFrameUndoIgnore>)>,
    just_maker_added_query: Query<(Entity, &T), (Added<UndoMarker>, Without<OneFrameUndoIgnore>)>,
    mut new_changes: EventWriter<NewChange>,
) {
    for (e, data) in query.iter() {
        storage.storage.insert(e, data.clone());
        commands.entity(e).insert(OneFrameUndoIgnore::default());
        new_changes.send(NewChange {
            change: Arc::new(AddedComponent {
                new_value: data.clone(),
                entity: e,
            }),
        });
    }

    for (e, data) in just_maker_added_query.iter() {
        storage.storage.insert(e, data.clone());
    }
}

fn auto_undo_reflected_add_init<T: Component + Reflect + FromReflect>(
    mut commands: Commands,
    mut storage: ResMut<AutoUndoStorage<T>>,
    query: Query<(Entity, &T), (With<UndoMarker>, Added<T>, Without<OneFrameUndoIgnore>)>,
    just_maker_added_query: Query<(Entity, &T), (Added<UndoMarker>, Without<OneFrameUndoIgnore>)>,
    mut new_changes: EventWriter<NewChange>,
) {
    for (e, data) in query.iter() {
        storage
            .storage
            .insert(e, <T as FromReflect>::from_reflect(data).unwrap());
        commands.entity(e).insert(OneFrameUndoIgnore::default());
        new_changes.send(NewChange {
            change: Arc::new(ReflectedAddedComponent {
                new_value: <T as FromReflect>::from_reflect(data).unwrap(),
                entity: e,
            }),
        });
    }

    for (e, data) in just_maker_added_query.iter() {
        storage
            .storage
            .insert(e, <T as FromReflect>::from_reflect(data).unwrap());
    }
}

fn undo_ignore_tick(mut ignore_storage: ResMut<UndoIgnoreStorage>) {
    for (_, frame) in ignore_storage.storage.iter_mut() {
        frame.counter -= 1;
    }
    ignore_storage.storage.retain(|_, frame| frame.counter > 0);
}

fn auto_undo_remove_detect<T: Component + Clone>(
    _commands: Commands,
    mut storage: ResMut<AutoUndoStorage<T>>,
    mut removed_query: RemovedComponents<T>,
    mut new_changes: EventWriter<NewChange>,
    ignore_storage: ResMut<UndoIgnoreStorage>,
) {
    for e in removed_query.read() {
        if !ignore_storage.storage.contains_key(&e) {
            if let Some(prev_value) = storage.storage.remove(&e) {
                new_changes.send(NewChange {
                    change: Arc::new(RemovedComponent {
                        old_value: prev_value,
                        entity: e,
                    }),
                });
            }
        }
    }
}

fn auto_undo_reflected_remove_detect<T: Component + Reflect + FromReflect>(
    _commands: Commands,
    mut storage: ResMut<AutoUndoStorage<T>>,
    mut removed_query: RemovedComponents<T>,
    mut new_changes: EventWriter<NewChange>,
    ignore_storage: ResMut<UndoIgnoreStorage>,
) {
    for e in removed_query.read() {
        if !ignore_storage.storage.contains_key(&e) {
            if let Some(prev_value) = storage.storage.remove(&e) {
                new_changes.send(NewChange {
                    change: Arc::new(ReflectedRemovedComponent {
                        old_value: prev_value,
                        entity: e,
                    }),
                });
            }
        }
    }
}

fn auto_undo_system_changed<T: Component>(
    mut commands: Commands,
    query: Query<Entity, (With<UndoMarker>, Changed<T>, Without<OneFrameUndoIgnore>)>,
) {
    for entity in query.iter() {
        commands
            .entity(entity)
            .insert(ChangedMarker::<T>::default());
    }
}

fn auto_undo_system<T: Component + Clone>(
    mut commands: Commands,
    mut storage: ResMut<AutoUndoStorage<T>>,
    mut query: Query<(Entity, &mut T), With<ChangedMarker<T>>>,
    mut new_change: EventWriter<NewChange>,
) {
    for (e, data) in query.iter_mut() {
        if !data.is_changed() {
            commands.entity(e).remove::<ChangedMarker<T>>();

            if let Some(prev_value) = storage.storage.get(&e) {
                new_change.send(NewChange {
                    change: Arc::new(ComponentChange {
                        old_value: prev_value.clone(),
                        new_value: data.clone(),
                        entity: e,
                    }),
                });
                debug!("Auto undo change for entity {:?}", e);
            }

            storage.storage.insert(e, data.clone());
        }
    }
}

fn auto_undo_reflected_system<T: Component + Reflect + FromReflect>(
    mut commands: Commands,
    mut storage: ResMut<AutoUndoStorage<T>>,
    mut query: Query<(Entity, &mut T, &mut ChangedMarker<T>)>,
    mut new_change: EventWriter<NewChange>,
) {
    for (e, data, mut marker) in query.iter_mut() {
        if !data.is_changed() {
            marker.latency -= 1;
            if marker.latency > 0 {
                continue;
            }

            commands.entity(e).remove::<ChangedMarker<T>>();

            if let Some(prev_value) = storage.storage.get(&e) {
                new_change.send(NewChange {
                    change: Arc::new(ReflectedComponentChange {
                        old_value: <T as FromReflect>::from_reflect(prev_value).unwrap(),
                        new_value: <T as FromReflect>::from_reflect(data.as_ref()).unwrap(),
                        entity: e,
                    }),
                });
                debug!("Auto undo change for entity {:?}", e);
            }

            storage
                .storage
                .insert(e, <T as FromReflect>::from_reflect(data.as_ref()).unwrap());
        } else {
            marker.latency = AUTO_UNDO_LATENCY;
        }
    }
}
