use bevy::{prelude::*, render::mesh::VertexAttributeValues};

use crate::resources::*;

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
        VertexAttributeValues::from(mesh_data.0),
    );
    mesh.insert_attribute(
        Mesh::ATTRIBUTE_COLOR,
        VertexAttributeValues::from(mesh_data.2),
    );
    mesh.set_indices(Some(mesh_data.1));
    mesh.duplicate_vertices();
    mesh.compute_flat_normals();

    let material = materials.add(StandardMaterial {
        base_color: Color::WHITE,
        ..default()
    });
    commands.spawn((
        PbrBundle {
            mesh: meshes.add(mesh),
            material,
            ..default()
        },
        TerrainDrawTag,
    ));
}

pub fn redraw_terrain(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut res: ResMut<TerrainMap>,
    query: Query<Entity, With<TerrainDrawTag>>,
) {
    if res.has_changes {
        for entity in query.iter() {
            commands.entity(entity).despawn();
        }

        let mesh_data = res.terrain_mesh();
        let mut mesh = Mesh::new(bevy::render::render_resource::PrimitiveTopology::TriangleList);
        mesh.insert_attribute(
            Mesh::ATTRIBUTE_POSITION,
            VertexAttributeValues::from(mesh_data.0),
        );
        mesh.insert_attribute(
            Mesh::ATTRIBUTE_COLOR,
            VertexAttributeValues::from(mesh_data.2),
        );
        mesh.set_indices(Some(mesh_data.1));

        let material = materials.add(StandardMaterial {
            base_color: Color::WHITE,
            ..default()
        });
        mesh.duplicate_vertices();
        mesh.compute_flat_normals();

        commands.spawn((
            PbrBundle {
                mesh: meshes.add(mesh),
                material,
                ..default()
            },
            TerrainDrawTag,
        ));
        res.has_changes = false;
    }
}

#[derive(Component)]
pub struct TerrainDrawTag;
