
// Simple platformer example
// Run command:
// cargo run run --example platformer --features bevy_xpbd_3d

use bevy_xpbd_3d::{prelude::{LinearVelocity, CollidingEntities, AngularVelocity, Position, RayHits}, PhysicsSchedule, PhysicsStepSet};
use space_editor::prelude::{*, component::EntityLink, spatial_query::RayCasterPrefab};
use bevy::{prelude::*, ecs::{entity::MapEntities, reflect::ReflectMapEntities}};


fn main() {
    App::default()
        .add_plugins(DefaultPlugins)
        .add_plugins(SpaceEditorPlugin::default())
        .add_systems(Startup, simple_editor_setup)
        .add_systems(Startup, configure_editor)

        .editor_registry::<PlayerController>()
        .editor_relation::<PlayerController, RigidBodyPrefab>()
        .editor_relation::<PlayerController, RayCasterPrefab>()

        .editor_registry::<FollowCamera>()
        .editor_relation::<FollowCamera, Camera3d>()

        .editor_tab(EditorTabName::Other("simple_tab".to_string()), "Simnple tab".into(), simple_tab_system)
        
        .add_systems(PhysicsSchedule, move_player.run_if(in_state(EditorState::Game)).before(PhysicsStepSet::BroadPhase))
        .add_systems(Update, camera_follow.run_if(in_state(EditorState::Game)))
        .run();
}

fn simple_tab_system(
    mut ui : NonSendMut<EditorUiRef>) {
    let ui = &mut ui.0;
    ui.label("Hello editor");
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
    pub jump_speed : f32,
    jumped : bool
}

impl Default for PlayerController {
    fn default() -> Self {
        Self {
            speed: 10.0,
            jump_speed: 100.0,
            jumped: false
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
    mut query: Query<(Entity, &mut LinearVelocity, &mut AngularVelocity, &mut PlayerController, &RayHits, &mut Transform)>,
    time : Res<Time>
) {
    for (_e, mut vel, mut rot, mut controller, hits, tranform) in query.iter_mut() {

        //take 1th hit, because 0th hit is self hit
        if let Some(hit) = hits.iter_sorted().nth(1) {
            if hit.time_of_impact > 0.7 {
                continue
            }
            info!("time of impact: {:?} {:?}", hit.entity, hit.time_of_impact);
            let frw = tranform.forward();
            let up = tranform.up();

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

            if keyboard_input.pressed(KeyCode::Space) && !controller.jumped {
                target_vel += up * controller.jump_speed;
                controller.jumped = true;
            }
            if !keyboard_input.pressed(KeyCode::Space) {
                controller.jumped = false;
            }

            //smooth change vel
            let cur_vel = vel.0;
            let cur_vel = vel.0 + (target_vel - cur_vel) * 10.0 * time.delta_seconds();
            vel.0 = cur_vel;
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
