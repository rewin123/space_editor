use super::*;

#[cfg(test)]
fn configure_app() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins).add_plugins(UndoPlugin);
    app
}

fn repeat_update(app: &mut App, times: usize) {
    for _ in 0..times {
        app.update();
    }
}

#[test]
fn test_undo() {
    let mut app = configure_app();
    app.auto_undo::<Name>();

    app.update();

    let test_id = app.world_mut().spawn_empty().id();
    app.world_mut().send_event(NewChange {
        change: Arc::new(AddedEntity { entity: test_id }),
    });

    app.update();
    app.update();

    app.world_mut()
        .entity_mut(test_id)
        .insert(Name::default())
        .insert(UndoMarker);
    app.world_mut()
        .get_mut::<Name>(test_id)
        .unwrap()
        .set_changed();
    repeat_update(&mut app, 10);
    assert!(app.world_mut().get_entity(test_id).is_some());

    app.world_mut().get_mut::<Name>(test_id).unwrap().set("foo");
    repeat_update(&mut app, 10);
    assert_eq!(
        app.world_mut().get::<Name>(test_id).unwrap().to_string(),
        "foo"
    );

    app.world_mut().send_event(UndoRedo::Undo);
    repeat_update(&mut app, 2);
    assert_eq!(
        app.world_mut().get::<Name>(test_id).unwrap().to_string(),
        ""
    );

    app.world_mut().send_event(UndoRedo::Redo);
    repeat_update(&mut app, 4);
    assert_eq!(
        app.world_mut().get::<Name>(test_id).unwrap().to_string(),
        "foo"
    );
    app.world_mut().send_event(UndoRedo::Undo);
    repeat_update(&mut app, 2);

    app.world_mut().send_event(UndoRedo::Undo);
    repeat_update(&mut app, 2);
    assert!(app.world_mut().get::<Name>(test_id).is_none());
    assert!(app.world_mut().get_entity(test_id).is_some());

    app.world_mut().send_event(UndoRedo::Undo);
    repeat_update(&mut app, 2);
    assert!(app.world_mut().get_entity(test_id).is_none());

    app.world_mut().send_event(UndoRedo::Redo);
    repeat_update(&mut app, 2);

    let mut query = app.world_mut().query_filtered::<(), With<UndoMarker>>();
    assert!(query.iter(&app.world_mut()).next().is_some());
}

#[test]
fn test_reflected_undo() {
    let mut app = configure_app();
    app.auto_reflected_undo::<Transform>();

    app.update();

    let test_id = app.world_mut().spawn_empty().id();
    app.world_mut().send_event(NewChange {
        change: Arc::new(AddedEntity { entity: test_id }),
    });
    repeat_update(&mut app, 2);

    app.world_mut()
        .entity_mut(test_id)
        .insert(Transform::default())
        .insert(UndoMarker);
    app.world_mut()
        .get_mut::<Transform>(test_id)
        .unwrap()
        .set_changed();
    repeat_update(&mut app, 10);
    assert!(app.world_mut().get_entity(test_id).is_some());

    app.world_mut()
        .get_mut::<Transform>(test_id)
        .unwrap()
        .translation = Vec3::X;
    app.world_mut()
        .get_mut::<Transform>(test_id)
        .unwrap()
        .set_changed();
    repeat_update(&mut app, 10);
    assert_eq!(
        app.world_mut()
            .get::<Transform>(test_id)
            .unwrap()
            .translation,
        Vec3::X
    );
    assert_eq!(app.world_mut().resource::<ChangeChain>().changes.len(), 3);

    app.world_mut().send_event(UndoRedo::Undo);
    repeat_update(&mut app, 2);
    assert_eq!(
        app.world_mut()
            .get::<Transform>(test_id)
            .unwrap()
            .translation,
        Vec3::ZERO
    );

    app.world_mut().send_event(UndoRedo::Redo);
    repeat_update(&mut app, 4);
    assert_eq!(
        app.world_mut()
            .get::<Transform>(test_id)
            .unwrap()
            .translation,
        Vec3::X
    );
    app.world_mut().send_event(UndoRedo::Undo);
    repeat_update(&mut app, 2);

    app.world_mut().send_event(UndoRedo::Undo);
    repeat_update(&mut app, 2);
    assert!(app.world_mut().get::<Transform>(test_id).is_none());
    assert!(app.world_mut().get_entity(test_id).is_some());

    app.world_mut().send_event(UndoRedo::Undo);
    app.update();
    app.update();
    assert!(app.world_mut().get_entity(test_id).is_none());
}

#[test]
fn test_reflected_redo() {
    let mut app = configure_app();
    app.auto_reflected_undo::<Transform>();

    app.update();

    let test_id = app.world_mut().spawn_empty().id();
    app.world_mut().send_event(NewChange {
        change: Arc::new(AddedEntity { entity: test_id }),
    });
    repeat_update(&mut app, 2);

    app.world_mut()
        .entity_mut(test_id)
        .insert(Transform::default())
        .insert(UndoMarker);
    app.world_mut()
        .get_mut::<Transform>(test_id)
        .unwrap()
        .set_changed();
    repeat_update(&mut app, 10);
    assert!(app.world_mut().get_entity(test_id).is_some());

    app.world_mut().send_event(UndoRedo::Undo);
    repeat_update(&mut app, 2);
    assert!(app.world_mut().get_entity(test_id).is_some());
    assert!(app.world_mut().get::<Transform>(test_id).is_none());

    app.world_mut().send_event(UndoRedo::Redo);
    repeat_update(&mut app, 10);
    assert!(app.world_mut().get_entity(test_id).is_some());
    assert!(app.world_mut().get::<Transform>(test_id).is_some());

    app.world_mut().entity_mut(test_id).remove::<Transform>();
    repeat_update(&mut app, 10);
    app.world_mut().send_event(UndoRedo::Undo);
    repeat_update(&mut app, 2);
    assert!(app.world_mut().entity(test_id).get::<Transform>().is_some());

    app.world_mut().send_event(UndoRedo::Redo);
    repeat_update(&mut app, 2);
    assert!(app.world_mut().entity(test_id).get::<Transform>().is_none());
}

#[test]
fn test_redo() {
    let mut app = configure_app();
    app.auto_undo::<Name>();
    app.update();

    let test_id = app.world_mut().spawn(Name::default()).id();
    app.world_mut().send_event(NewChange {
        change: Arc::new(AddedEntity { entity: test_id }),
    });
    repeat_update(&mut app, 10);

    app.world_mut()
        .entity_mut(test_id)
        .insert(Name::default())
        .insert(UndoMarker);
    app.world_mut()
        .get_mut::<Name>(test_id)
        .unwrap()
        .set_changed();
    repeat_update(&mut app, 10);
    assert!(app.world_mut().get_entity(test_id).is_some());

    app.world_mut().entity_mut(test_id).remove::<Name>();
    repeat_update(&mut app, 10);
    app.world_mut().send_event(UndoRedo::Undo);
    repeat_update(&mut app, 2);
    assert!(app.world_mut().get::<Name>(test_id).is_some());

    app.world_mut().send_event(UndoRedo::Redo);
    repeat_update(&mut app, 2);
    assert!(app.world_mut().get::<Name>(test_id).is_none());
}

#[test]
fn test_undo_with_remap() {
    let mut app = configure_app();
    app.add_plugins(HierarchyPlugin);

    app.auto_reflected_undo::<Parent>();
    app.auto_reflected_undo::<Children>();

    let test_id_1 = app.world_mut().spawn(UndoMarker).id();
    let test_id_2 = app.world_mut().spawn(UndoMarker).id();

    app.world_mut().send_event(NewChange {
        change: Arc::new(AddedEntity { entity: test_id_1 }),
    });
    app.world_mut().send_event(NewChange {
        change: Arc::new(AddedEntity { entity: test_id_2 }),
    });
    repeat_update(&mut app, 2);
    app.world_mut().entity_mut(test_id_1).add_child(test_id_2);
    repeat_update(&mut app, 2);
    app.cleanup();

    app.world_mut().entity_mut(test_id_1).despawn_recursive();
    app.world_mut().send_event(NewChange {
        change: Arc::new(RemovedEntity { entity: test_id_1 }),
    });
    repeat_update(&mut app, 2);

    app.world_mut().send_event(UndoRedo::Undo);
    repeat_update(&mut app, 2);

    assert!(app.world_mut().get_entity(test_id_1).is_none());
    assert!(app.world_mut().get_entity(test_id_2).is_none());
    assert_eq!(app.world_mut().entities().len(), 2);

    let mut query = app.world_mut().query::<&Children>();
    assert!(query.get_single(&app.world_mut()).is_ok());
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
    repeat_update(&mut app, 1);

    let mut query = app.world_mut().query::<(Entity, &OneFrameUndoIgnore)>();

    assert_eq!(query.iter(&app.world_mut()).count(), 1);
    assert!(query.iter(&app.world_mut()).all(|(_, i)| i.counter == 1));
}

#[test]
fn undo_ignore_ticks() {
    let mut storage = UndoIgnoreStorage::default();
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

    let ignore_storage = app.world_mut().resource::<UndoIgnoreStorage>();

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
    let id1 = app.world_mut().spawn((TestSync,)).id();
    repeat_update(&mut app, 2);

    assert!(app.world_mut().get::<UndoMarker>(id1).is_some());

    //Test remove UndoMarker after TestSync
    app.world_mut().entity_mut(id1).remove::<TestSync>();
    repeat_update(&mut app, 2);

    assert!(app.world_mut().get::<UndoMarker>(id1).is_none());
}
