use std::cell::RefCell;

use super::{
    chunks::{ChunkEntities, ChunkLoadingSet, DirtyChunks},
    terrain::TerrainGenSet,
    Chunk, ChunkShape, Voxel, CHUNK_LENGTH,
};
use crate::voxel::{
    render::{mesh_buffer, ChunkMaterialSingleton, MeshBuffers},
    storage::ChunkMap,
};
use bevy::{
    pbr::NotShadowCaster,
    prelude::*,
    render::{primitives::Aabb, render_resource::PrimitiveTopology},
    tasks::{AsyncComputeTaskPool, Task},
};
use futures_lite::future;
use once_cell::sync::Lazy;
use thread_local::ThreadLocal;

/// Attaches to the newly inserted chunk entities components required for rendering.
pub fn prepare_chunks(
    chunks: Query<(Entity, &Chunk), Added<Chunk>>,
    mut meshes: ResMut<Assets<Mesh>>,
    material: Res<ChunkMaterialSingleton>,
    mut cmds: Commands,
) {
    for (chunk, chunk_key) in chunks.iter() {
        let mut entity_commands = cmds.entity(chunk);
        entity_commands.insert((
            MaterialMeshBundle {
                material: (**material).clone(),
                mesh: meshes.add(Mesh::new(PrimitiveTopology::TriangleList)),
                transform: Transform::from_translation(chunk_key.0.as_vec3()),
                visibility: Visibility::Hidden,
                ..Default::default()
            },
            Aabb::from_min_max(Vec3::ZERO, Vec3::splat(CHUNK_LENGTH as f32)),
        ));
        // There is no need to cast shadows for chunks below the surface.
        if chunk_key.0.y <= 64 {
            entity_commands.insert(NotShadowCaster);
        }
    }
}

// a pool of mesh buffers shared between meshing tasks.
static SHARED_MESH_BUFFERS: Lazy<ThreadLocal<RefCell<MeshBuffers<Voxel, ChunkShape>>>> =
    Lazy::new(ThreadLocal::default);

/// Queues meshing tasks for the chunks in need of a remesh.
fn queue_mesh_tasks(
    mut commands: Commands,
    dirty_chunks: Res<DirtyChunks>,
    chunk_entities: Res<ChunkEntities>,
    chunks: Res<ChunkMap<Voxel, ChunkShape>>,
) {
    let task_pool = AsyncComputeTaskPool::get();

    dirty_chunks
        .iter_dirty()
        .filter_map(|key| chunk_entities.entity(*key).map(|entity| (key, entity)))
        .filter_map(|(key, entity)| {
            chunks
                .buffer_at(*key)
                .map(|buffer| (buffer.clone(), entity))
        })
        .map(|(buffer, entity)| {
            (
                entity,
                ChunkMeshingTask(task_pool.spawn(async move {
                    let mut mesh_buffers = SHARED_MESH_BUFFERS
                        .get_or(|| {
                            RefCell::new(MeshBuffers::<Voxel, ChunkShape>::new(ChunkShape {}))
                        })
                        .borrow_mut();

                    let mut mesh = Mesh::new(PrimitiveTopology::TriangleList);
                    mesh_buffer(&buffer, &mut mesh_buffers, &mut mesh, 1.0);

                    mesh
                })),
            )
        })
        .for_each(|(entity, task)| {
            commands.entity(entity).insert(task);
        });
}

/// Polls and process the generated chunk meshes
fn process_mesh_tasks(
    mut meshes: ResMut<Assets<Mesh>>,
    mut chunk_query: Query<(Entity, &Handle<Mesh>, &mut ChunkMeshingTask), With<Chunk>>,
    mut commands: Commands,
) {
    chunk_query.for_each_mut(|(entity, handle, mut mesh_task)| {
        if let Some(mesh) = future::block_on(future::poll_once(&mut mesh_task.0)) {
            *meshes.get_mut(handle).unwrap() = mesh;
            commands.entity(entity).remove::<ChunkMeshingTask>();
        }
    });
}

/// The set of systems which asynchronusly mesh the chunks.
#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Debug, Hash, SystemSet)]
pub struct ChunkMeshingSet;

/// Handles the meshing of the chunks.
pub struct VoxelWorldMeshingPlugin;

impl Plugin for VoxelWorldMeshingPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.configure_sets(
            Update,
            ChunkMeshingSet.after(TerrainGenSet).after(ChunkLoadingSet),
        )
        .add_systems(
            Update,
            (prepare_chunks, queue_mesh_tasks, process_mesh_tasks)
                .chain()
                .in_set(ChunkMeshingSet),
        );
    }
}

#[derive(Component)]
pub struct ChunkMeshingTask(Task<Mesh>);
