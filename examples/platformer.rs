use bevy_xpbd_3d::prelude::{LinearVelocity, CollidingEntities};
use space_editor::prelude::*;
use bevy::prelude::*;

fn main() {
    App::default()
        .add_plugins(DefaultPlugins)
        .add_plugins(SpaceEditorPlugin::default())
        .add_systems(Startup, simple_editor_setup)
        .editor_registry::<PlayerController>()
        .editor_relation::<PlayerController, RigidBodyPrefab>()
        .add_systems(Update, move_player.run_if(in_state(EditorState::Game)))
        .run();
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

fn move_player(
    keyboard_input: Res<Input<KeyCode>>,
    mut query: Query<(&mut LinearVelocity, &PlayerController, &CollidingEntities, &mut Transform)>,
    time : Res<Time>
) {
    for (mut vel, controller, colliding, mut tranform) in query.iter_mut() {
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
                tranform.rotate_y(2.0 * time.delta_seconds());
            }
            if keyboard_input.pressed(KeyCode::D) {
                tranform.rotate_y(-2.0 * time.delta_seconds());
            }
            
            target_vel *= controller.speed;

            if keyboard_input.pressed(KeyCode::Space) {
                target_vel += up * controller.jump_speed;
            }

            //smooth change vel
            let cur_vel = vel.0.clone();
            vel.0 = vel.0 + (target_vel - cur_vel) * 10.0 * time.delta_seconds();
        } else {
            info!("No collide");
        }
    }    
}
