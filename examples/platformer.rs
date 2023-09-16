use std::any::{Any, TypeId};

use bevy_xpbd_3d::{prelude::{LinearVelocity, CollidingEntities, AngularVelocity, Position}};
use space_editor::prelude::{*, component::EntityLink};
use bevy::{prelude::*, ecs::{entity::MapEntities, reflect::ReflectMapEntities}};

//Simple platformer example
//To run execute command:
// cargo run run --example platformer --features bevy_xpbd_3d

fn main() {
    App::default()
        .add_plugins(DefaultPlugins)
        .add_plugins(SpaceEditorPlugin::default())
        .add_systems(Startup, simple_editor_setup)
        .add_systems(Startup, configure_editor)

        .editor_registry::<PlayerController>()
        .editor_relation::<PlayerController, RigidBodyPrefab>()

        .editor_registry::<FollowCamera>()
        .editor_relation::<FollowCamera, Camera3d>()
        
        .add_systems(Update, move_player.run_if(in_state(EditorState::Game)))
        .add_systems(PostUpdate, camera_follow.run_if(in_state(EditorState::Game)))
        .run();
}

fn configure_editor(
    mut load_event : EventWriter<LoadEvent>
) {
    load_event.send(LoadEvent { path: "level_test".to_string() });
}

#[derive(Component, Reflect, Clone)]
#[reflect(Component, Default)]
struct PlayerController {
    pub speed : f32,
    pub jump_speed : f32
}

impl Default for PlayerController {
    fn default() -> Self {
        Self {
            speed: 10.0,
            jump_speed: 100.0
        }
    }
}

#[derive(Component, Reflect, Clone)]
#[reflect(Component, Default, MapEntities)]
struct FollowCamera {
    pub dist : f32,
    pub target : EntityLink,
    pub speed : f32
}

impl Default for FollowCamera {
    fn default() -> Self {
        Self {
            dist : 10.0,
            target : EntityLink::default(),
            speed : 10.0
        }
    }
}

impl MapEntities for FollowCamera {
    fn map_entities(&mut self, entity_mapper: &mut bevy::ecs::entity::EntityMapper) {
        self.target.entity = entity_mapper.get_or_reserve(self.target.entity);
    }
}

fn move_player(
    keyboard_input: Res<Input<KeyCode>>,
    mut query: Query<(Entity, &mut LinearVelocity, &mut AngularVelocity, &PlayerController, &CollidingEntities, &mut Transform)>,
    time : Res<Time>
) {
    for (e, mut vel, mut rot, controller, colliding, mut tranform) in query.iter_mut() {
        if colliding.len() > 0 {
            let frw = tranform.forward();
            let up = tranform.up();
            let right = tranform.right();

            let mut target_vel = Vector::new(0.0, 0.0, 0.0);
            if keyboard_input.pressed(KeyCode::W) {
                target_vel += frw;
            }
            if keyboard_input.pressed(KeyCode::S) {
                target_vel -= frw;
            }
            //rotate
            if keyboard_input.pressed(KeyCode::A) {
                rot.y = 2.0;
            } 
            if keyboard_input.pressed(KeyCode::D) {
                rot.y = -2.0;
            }
            if !keyboard_input.pressed(KeyCode::A) && !keyboard_input.pressed(KeyCode::D) {
                rot.y -= 10.0 * rot.y * time.delta_seconds();
            }
            
            target_vel *= controller.speed;

            if keyboard_input.pressed(KeyCode::Space) {
                target_vel += up * controller.jump_speed;
            }

            //smooth change vel
            let cur_vel = vel.0.clone();
            vel.0 = vel.0 + (target_vel - cur_vel) * 10.0 * time.delta_seconds();
        } else {

        }
    }    
}

fn camera_follow(
    targets : Query<&Position, Without<FollowCamera>>,
    mut cameras : Query<(&mut Transform, &FollowCamera)>,
    time : Res<Time>
) {
    for (mut cam_transform, cam) in cameras.iter_mut() {
        if cam.target.entity != Entity::PLACEHOLDER {
            if let Ok(target_transform) = targets.get(cam.target.entity) {
                let look_pos = cam_transform.translation + cam_transform.forward() * cam.dist;
                let dp = target_transform.0 - look_pos;
                cam_transform.translation += dp * time.delta_seconds() * cam.speed;
            }
        }
    }
}
