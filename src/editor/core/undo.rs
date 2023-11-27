use std::{sync::Arc, f32::consts::E};

use bevy::{prelude::*, utils::HashMap, ecs::entity::{EntityMapper, MapEntities}};

use crate::{PrefabMarker, EditorSet};

const MAX_REFLECT_RECURSION : i32 = 10;

pub struct UndoPlugin;

#[derive(SystemSet, Hash, PartialEq, Eq, Debug, Clone)]
pub enum UndoSet {
    PerType,
    Global
}

impl Plugin for UndoPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ChangeChain>();
        app.init_resource::<UndoIngnoreStorage>();

        app.add_event::<NewChange>();
        app.add_event::<UndoRedo>();

        app.configure_sets(PostUpdate, (UndoSet::PerType, UndoSet::Global).chain().in_set(EditorSet::Editor));

        app.add_systems(
            PostUpdate,
            (clear_one_frame_ignore, update_change_chain, undo_redo_logic, undo_ignore_tick).chain().in_set( UndoSet::Global),
        );
    }
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
        Self { counter: 4 }
    }
}

fn update_change_chain(mut change_chain: ResMut<ChangeChain>, mut events: EventReader<NewChange>) {
    let mut new_changes = vec![];
    for event in events.read() {
        new_changes.push(event.change.clone());
        change_chain.changes_for_redo.clear();
    }

    if new_changes.len() == 1 {
        change_chain.changes.push(new_changes[0].clone());
    } else if new_changes.len() > 1 {
        change_chain.changes.push(Arc::new(ManyChanges { changes: new_changes }));
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
                                change.revert(world, &change_chain.entity_remap).unwrap();
                                change_chain.changes_for_redo.push(change);
                            }
                        }
                        UndoRedo::Redo => {
                            if let Some(redo_change) = change_chain.changes_for_redo.pop() {
                                redo_change
                                    .apply(world, &change_chain.entity_remap)
                                    .unwrap();
                                change_chain.changes.push(redo_change);
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

impl ChangeChain {
    pub fn undo(&mut self, world: &mut World) {
        if let Some(change) = self.changes.pop() {
            let res = change.revert(world, &self.entity_remap).unwrap();
            self.changes_for_redo.push(change);
            self.update_remap(res);
        }
    }

    pub fn redo(&mut self, world: &mut World) {
        if let Some(change) = self.changes_for_redo.pop() {
            let res = change.apply(world, &self.entity_remap).unwrap();
            self.changes.push(change);
            self.update_remap(res);
        }
    }

    fn update_remap(&mut self, result: ChangeResult) {
        match result {
            ChangeResult::Success => {}
            ChangeResult::SuccessWithRemap(new_remap) => {
                for (prev, new) in new_remap {
                    self.entity_remap.insert(prev, new);
                }
            }
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
    fn apply(
        &self,
        world: &mut World,
        entity_remap: &HashMap<Entity, Entity>,
    ) -> Result<ChangeResult, String>;
    fn debug_text(&self) -> String;
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

#[derive(Event)]
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
        world.resource_mut::<UndoIngnoreStorage>()
            .storage
            .insert(e, OneFrameUndoIgnore::default());
        Ok(ChangeResult::Success)
    }

    fn apply(
        &self,
        world: &mut World,
        _: &HashMap<Entity, Entity>,
    ) -> Result<ChangeResult, String> {
        let new_id = world
            .spawn_empty()
            .insert(OneFrameUndoIgnore::default())
            .id();
        Ok(ChangeResult::SuccessWithRemap(vec![(self.entity, new_id)]))
    }

    fn debug_text(&self) -> String {
        format!("Added Entity: {}", self.entity.index())
    }
}

pub struct RemovedEntity {
    pub entity: Entity,
}

impl EditorChange for RemovedEntity {
    fn revert(
        &self,
        world: &mut World,
        _: &HashMap<Entity, Entity>,
    ) -> Result<ChangeResult, String> {
        let id = world.spawn_empty().insert(OneFrameUndoIgnore::default()).id();
        Ok(ChangeResult::SuccessWithRemap(vec![(self.entity, id)]))
    }

    fn apply(
        &self,
        world: &mut World,
        entity_remap: &HashMap<Entity, Entity>,
    ) -> Result<ChangeResult, String> {
        let e = get_entity_with_remap(self.entity, entity_remap);
        world.entity_mut(e).despawn_recursive();
        
        world.resource_mut::<UndoIngnoreStorage>()
            .storage
            .insert(e, OneFrameUndoIgnore::default());
        
        Ok(ChangeResult::Success)
    }

    fn debug_text(&self) -> String {
        format!("Removed Entity: {}", self.entity.index())
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
            .insert(self.old_value.clone())
            .insert(OneFrameUndoIgnore::default());
        Ok(ChangeResult::Success)
    }

    fn apply(
        &self,
        world: &mut World,
        entity_remap: &HashMap<Entity, Entity>,
    ) -> Result<ChangeResult, String> {
        world
            .entity_mut(get_entity_with_remap(self.entity, entity_remap))
            .insert(self.new_value.clone())
            .insert(OneFrameUndoIgnore::default());
        Ok(ChangeResult::Success)
    }

    fn debug_text(&self) -> String {
        format!("ComponentChange for entity {:?}", self.entity)
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
        
        world
            .entity_mut(e)
            .insert(<T as FromReflect>::from_reflect(&self.old_value).unwrap())
            .insert(OneFrameUndoIgnore::default());
        Ok(ChangeResult::Success)
    }

    fn apply(
        &self,
        world: &mut World,
        entity_remap: &HashMap<Entity, Entity>,
    ) -> Result<ChangeResult, String> {
        let e = get_entity_with_remap(self.entity, entity_remap);
        world
            .entity_mut(e)
            .insert(<T as FromReflect>::from_reflect(&self.new_value).unwrap())
            .insert(OneFrameUndoIgnore::default());
        Ok(ChangeResult::Success)
    }

    fn debug_text(&self) -> String {
        format!("ReflectedComponentChange for entity {:?}", self.entity)
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
                .resource_mut::<UndoIngnoreStorage>()
                .storage
                .insert(e, OneFrameUndoIgnore::default());
        }

        Ok(ChangeResult::Success)
    }

    fn apply(
        &self,
        world: &mut World,
        entity_remap: &HashMap<Entity, Entity>,
    ) -> Result<ChangeResult, String> {
        let e = get_entity_with_remap(self.entity, entity_remap);
        world
            .get_or_spawn(e)
            .unwrap()
            .insert(self.new_value.clone())
            .insert(OneFrameUndoIgnore::default());
        Ok(ChangeResult::Success)
    }

    fn debug_text(&self) -> String {
        format!("AddedComponent for entity {:?}", self.entity)
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
        let e = get_entity_with_remap(self.entity, entity_remap);
        world
            .entity_mut(e)
            .remove::<T>()
            .insert(OneFrameUndoIgnore::default());
        world
            .resource_mut::<UndoIngnoreStorage>()
            .storage
            .insert(e, OneFrameUndoIgnore::default());
        Ok(ChangeResult::Success)
    }

    fn apply(
        &self,
        world: &mut World,
        entity_remap: &HashMap<Entity, Entity>,
    ) -> Result<ChangeResult, String> {
        let e = get_entity_with_remap(self.entity, entity_remap);
        let id = world
            .get_or_spawn(e)
            .unwrap()
            .insert(<T as FromReflect>::from_reflect(&self.new_value).unwrap())
            .insert(OneFrameUndoIgnore::default()).id();
        Ok(ChangeResult::SuccessWithRemap(vec![(e, id)]))
    }

    fn debug_text(&self) -> String {
        format!("ReflectedAddedComponent for entity {:?}", self.entity)
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
        let e = get_entity_with_remap(self.entity, entity_remap);
        let mut new_id = if let Some(e) = world.get_entity_mut(e) {
            e
        } else {
            world.spawn_empty()
        };
        let new_id = new_id
            .insert(self.old_value.clone())
            .insert(OneFrameUndoIgnore::default())
            .id();
        Ok(ChangeResult::SuccessWithRemap(vec![(e, new_id)]))
    }

    fn apply(
        &self,
        world: &mut World,
        entity_remap: &HashMap<Entity, Entity>,
    ) -> Result<ChangeResult, String> {
        let e = get_entity_with_remap(self.entity, entity_remap);
        let new_id = world
            .get_or_spawn(e)
            .unwrap()
            .remove::<T>()
            .insert(OneFrameUndoIgnore::default())
            .id();
        
        world
            .resource_mut::<UndoIngnoreStorage>()
            .storage
            .insert(new_id, OneFrameUndoIgnore::default());

        Ok(ChangeResult::SuccessWithRemap(vec![(e, new_id)]))
    }

    fn debug_text(&self) -> String {
        format!("RemovedComponent for entity {:?}", self.entity)
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
        let e = get_entity_with_remap(self.entity, entity_remap);
        let mut new_id = if let Some(e) = world.get_entity_mut(e) {
            e
        } else {
            world.spawn_empty()
        };
        let new_id = new_id
            .insert(<T as FromReflect>::from_reflect(&self.old_value).unwrap())
            .insert(OneFrameUndoIgnore::default())
            .id();
        Ok(ChangeResult::SuccessWithRemap(vec![(e, new_id)]))
    }

    fn apply(
        &self,
        world: &mut World,
        entity_remap: &HashMap<Entity, Entity>,
    ) -> Result<ChangeResult, String> {
        let e = get_entity_with_remap(self.entity, entity_remap);
        let new_id = world
            .get_or_spawn(e)
            .unwrap()
            .remove::<T>()
            .insert(OneFrameUndoIgnore::default())
            .id();
        world
            .resource_mut::<UndoIngnoreStorage>()
            .storage
            .insert(new_id, OneFrameUndoIgnore::default());

        Ok(ChangeResult::SuccessWithRemap(vec![(e, new_id)]))
    }

    fn debug_text(&self) -> String {
        format!("ReflectedRemovedComponent for entity {:?}", self.entity)
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
        let mut remap = Vec::new();
        for change in self.changes.iter() {
            let res = change.revert(world, entity_remap)?;
            match res {
                ChangeResult::Success => {}
                ChangeResult::SuccessWithRemap(new_remap) => {
                    remap.extend(new_remap);
                }
            }
        }
        Ok(ChangeResult::SuccessWithRemap(remap))
    }

    fn apply(
        &self,
        world: &mut World,
        entity_remap: &HashMap<Entity, Entity>,
    ) -> Result<ChangeResult, String> {
        let mut remap = Vec::new();
        for change in self.changes.iter() {
            let res = change.apply(world, entity_remap)?;
            match res {
                ChangeResult::Success => {}
                ChangeResult::SuccessWithRemap(new_remap) => {
                    remap.extend(new_remap);
                }
            }
        }
        Ok(ChangeResult::SuccessWithRemap(remap))
    }

    fn debug_text(&self) -> String {
        format!("ManyChanges")
    }
}

#[derive(Component)]
pub struct ChangedMarker<T> {
    _phantom: std::marker::PhantomData<T>,
}

impl<T> Default for ChangedMarker<T> {
    fn default() -> Self {
        Self {
            _phantom: std::marker::PhantomData,
        }
    }
}

#[derive(Resource, Default)]
pub struct UndoIngnoreStorage {
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

    //Allow more complex undo and auto entity remaping
    fn auto_reflected_undo<T: Component + Reflect + FromReflect>(&mut self) -> &mut Self; 
}

impl AppAutoUndo for App {
    fn auto_undo<T: Component + Clone>(&mut self) -> &mut Self {

        self.world.insert_resource(AutoUndoStorage::<T>::default());
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
                .chain().in_set(UndoSet::PerType),
        );

        self
    }

    fn auto_reflected_undo<T: Component + Reflect + FromReflect>(&mut self) -> &mut Self {
        
        self.world.insert_resource(AutoUndoStorage::<T>::default());
        self.add_event::<UndoRedoApplied<T>>();

        self.add_systems(
            PostUpdate,
            (
                auto_undo_reflected_update_cache::<T>,
                auto_undo_reflected_add_init::<T>,
                auto_undo_reflected_remove_detect::<T>,
                apply_deferred,
                auto_remap_undo_redo::<T>,
                auto_undo_system_changed::<T>,
                auto_undo_reflected_system::<T>,
            )
                .chain().in_set(UndoSet::PerType),
        );

        self
    }
}

fn apply_for_every_typed_field<D : Reflect>(
    value : &mut dyn Reflect,
    applyer : &dyn Fn(&mut D),
    max_recursion : i32
) {
    if max_recursion < 0 {
        return;
    }
    if let Some(v) = value.as_any_mut().downcast_mut::<D>() {
        applyer(v);
    } else {
        match value.reflect_mut() {
            bevy::reflect::ReflectMut::Struct(s) => {
                for field_idx in 0..s.field_len() {
                    apply_for_every_typed_field(
                        s.field_at_mut(field_idx).unwrap(),
                         applyer,
                        max_recursion - 1
                    );
                }
            },
            bevy::reflect::ReflectMut::TupleStruct(s) => {
                for field_idx in 0..s.field_len() {
                    apply_for_every_typed_field(
                        s.field_mut(field_idx).unwrap(),
                         applyer,
                        max_recursion - 1
                    );
                }
            },
            bevy::reflect::ReflectMut::Tuple(s) => {
                for field_idx in 0..s.field_len() {
                    apply_for_every_typed_field(
                        s.field_mut(field_idx).unwrap(),
                         applyer,
                        max_recursion - 1
                    );
                }
            },
            bevy::reflect::ReflectMut::List(s) => {
                for field_idx in 0..s.len() {
                    apply_for_every_typed_field(
                        s.get_mut(field_idx).unwrap(),
                         applyer,
                        max_recursion - 1
                    )
                }
            },
            bevy::reflect::ReflectMut::Array(s) => {
                for field_idx in 0..s.len() {
                    apply_for_every_typed_field(
                        s.get_mut(field_idx).unwrap(),
                         applyer,
                        max_recursion - 1
                    );
                }
            },
            bevy::reflect::ReflectMut::Map(s) => {
                for field_idx in 0..s.len() {
                    let (key, value) = s.get_at_mut(field_idx).unwrap();
                    apply_for_every_typed_field(
                        value,
                        applyer,
                        max_recursion - 1
                    );
                }
            },
            bevy::reflect::ReflectMut::Enum(s) => {
                for field_idx in 0..s.field_len() {
                    apply_for_every_typed_field(
                        s.field_at_mut(field_idx).unwrap(),
                         applyer,
                        max_recursion - 1
                    );
                }
            },
            bevy::reflect::ReflectMut::Value(v) => {
                //do nothing. Value was checked before
            },
        }
    }
}

fn auto_remap_undo_redo<T: Component + Reflect>(
    change_chain : Res<ChangeChain>,
    mut query : Query<&mut T>,
    mut undoredo_applied : EventReader<UndoRedoApplied<T>>,
) {
    for event in undoredo_applied.read() {
        if let Ok(mut data) = query.get_mut(event.entity) {
            let reflect = data.as_reflect_mut();
            
            apply_for_every_typed_field::<Entity>(
                reflect,
                &|v| {
                    if let Some(e) = change_chain.entity_remap.get(v) {
                        *v = *e;
                    }
                },
                MAX_REFLECT_RECURSION
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
        storage.storage.insert(e, <T as FromReflect>::from_reflect(data).unwrap());
    }
}

fn auto_undo_add_init<T: Component + Clone>(
    mut commands: Commands,
    mut storage: ResMut<AutoUndoStorage<T>>,
    query: Query<(Entity, &T), (With<PrefabMarker>, Added<T>, Without<OneFrameUndoIgnore>)>,
    mut new_changes : EventWriter<NewChange>
) {
    for (e, data) in query.iter() {
        storage.storage.insert(e, data.clone());
        commands.entity(e).insert(OneFrameUndoIgnore::default());
        new_changes.send(NewChange {
            change: Arc::new(AddedComponent {
                new_value: data.clone(),
                entity: e,
            })
        })
    }
}

fn auto_undo_reflected_add_init<T: Component + Reflect + FromReflect>(
    mut commands: Commands,
    mut storage: ResMut<AutoUndoStorage<T>>,
    query: Query<(Entity, &T), (With<PrefabMarker>, Added<T>, Without<OneFrameUndoIgnore>)>,
    mut new_changes : EventWriter<NewChange>
) {
    for (e, data) in query.iter() {
        storage.storage.insert(e, <T as FromReflect>::from_reflect(data).unwrap());
        commands.entity(e).insert(OneFrameUndoIgnore::default());
        new_changes.send(NewChange {
            change: Arc::new(ReflectedAddedComponent {
                new_value: <T as FromReflect>::from_reflect(data).unwrap(),
                entity: e,
            })
        })
    }
}

fn undo_ignore_tick(
    mut ignore_storage : ResMut<UndoIngnoreStorage>
) {
    for (_, frame) in ignore_storage.storage.iter_mut() {
        frame.counter -= 1;
    }
    ignore_storage.storage.retain(|_, frame| frame.counter > 0);
}

fn auto_undo_remove_detect<T: Component + Clone>(
    mut commands: Commands,
    mut storage: ResMut<AutoUndoStorage<T>>,
    mut removed_query : RemovedComponents<T>,
    mut new_changes : EventWriter<NewChange>,
    mut ignore_storage : ResMut<UndoIngnoreStorage>) {
    
    for e in removed_query.read() {
        if !ignore_storage.storage.contains_key(&e) {
            if let Some(prev_value) = storage.storage.remove(&e) {
                new_changes.send(NewChange {
                    change: Arc::new(RemovedComponent {
                        old_value: prev_value,
                        entity: e,
                    })
                });
            }
        }
    }
}

fn auto_undo_reflected_remove_detect<T: Component + Reflect + FromReflect>(
    mut commands: Commands,
    mut storage: ResMut<AutoUndoStorage<T>>,
    mut removed_query : RemovedComponents<T>,
    mut new_changes : EventWriter<NewChange>,
    mut ignore_storage : ResMut<UndoIngnoreStorage>) {
    
    for e in removed_query.read() {
        if !ignore_storage.storage.contains_key(&e) {
            if let Some(prev_value) = storage.storage.remove(&e) {
                new_changes.send(NewChange {
                    change: Arc::new(ReflectedRemovedComponent {
                        old_value: prev_value,
                        entity: e,
                    })
                });
            }
        }
    }


}

fn auto_undo_system_changed<T: Component>(
    mut commands: Commands,
    query: Query<Entity, (With<PrefabMarker>, Changed<T>, Without<OneFrameUndoIgnore>)>,
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
                info!("Auto undo change for entity {:?}", e);
            }

            storage.storage.insert(e, data.clone());
        }
    }
}

fn auto_undo_reflected_system<T: Component + Reflect + FromReflect>(
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
                    change: Arc::new(ReflectedComponentChange {
                        old_value: <T as FromReflect>::from_reflect(prev_value).unwrap(),
                        new_value: <T as FromReflect>::from_reflect(data.as_ref()).unwrap(),
                        entity: e,
                    }),
                });
                info!("Auto undo change for entity {:?}", e);
            }

            storage.storage.insert(e, <T as FromReflect>::from_reflect(data.as_ref()).unwrap());
        }
    }
}
