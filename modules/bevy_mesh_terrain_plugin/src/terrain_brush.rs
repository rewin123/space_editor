use bevy::prelude::*;
use space_editor_ui::{
    ext::bevy_mod_picking::backends::raycast::bevy_mod_raycast::{prelude::Raycast, CursorRay},
    prelude::*,
};

use crate::{
    terrain_chunks::BrushableTerrain,
    terrain_tool::{TerrainTools, ToolMode},
};
use bevy_mesh_terrain::edit::{BrushType, EditTerrainEvent, EditingTool};

pub struct TerrainBrushPlugin;

impl Plugin for TerrainBrushPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            update_brush_paint
                .after(ui_camera_block)
                .in_set(UiSystemSet::AfterShow),
        );
    }
}

#[derive(Debug)]
struct EditingToolData {
    editing_tool: EditingTool,
    brush_type: BrushType,
    brush_radius: f32,
    brush_hardness: f32,
}

impl From<TerrainTools> for EditingToolData {
    fn from(state: TerrainTools) -> Self {
        let editing_tool = EditingTool::from(state.clone());

        Self {
            editing_tool,
            brush_radius: state.brush_radius as f32,
            brush_type: state.brush_type,
            brush_hardness: (state.brush_hardness as f32) / 100.0,
        }
    }
}

impl From<TerrainTools> for EditingTool {
    fn from(state: TerrainTools) -> Self {
        match state.tool_mode {
            ToolMode::Height => EditingTool::SetHeightMap {
                height: state.color.r,
            },
            ToolMode::Splat => EditingTool::SetSplatMap {
                r: state.color.r as u8,
                g: state.color.g as u8,
                b: state.color.b as u8,
            },
        }
    }
}

pub fn update_brush_paint(
    mut local_camera_block_latency: Local<usize>,
    mouse_input: Res<Input<MouseButton>>, //detect mouse click
    cursor_ray: Res<CursorRay>,
    mut raycast: Raycast,
    mut edit_event_writer: EventWriter<EditTerrainEvent>,
    game_view_tab: Res<GameViewTab>,
    brushable_terrain_query: Query<Entity, With<BrushableTerrain>>,
    mut camera_move_enabled: ResMut<EditorCameraEnabled>,
) {
    if !mouse_input.pressed(MouseButton::Left) {
        return;
    }

    if !camera_move_enabled.0 {
        return;
    }

    if *local_camera_block_latency > 0 {
        *local_camera_block_latency -= 1;
        camera_move_enabled.0 = false;
    }

    let active_tool_index = game_view_tab.active_tool.unwrap_or_default();

    if let Some(active_tool) = game_view_tab.tools.get(active_tool_index) {
        if let Some(tools_state) = active_tool.as_any().downcast_ref::<TerrainTools>() {
            // `tool` is now a reference to `TerrainTool`
            println!("Active tool is a TerrainTool!");
            // You can now use `tool` as a `TerrainTool`

            let tool_data: EditingToolData = (tools_state).clone().into();

            println!("brushing terrain w tool {:?}", tool_data);

            let radius = tool_data.brush_radius;
            let brush_hardness = tool_data.brush_hardness;
            let brush_type = tool_data.brush_type;

            // let tool = EditingTool::SetSplatMap(5,1,0,25.0,false);

            if let Some(cursor_ray) = **cursor_ray {
                if let Some((intersection_entity, intersection_data)) =
                    raycast.cast_ray(cursor_ray, &default()).first()
                {
                    //if we raycast to a brushable terrain, send terrain edit events
                    if brushable_terrain_query
                        .get(*intersection_entity)
                        .ok()
                        .is_some()
                    {
                        let hit_point = intersection_data.position();

                        //offset this by the world psn offset of the entity !? would need to query its transform ?  for now assume 0 offset.
                        let hit_coordinates = Vec2::new(hit_point.x, hit_point.z);

                        //use an event to pass the entity and hit coords to the terrain plugin so it can edit stuff there

                        println!("brushing terrain at {:?}", hit_coordinates);

                        camera_move_enabled.0 = false;
                        *local_camera_block_latency = 4;

                        edit_event_writer.send(EditTerrainEvent {
                            entity: intersection_entity.clone(),
                            tool: tool_data.editing_tool,
                            brush_type,
                            brush_hardness,
                            coordinates: hit_coordinates,
                            radius,
                        });
                    }
                }
            }
        }
    }
}
