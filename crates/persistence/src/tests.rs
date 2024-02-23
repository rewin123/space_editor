use super::*;

#[test]
fn save_on_close_triggers_event() {
    let mut app = App::new();
    app.init_resource::<PersistenceSettings>()
        .add_event::<PersistenceEvent>()
        .add_event::<WindowCloseRequested>()
        .add_systems(PreUpdate, persistence_save_on_close);
    app.world.send_event(WindowCloseRequested {
        window: Entity::PLACEHOLDER,
    });
    app.update();

    let event = app.world.get_resource::<Events<PersistenceEvent>>();

    assert!(event.is_some());
    assert_eq!(event.unwrap().len(), 1);
}

#[test]
fn save_on_close_false_doesnt_triggers_event() {
    let mut app = App::new();
    app.insert_resource(PersistenceSettings {
        load_on_startup: true,
        save_on_close: false,
    })
    .add_event::<PersistenceEvent>()
    .add_event::<WindowCloseRequested>()
    .add_systems(PreUpdate, persistence_save_on_close);
    app.world.send_event(WindowCloseRequested {
        window: Entity::PLACEHOLDER,
    });
    app.update();

    let event = app.world.get_resource::<Events<PersistenceEvent>>();

    assert!(event.is_some());
    assert_eq!(event.unwrap().len(), 0);
}

#[test]
fn load_on_startup_triggers_event() {
    let mut app = App::new();
    app.init_resource::<PersistenceSettings>()
        .add_event::<PersistenceEvent>()
        .add_systems(Update, persistence_startup_load);
    app.update();

    let events = app.world.get_resource::<Events<PersistenceEvent>>();
    assert!(events.is_some());
    assert_eq!(events.unwrap().len(), 1);
}

#[test]
fn not_load_on_startup_triggers_event() {
    let mut app = App::new();
    app.insert_resource(PersistenceSettings {
        load_on_startup: false,
        ..Default::default()
    })
    .add_event::<PersistenceEvent>()
    .add_systems(Update, persistence_startup_load);
    app.update();

    let events = app.world.get_resource::<Events<PersistenceEvent>>();
    assert!(events.is_some());
    assert_eq!(events.unwrap().len(), 0);
}

#[test]
fn persistence_starts_on_save() {
    let mut app = App::new();
    app.init_resource::<PersistenceRegistry>()
        .add_event::<PersistenceEvent>()
        .add_event::<PersistenceResourceBroadcastEvent>()
        .add_systems(PreUpdate, persistence_start);

    app.world.send_event(PersistenceEvent::Save);
    app.update();

    let event = app
        .world
        .get_resource::<Events<PersistenceResourceBroadcastEvent>>();

    assert!(event.is_some());
    assert_eq!(event.unwrap().len(), 1);

    let persistence = app.world.get_resource::<PersistenceRegistry>();

    assert!(persistence.is_some());
    let reg = persistence.unwrap();
    assert_eq!(reg.mode, PersistenceMode::Saving);
    assert_eq!(reg.save_counter, 0);
}

#[test]
fn persistence_starts_on_load_mem() {
    let mut app = App::new();
    app.insert_resource(PersistenceRegistry {
        source: PersistenceDataSource::Memory,
        ..Default::default()
    })
    .add_event::<PersistenceEvent>()
    .add_event::<PersistenceResourceBroadcastEvent>()
    .add_systems(PreUpdate, persistence_start);

    app.world.send_event(PersistenceEvent::Load);
    app.update();

    let event = app
        .world
        .get_resource::<Events<PersistenceResourceBroadcastEvent>>();

    assert!(event.is_some());
    assert_eq!(event.unwrap().len(), 1);

    let persistence = app.world.get_resource::<PersistenceRegistry>();

    assert!(persistence.is_some());
    let reg = persistence.unwrap();
    assert_eq!(reg.mode, PersistenceMode::Loading);
    assert_eq!(reg.load_counter, 0);
}

#[test]
fn persistence_starts_on_load_file() {
    let mut app = App::new();
    app.insert_resource(PersistenceRegistry {
        source: PersistenceDataSource::File(String::from("../../test_data/test_editor.ron")),
        ..Default::default()
    })
    .add_event::<PersistenceEvent>()
    .add_event::<PersistenceResourceBroadcastEvent>()
    .add_systems(PreUpdate, persistence_start);

    app.world.send_event(PersistenceEvent::Load);
    app.update();

    let event = app
        .world
        .get_resource::<Events<PersistenceResourceBroadcastEvent>>();

    assert!(event.is_some());
    assert_eq!(event.unwrap().len(), 1);

    let persistence = app.world.get_resource::<PersistenceRegistry>();

    assert!(persistence.is_some());
    let reg = persistence.unwrap();
    assert_eq!(reg.mode, PersistenceMode::Loading);
    assert_eq!(reg.load_counter, 0);
    assert!(reg
        .data
        .contains_key("space_persistence::PersistenceSettings"));
}
