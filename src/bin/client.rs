use std::{collections::HashMap, net::UdpSocket, time::SystemTime};

use bevy::{app::AppExit, prelude::*, window::exit_on_all_closed};
use bevy_egui::EguiPlugin;
use bevy_inspector_egui::WorldInspectorPlugin;
use bevy_renet::{
    renet::{ClientAuthentication, RenetClient},
    run_if_client_connected, RenetClientPlugin,
};
use simula_action::ActionPlugin;
use simula_camera::orbitcam::*;
// use simula_video::rt;
use smooth_bevy_cameras::LookTransformPlugin;

use bevy_rapier3d::prelude::{Collider, NoUserData, RapierPhysicsPlugin, Velocity};

use simula_viz::{grid::GridPlugin, lines::LinesPlugin};

pub const HEIGHT: f32 = 720.0;
pub const WIDTH: f32 = 1280.0;

use cagario::main_menu::*;
use cagario::physics::*;
use cagario::player::*;
use cagario::{
    camera_follow, cells::*, client_connection_config, setup_camera, spawn_grid_lines, spawn_scene,
    ClientChannel, ControlledPlayer, GameState, NetworkedEntities, PlayerCommand, PlayerInput,
    ServerChannel, ServerMessages, PROTOCOL_ID,
};

#[derive(Default, Resource)]
struct NetworkMapping(HashMap<Entity, Entity>);

#[derive(Debug)]
struct PlayerInfo {
    client_entity: Entity,
    server_entity: Entity,
}

#[derive(Debug, Default, Resource)]
pub struct ClientLobby {
    players: HashMap<u64, PlayerInfo>,
}
fn new_renet_client() -> RenetClient {
    let server_addr = "127.0.0.1:5000".parse().unwrap();
    let socket = UdpSocket::bind("127.0.0.1:0").unwrap();
    let connection_config = client_connection_config();
    let current_time = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap();
    let client_id = current_time.as_millis() as u64;
    let authentication = ClientAuthentication::Unsecure {
        client_id,
        protocol_id: PROTOCOL_ID,
        server_addr,
        user_data: None,
    };

    RenetClient::new(current_time, socket, connection_config, authentication).unwrap()
}

fn main() {
    let mut app = App::new();

    app.insert_resource(ClearColor(Color::rgb(0.2, 0.2, 0.2)))
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            window: WindowDescriptor {
                width: WIDTH,
                height: HEIGHT,
                title: "Cagario".to_string(),
                resizable: false,
                ..default()
            },
            ..default()
        }))
        .insert_resource(PlayerInput::default())
        .add_plugin(RapierPhysicsPlugin::<NoUserData>::default())
        // .add_plugin(RapierDebugRenderPlugin::default())
        .add_plugin(EguiPlugin)
        .add_plugin(ActionPlugin)
        .add_plugin(OrbitCameraPlugin)
        // .add_plugin(FlyCameraPlugin)
        .add_plugin(WorldInspectorPlugin::new())
        .add_state(GameState::InGame);

    app.add_event::<PlayerCommand>();
    app.register_type::<PlayerInput>();
    app.insert_resource(ClientLobby::default());
    app.insert_resource(PlayerInput::default());
    app.insert_resource(new_renet_client());
    app.insert_resource(NetworkMapping::default());

    // app.add_startup_system(setup_camera);
    // app.add_system(camera_follow);

    // my plugins
    app.add_plugin(MainMenuPlugin)
        .add_plugin(PlayerPlugin)
        // .add_plugin(CellsPlugin)
        .add_plugin(PhysicsPlugin)
        .add_plugin(LinesPlugin)
        .add_plugin(GridPlugin)
        .add_plugin(LookTransformPlugin)
        // .add_startup_system_to_stage(StartupStage::PreStartup, asset_loading)
        // .add_startup_system(setup)
        .add_plugin(RenetClientPlugin::default())
        .add_startup_system(spawn_scene)
        .add_startup_system(spawn_grid_lines)
        .add_startup_system(setup_camera)
        .add_system(camera_follow)
        .add_system(client_send_input.with_run_criteria(run_if_client_connected))
        .add_system(client_send_player_commands.with_run_criteria(run_if_client_connected))
        .add_system(client_sync_players.with_run_criteria(run_if_client_connected))
        .add_system(player_input)
        .add_system_to_stage(
            CoreStage::PostUpdate,
            disconnect_on_exit.after(exit_on_all_closed),
        )
        // .add_system_set(SystemSet::on_update(GameState::InGame).with_system(update_camera))
        .add_system_set(SystemSet::on_enter(GameState::InGame).with_system(spawn_grid_lines))
        .run();
}

// fn setup(
//     mut commands: Commands,
//     player_query: Query<&Transform, With<Player>>,
//     mut images: ResMut<Assets<Image>>,
// ) {
//     let rt_image = images.add(rt::common_render_target_image(UVec2 { x: 256, y: 256 }));
//     // first check if there is a player
//     // let player_transform = player_query.single();
//     commands
//         .spawn(Camera3dBundle {
//             transform: Transform::from_xyz(4.4, 77.3, -91.180)
//                 .looking_at(Vec3::new(0.0, 1.0, 0.0), Vec3::Y),

//             ..default()
//         })
//         .insert(RenderLayers::all())
//         .with_children(|parent| {
//             let mut _child = parent.spawn(Camera3dBundle {
//                 camera_3d: Camera3d {
//                     clear_color: ClearColorConfig::Custom(Color::BLACK),
//                     ..default()
//                 },
//                 camera: Camera {
//                     priority: -1,
//                     target: bevy::render::camera::RenderTarget::Image(rt_image.clone()),
//                     ..default()
//                 },
//                 ..default()
//             });
//         })
//         .insert(FollowCamera {
//             distance: 60.0,
//             height: 20.0,
//             speed: 1.0,
//         })
//         .insert(FlyCamera::default());
// }

// update camera position following player from a distance
// fn update_camera(
//     time: Res<Time>,
//     mut camera_query: Query<(&mut Transform, &FollowCamera), With<FollowCamera>>,
//     player_query: Query<(&Transform, &Cell), (With<Player>, Without<FollowCamera>)>,
// ) {
//     for (mut transform, follow_camera) in camera_query.iter_mut() {
//         let (player_transform, cell) = player_query.single();

//         // if player_transform  {
//         let mut camera_pos = transform.translation;
//         let player_pos = player_transform.translation;

//         let mut camera_dir = camera_pos - player_pos;
//         camera_dir = camera_dir.normalize();

//         camera_pos = player_pos + camera_dir * (follow_camera.distance + cell.size);
//         camera_pos.y = (cell.size * 2.0) + follow_camera.height;

//         let t = (time.delta_seconds() * follow_camera.speed).min(1.0); // adjust the speed of the transition using the `follow_camera.speed` value
//         transform.translation = transform.translation.lerp(camera_pos, t);
//         transform.look_at(player_pos, Vec3::Y);
//         // }
//     }
// }

fn client_send_input(player_input: Res<PlayerInput>, mut client: ResMut<RenetClient>) {
    let input_message = bincode::serialize(&*player_input).unwrap();
    client.send_message(ClientChannel::Input, input_message);
}

fn client_send_player_commands(
    mut player_commands: EventReader<PlayerCommand>,
    mut client: ResMut<RenetClient>,
) {
    for command in player_commands.iter() {
        let command_message = bincode::serialize(command).unwrap();
        client.send_message(ClientChannel::Command, command_message);
    }
}

fn client_sync_players(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut client: ResMut<RenetClient>,
    mut lobby: ResMut<ClientLobby>,
    mut network_mapping: ResMut<NetworkMapping>,
) {
    let client_id = client.client_id();
    while let Some(message) = client.receive_message(ServerChannel::ServerMessages) {
        let server_message = bincode::deserialize(&message).unwrap();
        match server_message {
            ServerMessages::PlayerCreate {
                id,
                translation,
                entity,
            } => {
                println!("Player {} connected.", id);
                let [x, y, z] = translation;
                let transform = Transform::from_xyz(x, y, z);
                let mut client_entity = commands.spawn(PbrBundle {
                    mesh: meshes.add(Mesh::from(shape::Icosphere {
                        radius: INITIAL_PLAYER_SIZE,
                        subdivisions: 4,
                    })),
                    material: materials.add(Color::rgb(0.5, 0.5, 1.0).into()),
                    transform,
                    ..Default::default()
                });

                client_entity
                    .insert(Cell {
                        size: INITIAL_PLAYER_SIZE,
                    })
                    .insert(Name::new("Player"))
                    .insert(PlayerInput::default())
                    .insert(Velocity::default())
                    .insert(Collider::ball(INITIAL_PLAYER_SIZE));

                print!("client id: {}, id {} ", client_id, id);
                if client_id == id {
                    client_entity.insert(ControlledPlayer);
                }

                let player_info = PlayerInfo {
                    server_entity: entity,
                    client_entity: client_entity.id(),
                };
                lobby.players.insert(id, player_info);
                network_mapping.0.insert(entity, client_entity.id());
            }
            ServerMessages::PlayerRemove { id } => {
                println!("Player {} disconnected.", id);
                if let Some(PlayerInfo {
                    server_entity,
                    client_entity,
                }) = lobby.players.remove(&id)
                {
                    commands.entity(client_entity).despawn();
                    network_mapping.0.remove(&server_entity);
                }
            }
            ServerMessages::SpawnNpcCell {
                entity,
                translation,
                size,
            } => {
                let [x, _y, z] = translation;
                let mut npc_entity = commands.spawn(PbrBundle {
                    mesh: meshes.add(Mesh::from(shape::Icosphere {
                        radius: size,
                        subdivisions: 4,
                    })),
                    material: materials.add(Color::rgb(x, z, size).into()),
                    transform: Transform::from_translation(Vec3::new(x, -size / 2.0, z)),
                    ..Default::default()
                });

                npc_entity
                    .insert(Cell { size })
                    .insert(Name::new("NPC"))
                    .insert(Collider::ball(size));
                network_mapping.0.insert(entity, npc_entity.id());
            }
        }
    }

    while let Some(message) = client.receive_message(ServerChannel::NetworkedEntities) {
        let networked_entities: NetworkedEntities = bincode::deserialize(&message).unwrap();

        for i in 0..networked_entities.entities.len() {
            if let Some(entity) = network_mapping.0.get(&networked_entities.entities[i]) {
                let translation = networked_entities.translations[i].into();
                let transform = Transform {
                    translation,
                    ..Default::default()
                };
                commands.entity(*entity).insert(transform);
            }
        }
    }
}

fn disconnect_on_exit(exit: EventReader<AppExit>, mut client: ResMut<RenetClient>) {
    if !exit.is_empty() && client.is_connected() {
        client.disconnect();
    }
}

fn player_input(keyboard_input: Res<Input<KeyCode>>, mut player_input: ResMut<PlayerInput>) {
    player_input.left = keyboard_input.pressed(KeyCode::A) || keyboard_input.pressed(KeyCode::Left);
    player_input.right =
        keyboard_input.pressed(KeyCode::D) || keyboard_input.pressed(KeyCode::Right);
    player_input.up = keyboard_input.pressed(KeyCode::W) || keyboard_input.pressed(KeyCode::Up);
    player_input.down = keyboard_input.pressed(KeyCode::S) || keyboard_input.pressed(KeyCode::Down);

    // if mouse_button_input.just_pressed(MouseButton::Left) {
    //     let target_transform = target_query.single();
    //     player_commands.send(PlayerCommand::BasicAttack {
    //         cast_at: target_transform.translation,
    //     });
    // }
}
