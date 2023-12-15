use bevy::{prelude::*, render::mesh::VertexAttributeValues};
use shared::PrefabMarker;

use crate::resources::{mesh::TerrainMesh, *};

const TERRAIN_MESH_NAME: &str = "Terrain Mesh";

#[derive(Resource, Clone, Debug, Default)]
pub struct TerrainMeshId(AssetId<Mesh>);

pub fn draw_terrain(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    res: Res<TerrainMap>,
) {
    let mesh_data = res.terrain_mesh();
    let mut mesh = Mesh::new(bevy::render::render_resource::PrimitiveTopology::TriangleList);
    mesh.insert_attribute(
        Mesh::ATTRIBUTE_POSITION,
        VertexAttributeValues::from(mesh_data.vertices.clone()),
    );
    mesh.insert_attribute(
        Mesh::ATTRIBUTE_COLOR,
        VertexAttributeValues::from(mesh_data.colors_from_biomes()),
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
    mut res: ResMut<TerrainMap>,
    mut terrain_mesh_data: ResMut<TerrainMesh>,
    mut mesh_handle_res: ResMut<TerrainMeshId>,
    query: Query<Entity, With<TerrainDrawTag>>,
) {
    if res.has_changes {
        let previous_id = mesh_handle_res.0;
        for entity in query.iter() {
            commands.entity(entity).despawn();
        }
        res.update_seed();

        let mesh_data = res.terrain_mesh();
        let mut mesh = Mesh::new(bevy::render::render_resource::PrimitiveTopology::TriangleList);
        mesh.insert_attribute(
            Mesh::ATTRIBUTE_POSITION,
            VertexAttributeValues::from(mesh_data.vertices.clone()),
        );
        mesh.insert_attribute(
            Mesh::ATTRIBUTE_COLOR,
            VertexAttributeValues::from(mesh_data.colors_from_biomes()),
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
        res.has_changes = false;
    }
}

#[derive(Component, Default, Clone, Reflect)]
pub struct TerrainDrawTag;
