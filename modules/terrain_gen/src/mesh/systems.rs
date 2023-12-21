use bevy::{prelude::*, render::mesh::VertexAttributeValues};
use space_shared::PrefabMarker;

use crate::heightmap::{HeightMap, MapSettings};

use super::{Generation, TerrainDrawTag, TerrainMesh, TerrainMeshId};

const TERRAIN_MESH_NAME: &str = "Terrain Mesh";

pub fn draw_terrain(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    settings: Res<MapSettings>,
    heightmap: Res<HeightMap>,
) {
    if heightmap.is_empty() {
        return;
    }
    let mesh_data = TerrainMesh::generate_mesh(&heightmap, &settings);
    let mut mesh = Mesh::new(bevy::render::render_resource::PrimitiveTopology::TriangleList);
    mesh.insert_attribute(
        Mesh::ATTRIBUTE_POSITION,
        VertexAttributeValues::from(mesh_data.vertices.clone()),
    );
    mesh.insert_attribute(
        Mesh::ATTRIBUTE_COLOR,
        VertexAttributeValues::from(mesh_data.colors()),
    );
    mesh.insert_attribute(
        Mesh::ATTRIBUTE_NORMAL,
        VertexAttributeValues::from(mesh_data.normals.clone()),
    );
    mesh.set_indices(Some(mesh_data.indices()));

    commands.insert_resource(mesh_data);

    let material = materials.add(StandardMaterial {
        base_color: Color::WHITE,
        ..default()
    });

    let mesh_handle = meshes.add(mesh);
    commands.insert_resource(TerrainMeshId(mesh_handle.id()));
    commands.spawn((
        PbrBundle {
            mesh: mesh_handle,
            material,
            ..default()
        },
        PrefabMarker,
        Name::from(TERRAIN_MESH_NAME),
        TerrainDrawTag,
    ));
}

pub fn redraw_terrain(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut settings: ResMut<MapSettings>,
    heightmap: Res<HeightMap>,
    mut terrain_mesh_data: ResMut<TerrainMesh>,
    mut mesh_handle_res: ResMut<TerrainMeshId>,
    query: Query<Entity, With<TerrainDrawTag>>,
) {
    if heightmap.is_empty() {
        return;
    }
    if settings.has_changes {
        let previous_id = mesh_handle_res.0;
        for entity in query.iter() {
            commands.entity(entity).despawn();
        }
        settings.update_seed();

        let mesh_data = TerrainMesh::generate_mesh(&heightmap, &settings);
        let mut mesh = Mesh::new(bevy::render::render_resource::PrimitiveTopology::TriangleList);
        mesh.insert_attribute(
            Mesh::ATTRIBUTE_POSITION,
            VertexAttributeValues::from(mesh_data.vertices.clone()),
        );
        mesh.insert_attribute(
            Mesh::ATTRIBUTE_COLOR,
            VertexAttributeValues::from(mesh_data.colors()),
        );
        mesh.insert_attribute(
            Mesh::ATTRIBUTE_NORMAL,
            VertexAttributeValues::from(mesh_data.normals.clone()),
        );
        mesh.set_indices(Some(mesh_data.indices()));
        *terrain_mesh_data = mesh_data;

        let material = materials.add(StandardMaterial {
            base_color: Color::WHITE,
            ..default()
        });

        let mesh_handle = meshes.add(mesh);
        mesh_handle_res.0 = mesh_handle.id();
        commands.spawn((
            PbrBundle {
                mesh: mesh_handle,
                material,
                ..default()
            },
            PrefabMarker,
            Name::from(TERRAIN_MESH_NAME),
            TerrainDrawTag,
        ));
        meshes.remove(previous_id);
        settings.has_changes = false;
    }
}
