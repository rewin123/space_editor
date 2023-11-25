// This part of code is used for saving and loading settings and window state
use bevy::{
    prelude::*,
    reflect::{
        serde::{ReflectSerializer, UntypedReflectDeserializer},
        GetTypeRegistration,
    },
    utils::HashMap,
    window::WindowCloseRequested,
};
use ron::ser::PrettyConfig;
use serde::de::DeserializeSeed;

pub struct PersistancePlugin;

#[derive(SystemSet, Hash, PartialEq, Clone, Debug, Eq)]
pub enum PersistanceSet {
    EventReader,
    ResourceProcess,
    Collect,
}

impl Plugin for PersistancePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<PersistanceRegistry>()
            .init_resource::<PersistanceSettings>();

        app.add_event::<PersistanceEvent>();
        app.add_event::<PersistanceResourceBroadcastEvent>();

        app.configure_sets(
            Update,
            (
                PersistanceSet::EventReader,
                PersistanceSet::ResourceProcess,
                PersistanceSet::Collect,
            )
                .chain(),
        );

        app.add_systems(Startup, persistance_startup_load);
        app.add_systems(PreUpdate, persistance_save_on_close);

        app.add_systems(
            Update,
            persistance_start.in_set(PersistanceSet::EventReader),
        );
        app.add_systems(Update, persistance_end.in_set(PersistanceSet::Collect));

        app.persistance_resource::<PersistanceSettings>();
    }
}

fn persistance_save_on_close(
    mut events: EventWriter<PersistanceEvent>,
    settings: Res<PersistanceSettings>,
    mut close_events: EventReader<WindowCloseRequested>,
) {
    if settings.save_on_close && close_events.read().next().is_some() {
        events.send(PersistanceEvent::Save);
    }
}

fn persistance_startup_load(
    mut events: EventWriter<PersistanceEvent>,
    settings: Res<PersistanceSettings>,
) {
    if settings.load_on_startup {
        events.send(PersistanceEvent::Load);
    }
}

fn persistance_start(
    mut events: EventReader<PersistanceEvent>,
    mut broadcast: EventWriter<PersistanceResourceBroadcastEvent>,
    mut persistance: ResMut<PersistanceRegistry>,
) {
    for event in events.read() {
        match event {
            PersistanceEvent::Save => {
                broadcast.send(PersistanceResourceBroadcastEvent::Pack);
                persistance.mode = PersistanceMode::Saving;
                persistance.save_counter = 0;
            }
            PersistanceEvent::Load => {
                match &persistance.source {
                    PersistanceDataSource::File(path) => {
                        let Ok(file) = std::fs::File::open(path) else {
                            warn!("Persistance file not found");
                            continue;
                        };
                        let data: HashMap<String, String> = ron::de::from_reader(file).unwrap();
                        persistance.data = data;
                    }
                    PersistanceDataSource::Memory => {
                        //do nothing
                    }
                }

                broadcast.send(PersistanceResourceBroadcastEvent::Unpack);
                persistance.mode = PersistanceMode::Loading;
                persistance.load_counter = 0;
            }
        }
    }
}

fn persistance_end(mut persistance: ResMut<PersistanceRegistry>) {
    let mode = persistance.mode.clone();
    match mode {
        PersistanceMode::Saving => {
            persistance.mode = PersistanceMode::None;
            if persistance.save_counter != persistance.target_count {
                error!(
                    "Persistance saving error: {} of {} resources were saved",
                    persistance.save_counter, persistance.target_count
                );
            }

            match &persistance.source {
                PersistanceDataSource::File(path) => {
                    let mut file = std::fs::File::create(path).unwrap();
                    ron::ser::to_writer_pretty(
                        &mut file,
                        &persistance.data,
                        PrettyConfig::default(),
                    )
                    .unwrap();
                }
                PersistanceDataSource::Memory => {
                    //do nothing
                }
            }
            {}
        }
        PersistanceMode::Loading => {
            persistance.mode = PersistanceMode::None;
            if persistance.load_counter != persistance.target_count {
                error!(
                    "Persistance loading error: {} of {} resources were loaded",
                    persistance.load_counter, persistance.target_count
                );
            }
        }
        _ => {}
    }
}

#[derive(Resource, Reflect)]
#[reflect(Resource)]
pub struct PersistanceSettings {
    pub load_on_startup: bool,
    pub save_on_close: bool,
}

impl Default for PersistanceSettings {
    fn default() -> Self {
        Self {
            load_on_startup: true,
            save_on_close: true,
        }
    }
}

#[derive(Default, Clone)]
enum PersistanceMode {
    Saving,
    Loading,
    #[default]
    None,
}

/// ['PersistanceRegistry'] contains lambda functions for loading/unloading editor state
/// At the moment of closing the window or starting the game mode,
/// all necessary data is saved to a file/memory, and then restored when the editor mode is opened.
/// When the restored resource is loaded, the ['PersistenceLoaded<T>'] event is generated
///
/// ['PersistenceLoaded<T>']: crate::editor::core::persistance::PersistanceLoaded
#[derive(Resource, Default)]
pub struct PersistanceRegistry {
    source: PersistanceDataSource,
    data: HashMap<String, String>,
    load_counter: usize,
    save_counter: usize,
    target_count: usize,
    mode: PersistanceMode,
}

#[derive(Event, Default)]
pub struct PersistanceLoaded<T> {
    _phantom: std::marker::PhantomData<T>,
}

#[derive(Event)]
pub enum PersistanceEvent {
    Save,
    Load,
}

#[derive(Event)]
enum PersistanceResourceBroadcastEvent {
    Unpack,
    Pack,
}

#[derive(Reflect, Clone)]
#[reflect(Default)]
pub enum PersistanceDataSource {
    File(String),
    Memory,
}

impl Default for PersistanceDataSource {
    fn default() -> Self {
        Self::File("editor.ron".to_string())
    }
}
#[derive(Resource)]
struct PersistanceLoadPipeline<T> {
    pub load_fn: Box<dyn Fn(&mut T, T) + Send + Sync>,
}

impl<T> Default for PersistanceLoadPipeline<T> {
    fn default() -> Self {
        Self {
            load_fn: Box::new(|dst, src| {
                *dst = src;
            }),
        }
    }
}

pub trait AppPersistanceExt {
    fn persistance_resource<T: Default + Reflect + FromReflect + Resource + GetTypeRegistration>(
        &mut self,
    ) -> &mut Self;

    fn persistance_resource_with_fn<
        T: Default + Reflect + FromReflect + Resource + GetTypeRegistration,
    >(
        &mut self,
        load_function: Box<dyn Fn(&mut T, T) + Send + Sync>,
    ) -> &mut Self;
}

impl AppPersistanceExt for App {
    fn persistance_resource<T: Default + Reflect + FromReflect + Resource + GetTypeRegistration>(
        &mut self,
    ) -> &mut Self {
        self.world
            .resource_mut::<PersistanceRegistry>()
            .target_count += 1;

        self.register_type::<T>();
        self.add_event::<PersistanceLoaded<T>>();

        self.init_resource::<PersistanceLoadPipeline<T>>();

        self.add_systems(
            Update,
            persistance_resource_system::<T>.in_set(PersistanceSet::ResourceProcess),
        );

        self
    }

    fn persistance_resource_with_fn<
        T: Default + Reflect + FromReflect + Resource + GetTypeRegistration,
    >(
        &mut self,
        load_function: Box<dyn Fn(&mut T, T) + Send + Sync>,
    ) -> &mut Self {
        self.world
            .resource_mut::<PersistanceRegistry>()
            .target_count += 1;

        self.register_type::<T>();
        self.add_event::<PersistanceLoaded<T>>();

        self.insert_resource(PersistanceLoadPipeline {
            load_fn: load_function,
        });

        self.add_systems(
            Update,
            persistance_resource_system::<T>.in_set(PersistanceSet::ResourceProcess),
        );

        self
    }
}

fn persistance_resource_system<
    T: Default + Reflect + FromReflect + Resource + GetTypeRegistration,
>(
    mut events: EventReader<PersistanceResourceBroadcastEvent>,
    mut persistance: ResMut<PersistanceRegistry>,
    mut resource: ResMut<T>,
    registry: Res<AppTypeRegistry>,
    mut persistance_loaded: EventWriter<PersistanceLoaded<T>>,
    pipeline: ResMut<PersistanceLoadPipeline<T>>,
) {
    for event in events.read() {
        match event {
            PersistanceResourceBroadcastEvent::Pack => {
                let type_registry = registry.read();
                let serializer = ReflectSerializer::new(resource.as_ref(), &type_registry);
                let data = ron::to_string(&serializer).unwrap();
                persistance.data.insert(
                    T::get_type_registration()
                        .type_info()
                        .type_path()
                        .to_string(),
                    data,
                );
                persistance.save_counter += 1;
            }
            PersistanceResourceBroadcastEvent::Unpack => {
                let Some(data) = persistance
                    .data
                    .get(T::get_type_registration().type_info().type_path())
                else {
                    warn!(
                        "Persistance resource {} not found",
                        T::get_type_registration().type_info().type_path()
                    );
                    continue;
                };
                let type_registry = registry.read();
                let deserializer = UntypedReflectDeserializer::new(&type_registry);
                let reflected_value = deserializer
                    .deserialize(&mut ron::Deserializer::from_str(data).unwrap())
                    .unwrap();

                let Some(converted) = <T as FromReflect>::from_reflect(&*reflected_value) else {
                    warn!(
                        "Persistance resource {} could not be converted",
                        T::get_type_registration().type_info().type_path()
                    );
                    continue;
                };
                (pipeline.load_fn)(resource.as_mut(), converted);
                resource.set_changed();

                persistance_loaded.send(PersistanceLoaded::<T>::default());
                persistance.load_counter += 1;
            }
        }
    }
}
