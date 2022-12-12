use bevy::prelude::*;

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
                    // .with_system(move_players_system)
                    .with_system(limit_player_movement) // .with_system(update_player_position)
                    .with_system(update_player_cell_size),
                //, // .with_system(slow_down_players)
                // .with_system(player_renderer), // .with_system(camera_follow),
            );
        // .add_system_set(SystemSet::on_enter(GameState::InGame).with_system(player_spawner));
    }
}

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

pub fn update_player_cell_size(
    // couldn't figure how to do this with mesh query to change the radius directly. Doing with transform instead.
    // mut player_query: Query<(&Handle<Mesh>, &mut Player), With<Player>>,
    mut player_query: Query<(&mut Transform, &Cell), (With<Cell>, With<PlayerInput>)>,
    // mut player_query: Query<(&mut Transform, &mut Player), With<Player>>,
) {
    for player in player_query.iter_mut() {
        let (mut transform, cell) = player;

        transform.scale = Vec3::new(cell.size / 4.0, cell.size / 4.0, cell.size / 4.0);
    }
}
