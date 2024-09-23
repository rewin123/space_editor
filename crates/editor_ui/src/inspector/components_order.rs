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
        if !self.world().contains_resource::<ComponentsOrder>() {
            self.insert_resource(ComponentsOrder::default());
        }
        if let Some(mut order) = self.world_mut().get_resource_mut::<ComponentsOrder>() {
            let component_name = pretty_type_name::pretty_type_name::<T>();
            order.components.insert(component_name, priority);
        } else {
            error!("Failed to configure components order");
        }

        self
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn component_priority() {
        let mut app = App::new();

        app.add_plugins(MinimalPlugins)
            .init_resource::<ComponentsOrder>()
            .editor_component_priority::<Name>(0)
            .editor_component_priority::<Transform>(1);

        let order = app.world().resource::<ComponentsOrder>();

        assert_eq!(order.components.get("Name"), Some(&0));
        assert_eq!(order.components.get("Transform"), Some(&1));
        assert_eq!(order.components.get("Visibility"), None);
    }
}
