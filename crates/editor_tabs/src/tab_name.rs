/// This module contains the tab naming logic

use std::{any::{Any, TypeId}, fmt::Debug};


pub trait TabName : Debug + Any {
    fn clear_background(&self) -> bool;
    fn title(&self) -> String;
}

#[derive(Clone, Hash, PartialEq, Eq, Debug, PartialOrd, Ord)]
pub struct TabNameHolder {
    pub value: String,
    pub type_id: TypeId,
    pub clear_background: bool,
    pub title: String
}

impl TabNameHolder {
    pub fn new<T : TabName>(value : T) -> Self {
        Self {
            value: format!("{:?}", value),
            type_id: TypeId::of::<T>(),
            clear_background: value.clear_background(),
            title: value.title()
        }
    }
}

impl<T : TabName> From<T> for TabNameHolder {
    fn from(value: T) -> Self {
        Self::new(value)
    }
}