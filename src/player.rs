use bevy::prelude::*;

use crate::{cells::Cell, physics::PhysicsBundle, GameState, FIELD_SIZE};

#[derive(Reflect, Component, Default)]
#[reflect(Component)]
pub struct Player {
    // Player attributes, such as position, orientation, etc.
    translation: Vec3,
    rotation: Quat,
    pub speed: Vec2,
    // pub size: f32,
}

pub struct PlayerPlugin;
impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<Player>()
            .add_system_set(
                SystemSet::on_update(GameState::InGame)
                    .with_system(player_controls)
                    .with_system(update_player_position)
                    .with_system(slow_down_players)
                    .with_system(update_player_cell_size.after(player_spawner)), // .with_system(player_renderer), // .with_system(camera_follow),
            )
            .add_system_set(SystemSet::on_enter(GameState::InGame).with_system(player_spawner));
    }
}

// Define the PlayerSpawner system
fn player_spawner(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    // system inputs, such as user input, time, etc.
    // mut players: ResMut<Game>,
) {
    // Spawn the Player's model
    commands
        .spawn(PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Icosphere {
                radius: 1.0,
                subdivisions: 4,
            })),
            material: materials.add(Color::rgb(0.5, 0.5, 1.0).into()),
            transform: Transform::from_translation(Vec3::new(0.0, 0.0, 0.0)),
            ..Default::default()
        })
        .insert(Player {
            translation: Vec3::new(0.0, 0.0, 0.0),
            rotation: Quat::from_xyzw(0.0, 0.0, 0.0, 0.0),
            speed: Vec2::new(0.0, 0.0),
        })
        .insert(Cell { size: 1.0 })
        .insert(Name::new("Player"))
        .insert(PhysicsBundle::moving_entity(Vec3::new(1.0, 1.0, 1.0)));
}

fn player_controls(
    keyboard: Res<Input<KeyCode>>,
    mut player_query: Query<(&mut Player, &Cell), (With<Player>, With<Cell>)>,
) {
    let (mut player, cell) = player_query.single_mut();
    // todo: a better way to determine max speed based on player size
    let max_speed = 10.0 / (cell.size / 3.0);

    let mut new_speed = player.speed;

    if keyboard.pressed(KeyCode::W) && new_speed.y > -max_speed {
        new_speed.y -= 0.4;
    }
    if keyboard.pressed(KeyCode::S) && new_speed.y < max_speed {
        new_speed.y += 0.4;
    }
    if keyboard.pressed(KeyCode::A) && new_speed.x < max_speed {
        new_speed.x += 0.4;
    }
    if keyboard.pressed(KeyCode::D) && new_speed.x > -max_speed {
        new_speed.x -= 0.4;
    }
    player.speed = new_speed;
}

fn update_player_position(
    mut player_query: Query<(&mut Transform, &mut Player), With<Player>>,
    time: Res<Time>,
) {
    let (mut transform, mut player) = player_query.single_mut();

    let mut forward = transform.forward();
    forward.y = 0.0;
    forward = forward.normalize();

    let mut left = transform.left();
    left.y = 0.0;
    left = left.normalize();

    // limit player movements to field

    if transform.translation.x > FIELD_SIZE / 2.0 {
        transform.translation.x = FIELD_SIZE / 2.0;
        player.speed.x = 0.0;
    }
    if transform.translation.x < -FIELD_SIZE / 2.0 {
        transform.translation.x = -FIELD_SIZE / 2.0;
        player.speed.x = 0.0;
    }

    if player.speed.y > 0.0 && !(transform.translation.y > FIELD_SIZE / 2.0) {
        transform.translation += forward * time.delta_seconds() * player.speed.y;
    }
    if player.speed.y < 0.0 && !(transform.translation.y < -FIELD_SIZE / 2.0) {
        transform.translation -= forward * time.delta_seconds() * -player.speed.y;
    }
    if player.speed.x < 0.0 && !(transform.translation.x < -FIELD_SIZE / 2.0) {
        transform.translation += left * time.delta_seconds() * -player.speed.x;
    }
    if player.speed.x > 0.0 && !(transform.translation.x > FIELD_SIZE / 2.0) {
        transform.translation -= left * time.delta_seconds() * player.speed.x;
    }
}

fn update_player_cell_size(
    // couldn't figure how to do this with mesh query to change the radius directly. Doing with transform instead.
    // mut player_query: Query<(&Handle<Mesh>, &mut Player), With<Player>>,
    mut player_query: Query<(&mut Transform, &Cell), (With<Cell>, With<Player>)>,
    // mut player_query: Query<(&mut Transform, &mut Player), With<Player>>,
) {
    let (mut transform, cell) = player_query.single_mut();

    transform.scale = Vec3::new(cell.size / 3.0, cell.size / 3.0, cell.size / 3.0);
}

fn slow_down_players(
    keyboard: Res<Input<KeyCode>>,
    mut player_query: Query<&mut Player, With<Player>>,
) {
    let no_keys_pressed = !keyboard.pressed(KeyCode::W)
        && !keyboard.pressed(KeyCode::S)
        && !keyboard.pressed(KeyCode::A)
        && !keyboard.pressed(KeyCode::D);

    let mut player = player_query.single_mut();

    if no_keys_pressed {
        if player.speed.y > 0.0 {
            player.speed.y -= 0.1;
        }
        if player.speed.y < 0.0 {
            player.speed.y += 0.1;
        }
        if player.speed.x < 0.0 {
            player.speed.x += 0.1;
        }
        if player.speed.x > 0.0 {
            player.speed.x -= 0.1;
        }
    }
}
