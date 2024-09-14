use bevy::{asset::LoadState, prelude::*};

pub struct BackgroundTaskStoragePlugin;

impl Plugin for BackgroundTaskStoragePlugin {
    #[cfg(not(tarpaulin_include))]
    fn build(&self, app: &mut App) {
        app.init_resource::<BackgroundTaskStorage>();

        app.add_systems(PostUpdate, update_storage);
    }
}

#[derive(Resource, Default)]
pub struct BackgroundTaskStorage {
    pub tasks: Vec<BackgroundTask>,
}

pub enum BackgroundTask {
    AssetLoading(String, UntypedHandle),
    None,
}

fn update_storage(mut storage: ResMut<BackgroundTaskStorage>, assets: Res<AssetServer>) {
    if !storage.tasks.is_empty() {
        let mut need_remove_task = false;
        match &storage.tasks[0] {
            BackgroundTask::AssetLoading(_path, handle) => {
                let load_state = assets.get_load_state(handle.id());
                if load_state == Some(LoadState::Loaded)
                    || load_state.is_none()
                    || matches!(load_state, Some(LoadState::Failed(_)))
                {
                    need_remove_task = true;
                }
            }
            BackgroundTask::None => {
                need_remove_task = true;
            }
        }

        if need_remove_task {
            storage.tasks.remove(0);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn single_none_bg_task() {
        let storage = BackgroundTaskStorage {
            tasks: vec![BackgroundTask::None],
        };

        let mut app = App::new();
        app.insert_resource(storage)
            .add_plugins((
                MinimalPlugins,
                AssetPlugin::default(),
                ImagePlugin::default(),
            ))
            .add_systems(Update, update_storage);

        assert_eq!(
            app.world().resource::<BackgroundTaskStorage>().tasks.len(),
            1
        );
        app.update();

        assert_eq!(
            app.world().resource::<BackgroundTaskStorage>().tasks.len(),
            0
        );
    }
}
