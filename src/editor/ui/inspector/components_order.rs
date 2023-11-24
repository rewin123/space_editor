use std::{collections::BTreeMap, cmp::Ordering, any::TypeId};

use bevy::{prelude::*, ecs::component::ComponentId};

// Cannot implement Reflect as BTree doesn't implement Reflect
#[derive(Resource, Default, Clone)]
pub struct ComponentsOrder {
    components: BTreeMap<u8, String>
}

pub trait ComponentsPriority {
    /// registers a component order priority for the inspector
    fn editor_component_priority<T: Component + Default + Send + 'static + Clone>(
        &mut self, priority: u8
    ) -> &mut Self;
}

impl ComponentsPriority for App {
    fn editor_component_priority<T: Component + Default + Send + 'static + Clone>(
        &mut self, priority: u8
    ) -> &mut Self {
        if !self.world.contains_resource::<ComponentsOrder>() {
            self.insert_resource(ComponentsOrder::default());
        }
        let component_name = stringify!("{}", T).to_string();
        let mut order = self.world.resource_mut::<ComponentsOrder>();
        order.components.insert(priority, component_name);

        self
    }
}

pub fn get_priority_sort(world: &World) -> Box<impl FnMut(&(ComponentId, TypeId, String),&(ComponentId, TypeId, String)) -> Ordering>{
    let components = world.resource::<ComponentsOrder>().clone().components;

    Box::new(move |(.., name_a): &(ComponentId, TypeId, String), (.., name_b): &(ComponentId, TypeId, String)| {
        if let Some((_, name)) = components.first_key_value() {
            let first = if name_a == name {
                Ordering::Less
            } else if name_b == name {
                Ordering::Greater
            } else {
                name_a.cmp(name_b)
            };
            let o = components.iter().skip(1).fold(first, |acc, (_, name)| acc.then(
                if name_a == name {
                    Ordering::Less
                } else if name_b == name {
                    Ordering::Greater
                } else {
                    name_a.cmp(name_b)
                }
               ));
               println!("{:?}", o);
               o
        } else {
            name_a.cmp(&name_b)
        }
    })
}
