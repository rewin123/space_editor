use bevy::{prelude::*, render::mesh::VertexAttributeValues};

use crate::{
    heightmap::{HeightMap, MapSettings},
    UpdateTerrain,
};

use super::{Generation, TerrainDrawTag, TerrainMesh};

pub fn draw_terrain(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut events: EventReader<UpdateTerrain>,
    mut terrains: Query<(Entity, &mut HeightMap, &MapSettings)>,
) {
    if events.is_empty() {
        return;
    }

    for event in events.read() {
        match event {
            UpdateTerrain::All => {
                for (e, mut heightmap, settings) in terrains.iter_mut() {
                    *heightmap = settings.heightmap();
                    generate_mesh(
                        &heightmap,
                        settings,
                        &mut materials,
                        &mut meshes,
                        &mut commands,
                        e,
                    );
                }
            }
            UpdateTerrain::One(entity) => {
                if let Ok((e, mut heightmap, settings)) = terrains.get_mut(*entity) {
                    info!("Updating terrain {:?}", e);
                    *heightmap = settings.heightmap();
                    generate_mesh(
                        &heightmap,
                        settings,
                        &mut materials,
                        &mut meshes,
                        &mut commands,
                        e,
                    );
                }
            }
        }
    }

    events.clear();
}

fn generate_mesh(
    heightmap: &HeightMap,
    settings: &MapSettings,
    materials: &mut ResMut<'_, Assets<StandardMaterial>>,
    meshes: &mut ResMut<'_, Assets<Mesh>>,
    commands: &mut Commands<'_, '_>,
    e: Entity,
) {
    let mesh_data = TerrainMesh::generate_mesh(heightmap, settings);
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

    let material = materials.add(StandardMaterial {
        base_color: Color::WHITE,
        ..default()
    });

    let mesh_handle = meshes.add(mesh);

    commands
        .entity(e)
        .insert((mesh_handle, material, TerrainDrawTag));
}
