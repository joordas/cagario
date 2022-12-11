use bevy::prelude::*;
use bevy_rapier3d::prelude::Velocity;

use crate::{cells::Cell, GameState, Player, PlayerInput, FIELD_SIZE};

// #[derive(Reflect, Component, Default)]
// #[reflect(Component)]
// pub struct Player {
//     pub speed: f32,
//     pub movement_angle: f32,
// }
// const SPEED_DECREASE_RATE: f32 = 0.1;
pub const INITIAL_PLAYER_SIZE: f32 = 1.0;

#[derive(Reflect, Component, Default)]
pub struct Spawned;

#[derive(Component)]
pub struct Playing;

pub struct PlayerPlugin;
impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app
            // .register_type::<Player>()
            .add_system_set(
                SystemSet::on_update(GameState::InGame)
                    // .with_system(player_controls)
                    .with_system(move_players_system)
                    .with_system(limit_player_movement) // .with_system(update_player_position)
                    .with_system(update_player_cell_size),
                //, // .with_system(slow_down_players)
                // .with_system(player_renderer), // .with_system(camera_follow),
            );
        // .add_system_set(SystemSet::on_enter(GameState::InGame).with_system(player_spawner));
    }
}
// Define the PlayerSpawner system
// fn player_spawner(
//     mut commands: Commands,
//     mut meshes: ResMut<Assets<Mesh>>,
//     mut materials: ResMut<Assets<StandardMaterial>>,
//     // system inputs, such as user input, time, etc.
//     mut playing_entities: Query<Entity, (With<Playing>, Without<Spawned>)>,
// ) {
//     for playing_entity in playing_entities.iter_mut() {
//         // rand x and y based on PLAYING_FIELD_SIZE
//         let rand_x = rand::random::<f32>() * FIELD_SIZE as f32;
//         let rand_z = rand::random::<f32>() * FIELD_SIZE as f32;

//         commands
//             .entity(playing_entity)
//             .insert(Spawned)
//             .with_children(|parent| {
//                 parent
//                     .spawn(PbrBundle {
//                         mesh: meshes.add(Mesh::from(shape::Icosphere {
//                             radius: INITIAL_PLAYER_SIZE,
//                             subdivisions: 4,
//                         })),
//                         material: materials.add(Color::rgb(0.5, 0.5, 1.0).into()),
//                         transform: Transform::from_translation(Vec3::new(rand_x, 0.0, rand_z)),
//                         ..Default::default()
//                     })
//                     .insert(Player {
//                         speed: 0.0,
//                         movement_angle: 0.0,
//                     })
//                     .insert(Cell {
//                         size: INITIAL_PLAYER_SIZE,
//                     })
//                     .insert(Name::new("Player"))
//                     .insert(PlayerInput::default())
//                     .insert(Velocity::default())
//                     .insert(RigidBody::Dynamic)
//                     .insert(Collider::ball(INITIAL_PLAYER_SIZE));
//             });
//     }
// }

fn move_players_system(mut query: Query<(&mut Velocity, &PlayerInput)>) {
    for (mut velocity, input) in query.iter_mut() {
        let x = (input.right as i8 - input.left as i8) as f32;
        let y = (input.down as i8 - input.up as i8) as f32;
        let direction = Vec2::new(x, y).normalize_or_zero();
        velocity.linvel.x = direction.x * 2.0;
        velocity.linvel.z = direction.y * 2.0;
    }
}

// fn player_controls(
//     keyboard: Res<Input<KeyCode>>,
//     mut player_query: Query<(&mut Player, &Cell), (With<Player>, With<Cell>)>,
//     mut camera_query: Query<&mut Transform, With<FlyCamera>>,
// ) {
//     let (mut player, cell) = player_query.single_mut();
//     // todo: a better way to determine max speed based on player size
//     // let max_speed = 2.0;

//     // max speed based on player cell size, smaller is faster
//     let mut max_speed = 3.0 / cell.size;
//     max_speed = max_speed.max(0.5);

//     if keyboard.pressed(KeyCode::W) {
//         player.speed += 0.1;
//         //                 // clamp the player's speed to the maximum speed
//         player.speed = player.speed.min(max_speed);
//     }

//     if keyboard.pressed(KeyCode::S) {
//         player.speed -= 0.1;
//         player.speed = player.speed.max(-max_speed);
//     }

//     if keyboard.pressed(KeyCode::A) {
//         player.movement_angle += 0.1;
//     }

//     if keyboard.pressed(KeyCode::D) {
//         player.movement_angle -= 0.1;
//     }
// }

// fn update_player_position(mut player_query: Query<(&mut Transform, &Player), With<Player>>) {
//     let (mut transform, player) = player_query.single_mut();

//     let movement_vector = Vec2::new(
//         (player.speed * player.movement_angle.cos()) / 5.0,
//         (player.speed * player.movement_angle.sin()) / 5.0,
//     );

//     let new_position = Vec3::new(
//         transform.translation.x + movement_vector.x,
//         0.0,
//         transform.translation.z + movement_vector.y,
//     );

//     transform.translation = new_position;

//     // if player.speed.y > 0.0 && !(transform.translation.y > FIELD_SIZE / 2.0) {
//     //     transform.translation += forward * time.delta_seconds() * player.speed.y;
//     // }
//     // if player.speed.y < 0.0 && !(transform.translation.y < -FIELD_SIZE / 2.0) {
//     //     transform.translation -= forward * time.delta_seconds() * -player.speed.y;
//     // }
//     // if player.speed.x < 0.0 && !(transform.translation.x < -FIELD_SIZE / 2.0) {
//     //     transform.translation += left * time.delta_seconds() * -player.speed.x;
//     // }
//     // if player.speed.x > 0.0 && !(transform.translation.x > FIELD_SIZE / 2.0) {
//     //     transform.translation -= left * time.delta_seconds() * player.speed.x;
//     // }
// }

// limit player movement to field size
fn limit_player_movement(mut player_query: Query<&mut Transform, With<Player>>) {
    for player in player_query.iter_mut() {
        let mut transform = player;

        if transform.translation.x > FIELD_SIZE / 2.0 {
            transform.translation.x = FIELD_SIZE / 2.0;
        }
        if transform.translation.x < -FIELD_SIZE / 2.0 {
            transform.translation.x = -FIELD_SIZE / 2.0;
        }
        if transform.translation.z > FIELD_SIZE / 2.0 {
            transform.translation.z = FIELD_SIZE / 2.0;
        }
        if transform.translation.z < -FIELD_SIZE / 2.0 {
            transform.translation.z = -FIELD_SIZE / 2.0;
        }
    }
}

fn update_player_cell_size(
    // couldn't figure how to do this with mesh query to change the radius directly. Doing with transform instead.
    // mut player_query: Query<(&Handle<Mesh>, &mut Player), With<Player>>,
    mut player_query: Query<(&mut Transform, &Cell), (With<Cell>, With<Player>)>,
    // mut player_query: Query<(&mut Transform, &mut Player), With<Player>>,
) {
    for player in player_query.iter_mut() {
        let (mut transform, cell) = player;

        transform.scale = Vec3::new(cell.size / 4.0, cell.size / 4.0, cell.size / 4.0);
    }
}

// fn slow_down_players(
//     keyboard: Res<Input<KeyCode>>,
//     mut player_query: Query<&mut Player, With<Player>>,
// ) {
//     let no_keys_pressed = !keyboard.pressed(KeyCode::W)
//         && !keyboard.pressed(KeyCode::S)
//         && !keyboard.pressed(KeyCode::A)
//         && !keyboard.pressed(KeyCode::D);

//     let mut player = player_query.single_mut();

//     if no_keys_pressed {
//         if player.speed > 0.0 {
//             player.speed -= SPEED_DECREASE_RATE;
//             player.speed = player.speed.max(0.0);
//         }
//         if player.speed < 0.0 {
//             player.speed += SPEED_DECREASE_RATE;
//             player.speed = player.speed.min(0.0);
//         }
//     }
// }
