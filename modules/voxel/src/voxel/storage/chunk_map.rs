use ilattice::{morton::Morton3i32, vector::Map as VecMap};
use std::{collections::BTreeMap, hash::Hash};

use bevy::{math::IVec3, prelude::Resource};
use ndshape::Shape;

use crate::voxel::CHUNK_LENGTH;

use super::buffer::VoxelBuffer;

/// Provides an interface to query or modify voxel data for worlds or scenes split into multiple voxel data buffers of a same shape with no level of detail.
#[derive(Resource)]
pub struct ChunkMap<V, S>
where
    V: Clone + Copy + Default + PartialEq + Eq + Hash,
    S: Shape<3, Coord = u32> + Clone,
{
    chunks: BTreeMap<Morton3i32, VoxelBuffer<V, S>>,
    shape_mask: IVec3,
    shape: S,
}

#[allow(dead_code)]
impl<V, S> ChunkMap<V, S>
where
    V: Clone + Copy + Default + PartialEq + Eq + Hash,
    S: Shape<3, Coord = u32> + Clone,
{
    pub fn new(chunk_shape: S) -> Self {
        Self {
            chunks: Default::default(),
            shape_mask: !(IVec3::from(chunk_shape.as_array().map(|x| x as i32)) - IVec3::ONE),
            shape: chunk_shape,
        }
    }

    pub fn voxel_at(&self, pos: IVec3) -> Option<V> {
        let chunk_minimum = pos & self.shape_mask;
        let local_minimum = ilattice::glam::IVec3::from(pos.to_array())
            .map(|x| x.rem_euclid(CHUNK_LENGTH as i32))
            .as_uvec3();

        self.buffer_at(chunk_minimum)
            .map(|buffer| buffer.voxel_at(local_minimum))
    }

    pub fn voxel_at_mut(&mut self, pos: IVec3) -> Option<&mut V> {
        let chunk_minimum = pos & self.shape_mask;
        let local_minimum = ilattice::glam::IVec3::from(pos.to_array())
            .map(|x| x.rem_euclid(CHUNK_LENGTH as i32))
            .as_uvec3();

        self.buffer_at_mut(chunk_minimum)
            .map(|buffer| buffer.voxel_at_mut(local_minimum))
    }

    /// Checks whether there's a buffer at the specified minimum.
    #[inline]
    pub fn exists(&self, minimum: IVec3) -> bool {
        let minimum = ilattice::glam::IVec3::from(minimum.to_array());
        self.chunks.contains_key(&minimum.into())
    }

    /// Returns a reference to the [`VoxelBuffer<V, S>`] at the specified minimum if there's one.
    #[inline]
    pub fn buffer_at(&self, minimum: IVec3) -> Option<&VoxelBuffer<V, S>> {
        let minimum = ilattice::glam::IVec3::from(minimum.to_array());
        self.chunks.get(&minimum.into())
    }

    /// Returns a mutable reference to the [`VoxelBuffer<V, S>`] at the specified minimum if there's one.
    #[inline]
    pub fn buffer_at_mut(&mut self, minimum: IVec3) -> Option<&mut VoxelBuffer<V, S>> {
        let minimum = ilattice::glam::IVec3::from(minimum.to_array());
        self.chunks.get_mut(&minimum.into())
    }

    /// Inserts a new buffer at the specified minimum.
    pub fn insert(&mut self, minimum: IVec3, buffer: VoxelBuffer<V, S>) {
        let minimum = ilattice::glam::IVec3::from(minimum.to_array());

        assert!(buffer.shape().as_array() == self.shape.as_array());
        self.chunks.insert(minimum.into(), buffer);
    }

    /// Inserts a new buffer inititalized with the default value of [`V`] at the specified minimum.
    pub fn insert_empty(&mut self, minimum: IVec3) {
        let minimum = ilattice::glam::IVec3::from(minimum.to_array());
        self.chunks.insert(
            minimum.into(),
            VoxelBuffer::<V, S>::new_empty(self.shape.clone()),
        );
    }

    /// Inserts buffers from an iterator passed as a parameter
    pub fn insert_batch<T: IntoIterator<Item = (Morton3i32, VoxelBuffer<V, S>)>>(
        &mut self,
        iter: T,
    ) {
        self.chunks.extend(iter);
    }

    /// Removes the buffer at the specified minimum and returns it if it exists.
    pub fn remove(&mut self, pos: IVec3) -> Option<VoxelBuffer<V, S>> {
        let pos = ilattice::glam::IVec3::from(pos.to_array());
        self.chunks.remove(&pos.into())
    }

    #[inline]
    pub const fn shape_mask(&self) -> IVec3 {
        self.shape_mask
    }
}
