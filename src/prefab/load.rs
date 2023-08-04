use bevy::prelude::*;


#[derive(Component, Default, Reflect)]
#[reflect(Component)]
pub struct PrefabLoader {
    pub path : String
}

fn load_prefab(
    mut commands : Commands,
    query : Query<(Entity, &PrefabLoader)>,
    asset_server : Res<AssetServer>
) {

}