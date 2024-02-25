use super::*;

fn configure_app() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins).add_plugins(UndoPlugin);
    app
}

#[test]
fn test_undo() {
    let mut app = configure_app();
    app.auto_undo::<Name>();

    app.update();

    let test_id = app.world.spawn_empty().id();
    app.world.send_event(NewChange {
        change: Arc::new(AddedEntity { entity: test_id }),
    });

    app.update();
    app.update();

    app.world
        .entity_mut(test_id)
        .insert(Name::default())
        .insert(UndoMarker);
    app.world.get_mut::<Name>(test_id).unwrap().set_changed();
    app.update();
    app.update();
    app.update();
    app.update();
    app.update();
    app.update();
    assert!(app.world.get_entity(test_id).is_some());

    app.world.send_event(UndoRedo::Undo);
    app.update();
    app.update();
    app.update();
    app.update();
    app.update();
    assert!(app.world.get::<Name>(test_id).is_none());
    assert!(app.world.get_entity(test_id).is_some());

    app.world.send_event(UndoRedo::Undo);
    app.update();
    app.update();
    assert!(app.world.get_entity(test_id).is_none());

    app.world.send_event(UndoRedo::Redo);
    app.update();
    app.update();

    let mut query = app.world.query_filtered::<(), With<UndoMarker>>();
    assert!(query.iter(&app.world).next().is_some());
}

#[test]
fn test_reflected_undo() {
    let mut app = configure_app();
    app.auto_reflected_undo::<Transform>();

    app.update();

    let test_id = app.world.spawn_empty().id();
    app.world.send_event(NewChange {
        change: Arc::new(AddedEntity { entity: test_id }),
    });

    app.update();
    app.update();

    app.world
        .entity_mut(test_id)
        .insert(Transform::default())
        .insert(UndoMarker);
    app.world
        .get_mut::<Transform>(test_id)
        .unwrap()
        .set_changed();
    app.update();
    app.update();
    app.update();
    app.update();
    app.update();
    app.update();
    assert!(app.world.get_entity(test_id).is_some());

    app.world.send_event(UndoRedo::Undo);
    app.update();
    app.update();
    app.update();
    app.update();
    app.update();
    assert!(app.world.get::<Transform>(test_id).is_none());
    assert!(app.world.get_entity(test_id).is_some());

    app.world.send_event(UndoRedo::Undo);
    app.update();
    app.update();
    assert!(app.world.get_entity(test_id).is_none());
}

#[test]
fn test_reflected_redo() {
    let mut app = configure_app();
    app.auto_reflected_undo::<Transform>();

    app.update();

    let test_id = app.world.spawn_empty().id();
    app.world.send_event(NewChange {
        change: Arc::new(AddedEntity { entity: test_id }),
    });

    app.update();
    app.update();

    app.world
        .entity_mut(test_id)
        .insert(Transform::default())
        .insert(UndoMarker);
    app.world
        .get_mut::<Transform>(test_id)
        .unwrap()
        .set_changed();
    app.update();
    app.update();
    app.update();
    app.update();
    app.update();
    app.update();
    assert!(app.world.get_entity(test_id).is_some());

    app.world.send_event(UndoRedo::Undo);
    app.update();
    app.update();
    assert!(app.world.get_entity(test_id).is_some());
    assert!(app.world.get::<Transform>(test_id).is_none());

    app.world.send_event(UndoRedo::Redo);
    app.update();
    app.update();
    assert!(app.world.get_entity(test_id).is_some());
    assert!(app.world.get::<Transform>(test_id).is_some());
}

#[test]
fn test_undo_with_remap() {
    let mut app = configure_app();
    app.add_plugins(HierarchyPlugin);

    app.auto_reflected_undo::<Parent>();
    app.auto_reflected_undo::<Children>();

    let test_id_1 = app.world.spawn(UndoMarker).id();
    let test_id_2 = app.world.spawn(UndoMarker).id();

    app.world.send_event(NewChange {
        change: Arc::new(AddedEntity { entity: test_id_1 }),
    });
    app.world.send_event(NewChange {
        change: Arc::new(AddedEntity { entity: test_id_2 }),
    });

    app.update();
    app.update();

    app.world.entity_mut(test_id_1).add_child(test_id_2);

    app.update();
    app.update();
    app.cleanup();

    app.world.entity_mut(test_id_1).despawn_recursive();
    app.world.send_event(NewChange {
        change: Arc::new(RemovedEntity { entity: test_id_1 }),
    });

    app.update();
    app.update();

    app.world.send_event(UndoRedo::Undo);

    app.update();
    app.update();
    app.update();

    assert!(app.world.get_entity(test_id_1).is_none());
    assert!(app.world.get_entity(test_id_2).is_none());
    assert_eq!(app.world.entities().len(), 2);

    let mut query = app.world.query::<&Children>();
    assert!(query.get_single(&app.world).is_ok());
}

#[test]
fn clear_one_frame_ignores() {
    let spawn = |mut commands: Commands| {
        commands.spawn(OneFrameUndoIgnore { counter: 0 });
        commands.spawn(OneFrameUndoIgnore { counter: 1 });
        commands.spawn(OneFrameUndoIgnore { counter: 2 });
    };
    let mut app = App::new();
    app.add_systems(Startup, spawn)
        .add_systems(Update, clear_one_frame_ignore);

    app.update();

    let mut query = app.world.query::<(Entity, &OneFrameUndoIgnore)>();

    assert_eq!(query.iter(&app.world).count(), 1);
    assert!(query.iter(&app.world).all(|(_, i)| i.counter == 1));
}

#[test]
fn undo_ignore_ticks() {
    let mut storage = UndoIngnoreStorage::default();
    storage
        .storage
        .insert(Entity::PLACEHOLDER, OneFrameUndoIgnore { counter: 0 });
    storage
        .storage
        .insert(Entity::from_raw(2), OneFrameUndoIgnore { counter: 1 });
    storage
        .storage
        .insert(Entity::from_raw(3), OneFrameUndoIgnore { counter: 2 });

    let mut app = App::new();
    app.insert_resource(storage);
    app.add_systems(Update, undo_ignore_tick);

    app.update();

    let ignore_storage = app.world.resource::<UndoIngnoreStorage>();

    assert_eq!(ignore_storage.storage.len(), 1)
}

#[derive(Component, Default)]
pub struct TestSync;

#[test]
fn test_marker_sync() {
    let mut app = App::default();

    app.add_plugins(MinimalPlugins)
        .add_plugins(SyncUndoMarkersPlugin::<TestSync>::default());

    app.update();

    //Test create UndoMarker after TestSync
    let id1 = app.world.spawn((TestSync,)).id();

    app.update();
    app.update();

    assert!(app.world.get::<UndoMarker>(id1).is_some());

    //Test remove UndoMarker after TestSync
    app.world.entity_mut(id1).remove::<TestSync>();

    app.update();
    app.update();

    assert!(app.world.get::<UndoMarker>(id1).is_none());
}
