#![allow(clippy::type_complexity)]

#[cfg(test)]
mod tests;
// This part of code is used for saving and loading settings and window state
use bevy::{
    prelude::*,
    reflect::{
        serde::{ReflectDeserializer, ReflectSerializer},
        GetTypeRegistration,
    },
    utils::HashMap,
    window::WindowCloseRequested,
};
use ron::ser::PrettyConfig;
use serde::de::DeserializeSeed;

/// Plugin that enables persistence for marked entities
pub struct PersistencePlugin;

#[derive(SystemSet, Hash, PartialEq, Clone, Debug, Eq)]
pub enum PersistenceSet {
    EventReader,
    ResourceProcess,
    Collect,
}

impl Plugin for PersistencePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<PersistenceRegistry>()
            .init_resource::<PersistenceSettings>();

        app.add_event::<PersistenceEvent>();
        app.add_event::<PersistenceResourceBroadcastEvent>();

        app.configure_sets(
            Update,
            (
                PersistenceSet::EventReader,
                PersistenceSet::ResourceProcess,
                PersistenceSet::Collect,
            )
                .chain(),
        );

        app.add_systems(Startup, persistence_startup_load);
        app.add_systems(PreUpdate, persistence_save_on_close);

        app.add_systems(
            Update,
            persistence_start.in_set(PersistenceSet::EventReader),
        );
        app.add_systems(Update, persistence_end.in_set(PersistenceSet::Collect));

        app.persistence_resource::<PersistenceSettings>();
    }
}

fn persistence_save_on_close(
    mut events: EventWriter<PersistenceEvent>,
    settings: Res<PersistenceSettings>,
    mut close_events: EventReader<WindowCloseRequested>,
) {
    if settings.save_on_close && close_events.read().next().is_some() {
        events.send(PersistenceEvent::Save);
    }
}

fn persistence_startup_load(
    mut events: EventWriter<PersistenceEvent>,
    settings: Res<PersistenceSettings>,
) {
    if settings.load_on_startup {
        events.send(PersistenceEvent::Load);
    }
}

fn persistence_start(
    mut events: EventReader<PersistenceEvent>,
    mut broadcast: EventWriter<PersistenceResourceBroadcastEvent>,
    mut persistence: ResMut<PersistenceRegistry>,
) {
    for event in events.read() {
        match event {
            PersistenceEvent::Save => {
                broadcast.send(PersistenceResourceBroadcastEvent::Pack);
                persistence.mode = PersistenceMode::Saving;
                persistence.save_counter = 0;
            }
            PersistenceEvent::Load => {
                match &persistence.source {
                    PersistenceDataSource::File(path) => {
                        let Ok(file) = std::fs::File::open(path) else {
                            warn!("Persistence file not found at path {}", path);
                            continue;
                        };
                        let data: HashMap<String, String> = ron::de::from_reader(file).unwrap();
                        persistence.data = data;
                    }
                    PersistenceDataSource::Memory => {
                        //do nothing
                    }
                }

                broadcast.send(PersistenceResourceBroadcastEvent::Unpack);
                persistence.mode = PersistenceMode::Loading;
                persistence.load_counter = 0;
            }
        }
    }
}

fn persistence_end(mut persistence: ResMut<PersistenceRegistry>) {
    let mode = persistence.mode.clone();
    match mode {
        PersistenceMode::Saving => {
            persistence.mode = PersistenceMode::None;
            if persistence.save_counter != persistence.target_count {
                error!(
                    "Persistence saving error: {} of {} resources were saved",
                    persistence.save_counter, persistence.target_count
                );
            }

            match &persistence.source {
                PersistenceDataSource::File(path) => {
                    let mut file = std::fs::File::create(path).unwrap();
                    ron::ser::to_writer_pretty(
                        &mut file,
                        &persistence.data,
                        PrettyConfig::default(),
                    )
                    .unwrap();
                }
                PersistenceDataSource::Memory => {
                    //do nothing
                }
            }
            {}
        }
        PersistenceMode::Loading => {
            persistence.mode = PersistenceMode::None;
            if persistence.load_counter != persistence.target_count {
                error!(
                    "Persistence loading error: {} of {} resources were loaded",
                    persistence.load_counter, persistence.target_count
                );
            }
        }
        _ => {}
    }
}

#[derive(Resource, Reflect)]
#[reflect(Resource)]
pub struct PersistenceSettings {
    pub load_on_startup: bool,
    pub save_on_close: bool,
}

impl Default for PersistenceSettings {
    fn default() -> Self {
        Self {
            load_on_startup: true,
            save_on_close: true,
        }
    }
}

#[derive(Debug, Default, Clone, PartialEq)]
enum PersistenceMode {
    Saving,
    Loading,
    #[default]
    None,
}

/// ['PersistenceRegistry'] contains lambda functions for loading/unloading editor state
/// At the moment of closing the window or starting the game mode,
/// all necessary data is saved to a file/memory, and then restored when the editor mode is opened.
/// When the restored resource is loaded, the ['PersistenceLoaded<T>'] event is generated
///
/// ['PersistenceLoaded<T>']: crate::editor::core::persistence::PersistenceLoaded
#[derive(Resource, Default)]
pub struct PersistenceRegistry {
    source: PersistenceDataSource,
    data: HashMap<String, String>,
    load_counter: usize,
    save_counter: usize,
    target_count: usize,
    mode: PersistenceMode,
}

#[derive(Event, Default)]
pub struct PersistenceLoaded<T> {
    _phantom: std::marker::PhantomData<T>,
}

#[derive(Event)]
pub enum PersistenceEvent {
    Save,
    Load,
}

#[derive(Event)]
enum PersistenceResourceBroadcastEvent {
    Unpack,
    Pack,
}

#[derive(Reflect, Clone)]
#[reflect(Default)]
pub enum PersistenceDataSource {
    File(String),
    Memory,
}

impl Default for PersistenceDataSource {
    fn default() -> Self {
        Self::File("editor.ron".to_string())
    }
}

#[derive(Resource)]
struct PersistenceLoadPipeline<T> {
    pub load_fn: Box<dyn Fn(&mut T, T) + Send + Sync>,
}

impl<T> Default for PersistenceLoadPipeline<T> {
    fn default() -> Self {
        Self {
            load_fn: Box::new(|dst, src| {
                *dst = src;
            }),
        }
    }
}

pub trait AppPersistenceExt {
    fn persistence_resource<T: Default + Reflect + FromReflect + Resource + GetTypeRegistration>(
        &mut self,
    ) -> &mut Self;

    fn persistence_resource_with_fn<
        T: Default + Reflect + FromReflect + Resource + GetTypeRegistration,
    >(
        &mut self,
        load_function: Box<dyn Fn(&mut T, T) + Send + Sync>,
    ) -> &mut Self;
}

impl AppPersistenceExt for App {
    fn persistence_resource<T: Default + Reflect + FromReflect + Resource + GetTypeRegistration>(
        &mut self,
    ) -> &mut Self {
        self.world_mut()
            .resource_mut::<PersistenceRegistry>()
            .target_count += 1;

        self.register_type::<T>();
        self.add_event::<PersistenceLoaded<T>>();

        self.init_resource::<PersistenceLoadPipeline<T>>();

        self.add_systems(
            Update,
            persistence_resource_system::<T>.in_set(PersistenceSet::ResourceProcess),
        );

        self
    }

    fn persistence_resource_with_fn<
        T: Default + Reflect + FromReflect + Resource + GetTypeRegistration,
    >(
        &mut self,
        load_function: Box<dyn Fn(&mut T, T) + Send + Sync>,
    ) -> &mut Self {
        self.world_mut()
            .resource_mut::<PersistenceRegistry>()
            .target_count += 1;

        self.register_type::<T>();
        self.add_event::<PersistenceLoaded<T>>();

        self.insert_resource(PersistenceLoadPipeline {
            load_fn: load_function,
        });

        self.add_systems(
            Update,
            persistence_resource_system::<T>.in_set(PersistenceSet::ResourceProcess),
        );

        self
    }
}

fn persistence_resource_system<
    T: Default + Reflect + FromReflect + Resource + GetTypeRegistration,
>(
    mut events: EventReader<PersistenceResourceBroadcastEvent>,
    mut persistence: ResMut<PersistenceRegistry>,
    mut resource: ResMut<T>,
    registry: Res<AppTypeRegistry>,
    mut persistence_loaded: EventWriter<PersistenceLoaded<T>>,
    pipeline: ResMut<PersistenceLoadPipeline<T>>,
) {
    for event in events.read() {
        match event {
            PersistenceResourceBroadcastEvent::Pack => {
                let type_registry = registry.read();
                let serializer = ReflectSerializer::new(resource.as_ref(), &type_registry);
                let data = ron::to_string(&serializer).unwrap();
                persistence.data.insert(
                    T::get_type_registration()
                        .type_info()
                        .type_path()
                        .to_string(),
                    data,
                );
                persistence.save_counter += 1;
            }
            PersistenceResourceBroadcastEvent::Unpack => {
                let Some(data) = persistence
                    .data
                    .get(T::get_type_registration().type_info().type_path())
                else {
                    warn!(
                        "Persistence resource {} not found",
                        T::get_type_registration().type_info().type_path()
                    );
                    continue;
                };
                let type_registry = registry.read();
                let deserializer = ReflectDeserializer::new(&type_registry);
                let Ok(reflected_value) =
                    deserializer.deserialize(&mut ron::Deserializer::from_str(data).unwrap())
                else {
                    warn!(
                        "Persistence resource {} could not be deserialized",
                        T::get_type_registration().type_info().type_path()
                    );
                    continue;
                };

                let Some(converted) = <T as FromReflect>::from_reflect(&*reflected_value) else {
                    warn!(
                        "Persistence resource {} could not be converted",
                        T::get_type_registration().type_info().type_path()
                    );
                    continue;
                };
                (pipeline.load_fn)(resource.as_mut(), converted);
                resource.set_changed();

                persistence_loaded.send(PersistenceLoaded::<T>::default());
                persistence.load_counter += 1;
            }
        }
    }
}
