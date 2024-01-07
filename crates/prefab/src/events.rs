use bevy::{prelude::*, reflect::GetTypeRegistration};
use std::any::TypeId;

pub(crate) trait BevyEvent:
    Event + Default + Resource + Reflect + Send + Clone + 'static + GetTypeRegistration
{
    type T: Event + Clone;

    fn inner_event(&self) -> &Self::T;

    fn inner_path() -> String {
        std::any::type_name::<Self::T>().to_string()
    }

    fn inner_type_id() -> TypeId {
        TypeId::of::<Self::T>()
    }
}

pub(crate) fn register_events(app: &mut App) -> &mut App {
    // TODO: add all events and relevant types (https://github.com/rewin123/space_editor/issues/159)
    app
}
