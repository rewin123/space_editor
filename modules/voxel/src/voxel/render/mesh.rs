use std::marker::PhantomData;

use crate::voxel::{storage::VoxelBuffer, MaterialVoxel};
use bevy::{
    prelude::Mesh,
    render::mesh::{Indices, VertexAttributeValues},
};
use block_mesh::{greedy_quads, GreedyQuadsBuffer, RIGHT_HANDED_Y_UP_CONFIG};
use ndcopy::copy3;
use ndshape::{RuntimeShape, Shape};

use super::VoxelTerrainMesh;

/// Intermediate buffers for greedy meshing of voxel data which are reusable between frames to not allocate.
pub struct MeshBuffers<T, S: Shape<3, Coord = u32>>
where
    T: Copy + Default + MaterialVoxel,
{
    // A padded buffer to run greedy meshing algorithm on
    scratch_buffer: VoxelBuffer<T, RuntimeShape<u32, 3>>,
    greedy_buffer: GreedyQuadsBuffer,
    _phantom: PhantomData<S>,
}

impl<T, S: Shape<3, Coord = u32>> MeshBuffers<T, S>
where
    T: Copy + Default + MaterialVoxel,
{
    pub fn new(shape: S) -> Self {
        let padded_shape = RuntimeShape::<u32, 3>::new(shape.as_array().map(|x| x + 2));

        Self {
            greedy_buffer: GreedyQuadsBuffer::new(padded_shape.size() as usize),
            scratch_buffer: VoxelBuffer::<T, RuntimeShape<u32, 3>>::new_empty(padded_shape),
            _phantom: Default::default(),
        }
    }
}

// Processes the voxel data buffer specified as a parameter and generate.
//todo: don't populate mesh directly, introduce a meshbuilding system.
pub fn mesh_buffer<T, S>(
    buffer: &VoxelBuffer<T, S>,
    mesh_buffers: &mut MeshBuffers<T, S>,
    render_mesh: &mut Mesh,
    scale: f32,
) where
    T: Copy + Default + MaterialVoxel,
    S: Shape<3, Coord = u32>,
{
    mesh_buffers
        .greedy_buffer
        .reset(buffer.shape().size() as usize);

    let dst_shape = mesh_buffers.scratch_buffer.shape().clone();

    copy3(
        buffer.shape().as_array(),
        buffer.slice(),
        buffer.shape(),
        [0; 3],
        mesh_buffers.scratch_buffer.slice_mut(),
        &dst_shape,
        [1; 3],
    );

    greedy_quads(
        mesh_buffers.scratch_buffer.slice(),
        mesh_buffers.scratch_buffer.shape(),
        [0; 3],
        mesh_buffers
            .scratch_buffer
            .shape()
            .as_array()
            .map(|axis| axis - 1),
        &RIGHT_HANDED_Y_UP_CONFIG.faces,
        &mut mesh_buffers.greedy_buffer,
    );

    let num_indices = mesh_buffers.greedy_buffer.quads.num_quads() * 6;
    let num_vertices = mesh_buffers.greedy_buffer.quads.num_quads() * 4;
    let mut indices = Vec::with_capacity(num_indices);
    let mut positions = Vec::with_capacity(num_vertices);
    let mut data = Vec::with_capacity(num_vertices);

    //normal face index depends on the quad orientation config
    for (block_face_normal_index, (group, face)) in mesh_buffers
        .greedy_buffer
        .quads
        .groups
        .as_ref()
        .iter()
        .zip(RIGHT_HANDED_Y_UP_CONFIG.faces.iter())
        .enumerate()
    {
        for quad in group.iter() {
            indices.extend_from_slice(&face.quad_mesh_indices(positions.len() as u32));
            positions.extend_from_slice(&face.quad_mesh_positions(quad, scale));
            data.extend_from_slice(
                &[(block_face_normal_index as u32) << 8u32
                    | buffer
                        .voxel_at(quad.minimum.map(|x| x - 1).into())
                        .as_mat_id() as u32; 4],
            );
        }
    }

    render_mesh.insert_attribute(
        Mesh::ATTRIBUTE_POSITION,
        VertexAttributeValues::Float32x3(positions),
    );

    //todo: in the future we might want to encode all the information onto a single uint32
    render_mesh.insert_attribute(
        VoxelTerrainMesh::ATTRIBUTE_DATA,
        VertexAttributeValues::Uint32(data),
    );

    render_mesh.set_indices(Some(Indices::U32(indices.clone())));
}
