use std::sync::Arc;

use bevy::{prelude::*, utils::HashMap};

use crate::PrefabMarker;

pub struct UndoPlugin;

impl Plugin for UndoPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ChangeChain>();

        app.add_event::<NewChange>();
        app.add_event::<UndoRedo>();

        app.add_systems(
            PostUpdate,
            (clear_one_frame_ignore, update_change_chain, undo_redo_logic).chain(),
        );

        app.auto_undo::<Transform>();
    }
}

#[derive(Component)]
pub struct OneFrameUndoIgnore {
    pub counter: i32,
}

impl Default for OneFrameUndoIgnore {
    fn default() -> Self {
        Self { counter: 3 }
    }
}

fn update_change_chain(mut change_chain: ResMut<ChangeChain>, mut events: EventReader<NewChange>) {
    for event in events.read() {
        change_chain.changes.push(event.change.clone());
        change_chain.changes_for_redo.clear();
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

pub struct ComponentChange<T: Component + Clone> {
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

pub struct NewEntityChange {
    entity: Entity,
}

impl EditorChange for NewEntityChange {
    fn revert(
        &self,
        world: &mut World,
        entity_remap: &HashMap<Entity, Entity>,
    ) -> Result<ChangeResult, String> {
        world
            .entity_mut(get_entity_with_remap(self.entity, entity_remap))
            .despawn();
        Ok(ChangeResult::Success)
    }

    fn apply(
        &self,
        world: &mut World,
        _entity_remap: &HashMap<Entity, Entity>,
    ) -> Result<ChangeResult, String> {
        let new_entity = world.spawn_empty().id();
        Ok(ChangeResult::SuccessWithRemap(vec![(
            self.entity,
            new_entity,
        )]))
    }

    fn debug_text(&self) -> String {
        format!("NewEntityChange for entity {:?}", self.entity)
    }
}

pub struct RemoveEntityChange {
    entity: Entity,
    scene: DynamicScene,
}

impl EditorChange for RemoveEntityChange {
    fn revert(
        &self,
        world: &mut World,
        _entity_remap: &HashMap<Entity, Entity>,
    ) -> Result<ChangeResult, String> {
        let mut entity_map = HashMap::new();

        self.scene.write_to_world(world, &mut entity_map).unwrap();
        let vec_changes = entity_map
            .into_iter()
            .map(|(prev, new)| (prev, new))
            .collect::<Vec<_>>();

        Ok(ChangeResult::SuccessWithRemap(vec_changes))
    }

    fn apply(
        &self,
        world: &mut World,
        entity_remap: &HashMap<Entity, Entity>,
    ) -> Result<ChangeResult, String> {
        world
            .entity_mut(get_entity_with_remap(self.entity, entity_remap))
            .despawn();
        Ok(ChangeResult::Success)
    }

    fn debug_text(&self) -> String {
        format!("RemoveEntityChange for entity {:?}", self.entity)
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

#[derive(Resource)]
pub struct AutoUndoStorage<T: Component + Clone> {
    pub storage: HashMap<Entity, T>,
}

impl<T: Component + Clone> Default for AutoUndoStorage<T> {
    fn default() -> Self {
        Self {
            storage: HashMap::new(),
        }
    }
}

pub trait AppAutoUndo {
    fn auto_undo<T: Component + Clone>(&mut self) -> &mut Self;
}

impl AppAutoUndo for App {
    fn auto_undo<T: Component + Clone>(&mut self) -> &mut Self {
        self.world.insert_resource(AutoUndoStorage::<T>::default());

        self.add_systems(
            PostUpdate,
            (
                auto_undo_update_cache::<T>,
                auto_undo_add_init::<T>,
                auto_undo_system_changed::<T>,
                auto_undo_system::<T>,
            )
                .chain(),
        );

        self
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

fn auto_undo_add_init<T: Component + Clone>(
    mut storage: ResMut<AutoUndoStorage<T>>,
    query: Query<(Entity, &T), (With<PrefabMarker>, Added<T>, Without<OneFrameUndoIgnore>)>,
) {
    for (e, data) in query.iter() {
        storage.storage.insert(e, data.clone());
    }
}

fn auto_undo_system_changed<T: Component + Clone>(
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
