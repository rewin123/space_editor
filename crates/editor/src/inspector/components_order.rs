use bevy::{prelude::*, utils::HashMap};

// Cannot implement Reflect as BTree doesn't implement Reflect
#[derive(Reflect, Resource, Default, Clone)]
#[reflect(Resource)]
pub struct ComponentsOrder {
    pub components: HashMap<String, u8>,
}

pub trait ComponentsPriority {
    /// registers a component order priority for the inspector
    fn editor_component_priority<T: Component + Default + Send + 'static + Clone>(
        &mut self,
        priority: u8,
    ) -> &mut Self;
}

impl ComponentsPriority for App {
    fn editor_component_priority<T: Component + Default + Send + 'static + Clone>(
        &mut self,
        priority: u8,
    ) -> &mut Self {
        if !self.world.contains_resource::<ComponentsOrder>() {
            self.insert_resource(ComponentsOrder::default());
        }
        let component_name = pretty_type_name::pretty_type_name::<T>();
        let mut order = self.world.resource_mut::<ComponentsOrder>();
        order.components.insert(component_name, priority);

        self
    }
}
