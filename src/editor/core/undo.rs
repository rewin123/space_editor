use std::sync::Arc;

use bevy::prelude::*;

pub struct UndoPlugin;

impl Plugin for UndoPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ChangeChain>();

        app.add_event::<NewChange>();

        app.add_systems(
            Update,
            (update_change_chain, undo_redo_logic)
        );
    }
}

fn update_change_chain(
    mut change_chain: ResMut<ChangeChain>,
    mut events: EventReader<NewChange>
) {
    for event in events.read() {
        change_chain.changes.push(event.change.clone());
        change_chain.changes_for_redo.clear();
    }
}

fn undo_redo_logic(
    world : &mut World
) {
    world.resource_scope::<Events<UndoRedo>, _>(|world, events| {
        world.resource_scope::<ChangeChain, _>(|world, mut change_chain| {
            let mut reader = events.get_reader();
            for event in reader.read(&events) {
                match event {
                    UndoRedo::Undo => {
                        if let Some(change) = change_chain.changes.pop() {
                            change.revert(world).unwrap();
                            change_chain.changes_for_redo.push(change);
                        }
                    },
                    UndoRedo::Redo => {
                        if let Some(redo_change) = change_chain.changes_for_redo.pop() {
                            redo_change.apply(world).unwrap();
                            change_chain.changes.push(redo_change);
                        }
                    },
                }
            }
        });
    });
}

#[derive(Resource, Default)]
pub struct ChangeChain {
    pub changes: Vec<Arc<dyn EditorChange + Send + Sync>>,
    pub changes_for_redo : Vec<Arc<dyn EditorChange + Send + Sync>>,
}

pub trait EditorChange {
    fn revert(&self, world : &mut World) -> Result<(), String>;
    fn apply(&self, world : &mut World) -> Result<(), String>;
}

#[derive(Event)]
pub enum UndoRedo {
    Undo,
    Redo
}

#[derive(Event)]
pub struct NewChange {
    pub change: Arc<dyn EditorChange + Send + Sync>
}

pub struct ComponentChange<T : Component + Clone> {
    old_value: T,
    new_value: T,
    entity: Entity
}

impl<T : Component + Clone> EditorChange for ComponentChange<T> {
    fn revert(&self, world : &mut World) -> Result<(), String> {
        world.entity_mut(self.entity).insert(self.old_value.clone());
        Ok(())
    }

    fn apply(&self, world : &mut World) -> Result<(), String> {
        world.entity_mut(self.entity).insert(self.new_value.clone());
        Ok(())
    }
} 

pub struct NewEntityChange {
    entity: Entity
}

impl EditorChange for NewEntityChange {
    fn revert(&self, world : &mut World) -> Result<(), String> {
        world.entity_mut(self.entity).despawn();
        Ok(())
    }

    fn apply(&self, world : &mut World) -> Result<(), String> {
        // world.get_or_spawn(entity)
        Ok(())
    }
}