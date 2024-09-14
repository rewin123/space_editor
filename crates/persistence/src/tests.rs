use super::*;

#[test]
fn save_on_close_triggers_event() {
    let mut app = App::new();
    app.init_resource::<PersistenceSettings>()
        .add_event::<PersistenceEvent>()
        .add_event::<WindowCloseRequested>()
        .add_systems(PreUpdate, persistence_save_on_close);
    app.world_mut().send_event(WindowCloseRequested {
        window: Entity::PLACEHOLDER,
    });
    app.update();

    let event = app.world_mut().get_resource::<Events<PersistenceEvent>>();

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
    app.world_mut().send_event(WindowCloseRequested {
        window: Entity::PLACEHOLDER,
    });
    app.update();

    let event = app.world_mut().get_resource::<Events<PersistenceEvent>>();

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

    let events = app.world_mut().get_resource::<Events<PersistenceEvent>>();
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

    let events = app.world_mut().get_resource::<Events<PersistenceEvent>>();
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

    app.world_mut().send_event(PersistenceEvent::Save);
    app.update();

    let event = app
        .world_mut()
        .get_resource::<Events<PersistenceResourceBroadcastEvent>>();

    assert!(event.is_some());
    assert_eq!(event.unwrap().len(), 1);

    let persistence = app.world_mut().get_resource::<PersistenceRegistry>();

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

    app.world_mut().send_event(PersistenceEvent::Load);
    app.update();

    let event = app
        .world_mut()
        .get_resource::<Events<PersistenceResourceBroadcastEvent>>();

    assert!(event.is_some());
    assert_eq!(event.unwrap().len(), 1);

    let persistence = app.world_mut().get_resource::<PersistenceRegistry>();

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

    app.world_mut().send_event(PersistenceEvent::Load);
    app.update();

    let event = app
        .world_mut()
        .get_resource::<Events<PersistenceResourceBroadcastEvent>>();

    assert!(event.is_some());
    assert_eq!(event.unwrap().len(), 1);

    let persistence = app.world_mut().get_resource::<PersistenceRegistry>();

    assert!(persistence.is_some());
    let reg = persistence.unwrap();
    assert_eq!(reg.mode, PersistenceMode::Loading);
    assert_eq!(reg.load_counter, 0);
    assert!(reg
        .data
        .contains_key("space_persistence::PersistenceSettings"));
}

#[test]
fn persistence_starts_on_file_not_found() {
    let mut app = App::new();
    app.insert_resource(PersistenceRegistry {
        source: PersistenceDataSource::File(String::from("../../test_data/fake_editor.ron")),
        ..Default::default()
    })
    .add_event::<PersistenceEvent>()
    .add_event::<PersistenceResourceBroadcastEvent>()
    .add_systems(PreUpdate, persistence_start);

    app.world_mut().send_event(PersistenceEvent::Load);
    app.update();

    let event = app
        .world_mut()
        .get_resource::<Events<PersistenceResourceBroadcastEvent>>();

    assert_eq!(event.unwrap().len(), 0);

    let persistence = app.world_mut().get_resource::<PersistenceRegistry>();
    let reg = persistence.unwrap();
    assert_eq!(reg.mode, PersistenceMode::None);
    assert!(reg.data.is_empty());
}

#[test]
fn persistence_starts_on_load_from_memory() {
    let mut app = App::new();
    app.insert_resource(PersistenceRegistry {
        source: PersistenceDataSource::Memory,
        ..Default::default()
    })
    .add_event::<PersistenceEvent>()
    .add_event::<PersistenceResourceBroadcastEvent>()
    .add_systems(PreUpdate, persistence_start);

    app.world_mut().send_event(PersistenceEvent::Load);
    app.update();

    let event = app
        .world_mut()
        .get_resource::<Events<PersistenceResourceBroadcastEvent>>();

    assert!(event.is_some());
    assert_eq!(event.unwrap().len(), 1);

    let persistence = app.world_mut().get_resource::<PersistenceRegistry>();

    assert!(persistence.is_some());
    let reg = persistence.unwrap();
    assert_eq!(reg.mode, PersistenceMode::Loading);
    assert_eq!(reg.load_counter, 0);
    assert!(reg.data.is_empty());
}

#[test]
fn persistence_load_pipeline_default() {
    let default = PersistenceLoadPipeline::<u8>::default();
    let mut dest = 0;

    (default.load_fn)(&mut dest, 12);

    assert_eq!(dest, 12);
}

#[test]
fn persistence_end_from_loading() {
    let mut app = App::new();
    app.insert_resource(PersistenceRegistry {
        mode: PersistenceMode::Loading,
        ..Default::default()
    })
    .add_systems(PreUpdate, persistence_end);

    app.update();

    let persistence = app.world_mut().resource::<PersistenceRegistry>();
    assert_eq!(persistence.mode, PersistenceMode::None);
}

#[test]
fn persistence_end_from_saving_mem() {
    let mut app = App::new();
    app.insert_resource(PersistenceRegistry {
        mode: PersistenceMode::Saving,
        source: PersistenceDataSource::Memory,
        ..Default::default()
    })
    .add_systems(PreUpdate, persistence_end);

    app.update();

    let persistence = app.world_mut().resource::<PersistenceRegistry>();
    assert_eq!(persistence.mode, PersistenceMode::None);
}

#[test]
fn persistence_end_from_saving_file() {
    let mut app = App::new();
    app.insert_resource(PersistenceRegistry {
        mode: PersistenceMode::Saving,
        source: PersistenceDataSource::File("../../target/fake_editor.ron".to_string()),
        data: HashMap::from([("test".to_string(), "hello world".to_string())]),
        ..Default::default()
    })
    .add_systems(PreUpdate, persistence_end);

    app.update();

    let persistence = app.world_mut().resource::<PersistenceRegistry>();
    assert_eq!(persistence.mode, PersistenceMode::None);

    assert!(std::fs::metadata("../../target/fake_editor.ron").is_ok());
}

#[test]
fn persistence_system_unpack() {
    let mut app = App::new();
    app.insert_resource(PersistenceRegistry {
        mode: PersistenceMode::Loading,
        source: PersistenceDataSource::Memory,
        data: HashMap::from([("space_persistence::PersistenceSettings".to_string(), "{\"space_persistence::PersistenceSettings\":(load_on_startup:true,save_on_close:false)}".to_string())]),
        ..Default::default()
    })
    .init_resource::<PersistenceSettings>()
    .add_event::<PersistenceEvent>()
    .add_event::<PersistenceResourceBroadcastEvent>();
    app.configure_sets(
        Update,
        (
            PersistenceSet::EventReader,
            PersistenceSet::ResourceProcess,
            PersistenceSet::Collect,
        )
            .chain(),
    );
    app.persistence_resource::<PersistenceSettings>();
    app.update();
    app.world_mut()
        .send_event(PersistenceResourceBroadcastEvent::Unpack);
    app.update();

    let settings = app.world().resource::<PersistenceSettings>();
    let count = app.world().resource::<PersistenceRegistry>();

    assert_eq!(count.target_count, 1);
    assert_eq!(count.load_counter, 1);
    assert_eq!(count.save_counter, 0);
    assert!(!settings.save_on_close);
    assert!(settings.load_on_startup);
}

#[test]
fn persistence_system_pack() {
    let mut app = App::new();
    app.insert_resource(PersistenceRegistry {
        mode: PersistenceMode::Saving,
        source: PersistenceDataSource::File("../../target/persistence_test.ron".to_string()),
        data: HashMap::from([("space_persistence::PersistenceSettings".to_string(), "{\"space_persistence::PersistenceSettings\":(load_on_startup:true,save_on_close:false)}".to_string())]),
        ..Default::default()
    })
    .init_resource::<PersistenceSettings>()
    .add_event::<PersistenceEvent>()
    .add_event::<PersistenceResourceBroadcastEvent>();
    app.configure_sets(
        Update,
        (
            PersistenceSet::EventReader,
            PersistenceSet::ResourceProcess,
            PersistenceSet::Collect,
        )
            .chain(),
    );
    app.persistence_resource::<PersistenceSettings>();
    app.update();
    app.world_mut()
        .send_event(PersistenceResourceBroadcastEvent::Pack);
    app.update();

    let reg = app.world_mut().resource::<PersistenceRegistry>();
    assert_eq!(reg.save_counter, 1)
}
