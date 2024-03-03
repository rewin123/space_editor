use bevy::{
    prelude::{
        Commands, Component, Entity, IntoSystemConfigs, IntoSystemSetConfigs, Plugin, PostUpdate,
        Query, RemovedComponents, Res, SystemSet, Transform, Update, Visibility,
    },
    time::Time,
};

use super::{
    meshing::{ChunkMeshingSet, ChunkMeshingTask},
    Chunk,
};

const ANIMATION_DURATION: f32 = 0.8;
const ANIMATION_HEIGHT: f32 = 128.;

#[derive(Component)]
pub struct ChunkSpawnAnimation {
    start_time: f32,
}

fn attach_chunk_animation(
    mut ready_chunks: Query<(&mut Transform, &mut Visibility, &Chunk)>,
    mut removed_chunk_meshes: RemovedComponents<ChunkMeshingTask>,
    time: Res<Time>,
    mut commands: Commands,
) {
    removed_chunk_meshes.read().for_each(|entity| {
        if ready_chunks.contains(entity) {
            commands.entity(entity).insert(ChunkSpawnAnimation {
                start_time: time.elapsed_seconds(),
            });
            if let Ok((mut transform, mut visibility, chunk)) = ready_chunks.get_mut(entity) {
                *visibility = Visibility::Visible;
                transform.translation.y = chunk.0.y as f32 - ANIMATION_HEIGHT;
            };
        }
    });
}

/// Steps the chunk animation by one frame.
fn step_chunk_animation(
    mut chunks: Query<(Entity, &mut Transform, &Chunk, &ChunkSpawnAnimation)>,
    time: Res<Time>,
    mut commands: Commands,
) {
    chunks.for_each_mut(|(entity, mut transform, _chunk, animation)| {
        let delta = (time.elapsed_seconds() - animation.start_time).min(ANIMATION_DURATION);

        let ytransform = (1. - (1. - (delta / ANIMATION_DURATION)).powi(5))
            .mul_add(ANIMATION_HEIGHT, _chunk.0.y as f32 - ANIMATION_HEIGHT);

        transform.translation.y = ytransform;

        if delta == ANIMATION_DURATION {
            commands.entity(entity).remove::<ChunkSpawnAnimation>();
        }
    });
}

/// Animates the spawning of chunk entities that come into sight.
pub struct ChunkAppearanceAnimatorPlugin;

#[derive(PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Clone, SystemSet)]
pub struct ChunkAppearanceAnimatorSet;

impl Plugin for ChunkAppearanceAnimatorPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.configure_sets(
            PostUpdate,
            ChunkAppearanceAnimatorSet.after(ChunkMeshingSet),
        )
        .add_systems(
            Update,
            (step_chunk_animation, attach_chunk_animation).in_set(ChunkAppearanceAnimatorSet),
        );
    }
}
