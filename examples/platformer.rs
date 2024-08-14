/// Simple platformer example
/// Run command:
/// cargo run --example platformer --features bevy_xpbd_3d
use bevy::{
    ecs::{entity::MapEntities, reflect::ReflectMapEntities},
    prelude::*,
};
use space_editor::prelude::*;
use avian3d::prelude::*;

fn main() {
    App::default()
        .add_plugins(DefaultPlugins)
        .add_plugins(SpaceEditorPlugin)
        .add_systems(Startup, simple_editor_setup)
        .add_systems(Startup, configure_editor)
        .editor_registry::<PlayerController>()
        .editor_relation::<PlayerController, RigidBodyPrefab>()
        .editor_relation::<PlayerController, RayCasterPrefab>()
        .editor_registry::<FollowCamera>()
        .editor_relation::<FollowCamera, Camera3d>()
        // .add_systems(Update, move_player.run_if(in_state(EditorState::Game)))
        // .add_systems(Update, camera_follow.run_if(in_state(EditorState::Game)))
        .run();
}

fn configure_editor(mut load_event: EventWriter<EditorEvent>) {
    load_event.send(EditorEvent::Load(EditorPrefabPath::File(
        "scenes/platformer.scn.ron".to_string(),
    )));
}

#[derive(Component, Reflect, Clone)]
#[reflect(Component, Default)]
struct PlayerController {
    pub speed: f32,
    pub jump_speed: f32,
    jumped: bool,
}

impl Default for PlayerController {
    fn default() -> Self {
        Self {
            speed: 10.0,
            jump_speed: 100.0,
            jumped: false,
        }
    }
}

#[derive(Component, Reflect, Clone)]
#[reflect(Component, Default, MapEntities)]
struct FollowCamera {
    pub dist: f32,
    pub target: EntityLink,
    pub speed: f32,
}

impl Default for FollowCamera {
    fn default() -> Self {
        Self {
            dist: 10.0,
            target: EntityLink::default(),
            speed: 10.0,
        }
    }
}

impl MapEntities for FollowCamera {
    fn map_entities<M: EntityMapper>(&mut self, entity_mapper: &mut M) {
        self.target.entity = entity_mapper.map_entity(self.target.entity);
    }
}

fn move_player(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut query: Query<(
        Entity,
        &mut LinearVelocity,
        &mut AngularVelocity,
        &mut PlayerController,
        &RayHits,
        &mut Transform,
    )>,
    time: Res<Time>,
) {
    for (_e, mut vel, mut rot, mut controller, hits, transform) in query.iter_mut() {
        //take 1th hit, because 0th hit is self hit
        if let Some(hit) = hits.iter_sorted().next() {
            if hit.time_of_impact > 0.7 {
                continue;
            }
            info!("time of impact: {:?} {:?}", hit.entity, hit.time_of_impact);
            let frw = transform.forward();
            // let up = transform.up();
            let right = transform.right();

            let mut target_vel = Vector::new(0.0, 0.0, 0.0);
            if keyboard_input.pressed(KeyCode::KeyW) {
                target_vel += Vec3::from(frw);
            }
            if keyboard_input.pressed(KeyCode::KeyS) {
                target_vel -= Vec3::from(frw);
            }
            //rotate
            if keyboard_input.pressed(KeyCode::KeyA) {
                rot.y = 2.0;
            }
            if keyboard_input.pressed(KeyCode::KeyD) {
                rot.y = -2.0;
            }
            if !keyboard_input.pressed(KeyCode::KeyA) && !keyboard_input.pressed(KeyCode::KeyD) {
                rot.y -= 10.0 * rot.y * time.delta_seconds();
            }

            target_vel *= controller.speed;

            info!("target_vel: {:?}", target_vel);

            //smooth change vel
            let mut cur_vel = vel.0;
            cur_vel = vel.0 + (target_vel - cur_vel) * 10.0 * time.delta_seconds();

            if keyboard_input.just_pressed(KeyCode::Space) && !controller.jumped {
                cur_vel += right * controller.jump_speed / 12.0;
                controller.jumped = true;
            }
            if !keyboard_input.just_pressed(KeyCode::Space) {
                controller.jumped = false;
            }

            vel.0 = cur_vel;
        } else {
            info!("no hits");
        }
    }
}

fn camera_follow(
    targets: Query<&Position, Without<FollowCamera>>,
    mut cameras: Query<(&mut Transform, &FollowCamera)>,
    time: Res<Time>,
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
