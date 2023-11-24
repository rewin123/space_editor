use bevy::{asset::LoadState, prelude::*};

pub struct BackgroundTaskStoragePlugin;

impl Plugin for BackgroundTaskStoragePlugin {
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
                    || load_state == Some(LoadState::Failed)
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
