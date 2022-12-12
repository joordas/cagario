use std::{collections::HashMap, net::UdpSocket, time::SystemTime};

use bevy::{
    app::AppExit,
    diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin},
    prelude::*,
    window::exit_on_all_closed,
};
use bevy_egui::EguiPlugin;
use bevy_rapier3d::prelude::*;
use bevy_renet::{
    renet::{RenetServer, ServerAuthentication, ServerConfig, ServerEvent},
    RenetServerPlugin,
};
use cagario::{
    cells::{spawn_spheres, Cell},
    physics::{PhysicsBundle, PhysicsPlugin},
    player::{update_player_cell_size, INITIAL_PLAYER_SIZE},
    server_connection_config, ClientChannel, Player, PlayerCommand, PlayerInput, ServerChannel,
    ServerMessages, PROTOCOL_ID,
};

use bevy_inspector_egui::WorldInspectorPlugin;

#[derive(Debug, Default, Resource)]
pub struct ServerLobby {
    pub players: HashMap<u64, Entity>,
}

const PLAYER_MOVE_SPEED: f32 = 10.0;

fn new_renet_server() -> RenetServer {
    let server_addr = "127.0.0.1:5000".parse().unwrap();
    let socket = UdpSocket::bind(server_addr).unwrap();
    let connection_config = server_connection_config();
    let server_config =
        ServerConfig::new(64, PROTOCOL_ID, server_addr, ServerAuthentication::Unsecure);
    let current_time = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap();
    RenetServer::new(current_time, server_config, connection_config, socket).unwrap()
}

use cagario::*;
use rand::Rng;
use simula_viz::{grid::GridPlugin, lines::LinesPlugin};
use smooth_bevy_cameras::LookTransformPlugin;

fn main() {
    let mut app = App::new();
    app.add_plugins(DefaultPlugins);

    app.add_plugin(RenetServerPlugin::default());
    app.add_plugin(RapierPhysicsPlugin::<NoUserData>::default());
    app.add_plugin(RapierDebugRenderPlugin::default());
    app.add_plugin(FrameTimeDiagnosticsPlugin::default());
    app.add_plugin(LogDiagnosticsPlugin::default());
    app.add_plugin(EguiPlugin);
    app.add_plugin(WorldInspectorPlugin::new());
    // app.insert_resource(PlayerInput::default());

    app.add_plugin(PhysicsPlugin);
    app.add_plugin(LinesPlugin);
    app.add_plugin(GridPlugin);
    app.add_plugin(LookTransformPlugin);

    app.insert_resource(ServerLobby::default());
    app.insert_resource(new_renet_server());
    app.register_type::<Cell>();
    // app.insert_resource(RenetServerVisualizer::<200>::default());

    app.add_system(server_update_system);

    app.add_startup_system(spawn_scene);
    app.add_startup_system(spawn_grid_lines);
    // app.add_startup_system(setup_camera);
    // app.add_system(camera_follow);
    app.add_system(server_network_sync);
    app.add_system(move_players_system);
    app.add_system(spawn_spheres);
    app.add_system(update_player_cell_size);
    // app.add_system(move_players_system);
    // app.add_system(update_projectiles_system);
    // app.add_system(update_visulizer_system);
    // app.add_system(despawn_projectile_system);
    // app.add_system_to_stage(CoreStage::PostUpdate, projectile_on_removal_system);
    app.add_system_to_stage(
        CoreStage::PostUpdate,
        disconnect_clients_on_exit.after(exit_on_all_closed),
    );

    app.insert_resource(Game {
        cell_spawn_timer: Timer::from_seconds(1.0, TimerMode::Repeating),
    });

    // app.add_startup_system(setup_level);
    app.add_startup_system(setup_simple_camera);

    app.run();
}

#[allow(clippy::too_many_arguments)]
fn server_update_system(
    mut server_events: EventReader<ServerEvent>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut lobby: ResMut<ServerLobby>,
    mut server: ResMut<RenetServer>,
    // mut visualizer: ResMut<RenetServerVisualizer<200>>,
    players: Query<(Entity, &Player, &Transform)>,
) {
    for event in server_events.iter() {
        match event {
            ServerEvent::ClientConnected(id, _) => {
                println!("Player {} connected.", id);
                // visualizer.add_client(*id);

                // Initialize other players for this new client
                for (entity, player, transform) in players.iter() {
                    let translation: [f32; 3] = transform.translation.into();
                    let message = bincode::serialize(&ServerMessages::PlayerCreate {
                        id: player.id,
                        entity,
                        translation,
                    })
                    .unwrap();
                    server.send_message(*id, ServerChannel::ServerMessages, message);
                }
                let mut rng = rand::thread_rng();

                let x = rng.gen_range(-FIELD_SIZE / 2.0..FIELD_SIZE / 2.0) as f32;
                let z = rng.gen_range(-FIELD_SIZE / 2.0..FIELD_SIZE / 2.0) as f32;
                // let rand_transform = Transform::from_xyz(x, 0.0, z);
                let rand_transform = Transform::from_xyz(0.0, 0.0, 0.0);
                let player_entity = commands
                    .spawn(PbrBundle {
                        mesh: meshes.add(Mesh::from(shape::Icosphere {
                            radius: INITIAL_PLAYER_SIZE,
                            subdivisions: 4,
                        })),
                        material: materials.add(Color::rgb(0.5, 0.5, 1.0).into()),
                        transform: rand_transform,
                        ..Default::default()
                    })
                    .insert(Player { id: *id })
                    .insert(Cell {
                        size: INITIAL_PLAYER_SIZE,
                    })
                    .insert(Name::new("Player"))
                    .insert(PlayerInput::default())
                    .insert(Velocity::default())
                    .insert(PhysicsBundle::moving_entity())
                    .insert(Collider::ball(INITIAL_PLAYER_SIZE))
                    .id();

                lobby.players.insert(*id, player_entity);

                let translation: [f32; 3] = rand_transform.translation.into();
                let message = bincode::serialize(&ServerMessages::PlayerCreate {
                    id: *id,
                    entity: player_entity,
                    translation,
                })
                .unwrap();
                server.broadcast_message(ServerChannel::ServerMessages, message);
            }
            ServerEvent::ClientDisconnected(id) => {
                println!("Player {} disconnected.", id);
                // visualizer.remove_client(*id);
                if let Some(player_entity) = lobby.players.remove(id) {
                    commands.entity(player_entity).despawn();
                }

                let message =
                    bincode::serialize(&ServerMessages::PlayerRemove { id: *id }).unwrap();
                server.broadcast_message(ServerChannel::ServerMessages, message);
            }
        }
    }

    for client_id in server.clients_id().into_iter() {
        while let Some(message) = server.receive_message(client_id, ClientChannel::Command) {
            let command: PlayerCommand = bincode::deserialize(&message).unwrap();
            match command {
                PlayerCommand::BasicAttack { mut cast_at } => {
                    println!(
                        "Received basic attack from client {}: {:?}",
                        client_id, cast_at
                    );

                    if let Some(player_entity) = lobby.players.get(&client_id) {
                        if let Ok((_, _, player_transform)) = players.get(*player_entity) {
                            cast_at[1] = player_transform.translation[1];

                            let direction =
                                (cast_at - player_transform.translation).normalize_or_zero();
                            let mut translation = player_transform.translation + (direction * 0.7);
                            translation[1] = 1.0;
                        }
                    }
                }
            }
        }
        while let Some(message) = server.receive_message(client_id, ClientChannel::Input) {
            let input: PlayerInput = bincode::deserialize(&message).unwrap();
            if let Some(player_entity) = lobby.players.get(&client_id) {
                commands.entity(*player_entity).insert(input);
            }
        }
    }
}

fn move_players_system(mut query: Query<(&mut Transform, &PlayerInput)>, time: Res<Time>) {
    for (mut transform, input) in query.iter_mut() {
        let x = (input.right as i8 - input.left as i8) as f32;
        let y = (input.down as i8 - input.up as i8) as f32;
        let direction = Vec2::new(x, y).normalize_or_zero();

        transform.translation +=
            Vec3::new(direction.x, 0.0, direction.y) * PLAYER_MOVE_SPEED * time.delta_seconds();
    }
}

#[allow(clippy::type_complexity)]
fn server_network_sync(
    mut server: ResMut<RenetServer>,
    query: Query<(Entity, &Transform), With<Player>>,
) {
    let mut networked_entities = NetworkedEntities::default();
    for (entity, transform) in query.iter() {
        networked_entities.entities.push(entity);
        networked_entities
            .translations
            .push(transform.translation.into());
        networked_entities.scalings.push(transform.scale.into());
    }

    let sync_message = bincode::serialize(&networked_entities).unwrap();
    server.broadcast_message(ServerChannel::NetworkedEntities, sync_message);
}

pub fn setup_simple_camera(mut commands: Commands) {
    // camera
    commands.spawn(Camera3dBundle {
        transform: Transform::from_xyz(-5.5, 120.0, 75.5).looking_at(Vec3::ZERO, Vec3::Y),
        ..Default::default()
    });
}

fn disconnect_clients_on_exit(exit: EventReader<AppExit>, mut server: ResMut<RenetServer>) {
    if !exit.is_empty() {
        server.disconnect_clients();
    }
}
