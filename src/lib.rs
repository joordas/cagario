use std::time::Duration;

use bevy::prelude::*;

use cells::Cell;
use simula_viz::{
    grid::{Grid, GridBundle},
    lines::{LineMesh, LinesMaterial},
};

use bevy_renet::renet::{
    ChannelConfig, ReliableChannelConfig, RenetConnectionConfig, UnreliableChannelConfig,
    NETCODE_KEY_BYTES,
};
use serde::{Deserialize, Serialize};
use smooth_bevy_cameras::{LookTransform, LookTransformBundle, Smoother};

pub mod cells;
pub mod main_menu;
pub mod physics;
pub mod player;

pub const FIELD_SIZE: f32 = 900.0;

pub const PRIVATE_KEY: &[u8; NETCODE_KEY_BYTES] = b"an example very very secret key."; // 32-bytes
pub const PROTOCOL_ID: u64 = 7;

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum GameState {
    MainMenu,
    InGame,
    GameOver,
}

#[derive(Resource)]
pub struct GameAssets {
    // pub tower_base_scene: Handle<Scene>,
}

#[derive(Resource)]
pub struct Game {
    pub cell_spawn_timer: Timer,
}

#[derive(Reflect, Component, Default)]
#[reflect(Component)]
pub struct FollowCamera {
    pub height: f32,
    pub distance: f32,
    pub speed: f32,
}

#[derive(Debug, Component)]
pub struct Player {
    pub id: u64,
}

#[derive(Component)]
pub struct ControlledPlayer;

#[derive(Debug, Default, Clone, Copy, Reflect, Serialize, Deserialize, Component, Resource)]
#[reflect(Component)]
pub struct PlayerInput {
    pub up: bool,
    pub down: bool,
    pub left: bool,
    pub right: bool,
}

#[derive(Debug, Serialize, Deserialize, Component)]
pub enum PlayerCommand {
    BasicAttack { cast_at: Vec3 },
}

pub enum ClientChannel {
    Input,
    Command,
}

pub enum ServerChannel {
    ServerMessages,
    NetworkedEntities,
}

#[derive(Debug, Serialize, Deserialize, Component)]
pub enum ServerMessages {
    PlayerCreate {
        entity: Entity,
        id: u64,
        translation: [f32; 3],
    },
    PlayerRemove {
        id: u64,
    },
    SpawnNpcCell {
        entity: Entity,
        translation: [f32; 3],
        size: f32,
    },
    DespawnEntity {
        entity: Entity,
    },
    UpdateEntityCell {
        entity: Entity,
        size: f32,
    },
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct NetworkedEntities {
    pub entities: Vec<Entity>,
    pub translations: Vec<[f32; 3]>,
    pub scalings: Vec<[f32; 3]>,
}

impl From<ClientChannel> for u8 {
    fn from(channel_id: ClientChannel) -> Self {
        match channel_id {
            ClientChannel::Command => 0,
            ClientChannel::Input => 1,
        }
    }
}

impl ClientChannel {
    pub fn channels_config() -> Vec<ChannelConfig> {
        vec![
            ReliableChannelConfig {
                channel_id: Self::Input.into(),
                message_resend_time: Duration::ZERO,
                ..Default::default()
            }
            .into(),
            ReliableChannelConfig {
                channel_id: Self::Command.into(),
                message_resend_time: Duration::ZERO,
                ..Default::default()
            }
            .into(),
        ]
    }
}

impl From<ServerChannel> for u8 {
    fn from(channel_id: ServerChannel) -> Self {
        match channel_id {
            ServerChannel::NetworkedEntities => 0,
            ServerChannel::ServerMessages => 1,
        }
    }
}

impl ServerChannel {
    pub fn channels_config() -> Vec<ChannelConfig> {
        vec![
            UnreliableChannelConfig {
                channel_id: Self::NetworkedEntities.into(),
                sequenced: true, // We don't care about old positions
                ..Default::default()
            }
            .into(),
            ReliableChannelConfig {
                channel_id: Self::ServerMessages.into(),
                message_resend_time: Duration::from_millis(200),
                ..Default::default()
            }
            .into(),
        ]
    }
}

pub fn client_connection_config() -> RenetConnectionConfig {
    RenetConnectionConfig {
        send_channels_config: ClientChannel::channels_config(),
        receive_channels_config: ServerChannel::channels_config(),
        ..Default::default()
    }
}

pub fn server_connection_config() -> RenetConnectionConfig {
    RenetConnectionConfig {
        send_channels_config: ServerChannel::channels_config(),
        receive_channels_config: ClientChannel::channels_config(),
        ..Default::default()
    }
}

pub fn spawn_grid_lines(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut lines_materials: ResMut<Assets<LinesMaterial>>,
    line_mesh: Res<LineMesh>,
) {
    let start_color: Color = Color::rgb(0.4, 0.4, 0.4);
    let end_color: Color = Color::rgb(0.4, 0.4, 0.4);

    commands
        .spawn(GridBundle {
            grid: Grid {
                size: FIELD_SIZE as u32,
                divisions: FIELD_SIZE as u32 / 4,
                start_color,
                end_color,
                ..default()
            },
            mesh: meshes.add(line_mesh.clone()),
            material: lines_materials.add(LinesMaterial {}),
            transform: Transform::from_translation(Vec3::new(0.0, 0.01, 0.0)),
            ..default()
        })
        .insert(Name::new("Grid"));
}
pub fn spawn_scene(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    // mut lines_materials: ResMut<Assets<LinesMaterial>>,
    // line_mesh: Res<LineMesh>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands
        .spawn(PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Plane { size: FIELD_SIZE })),
            material: materials.add(Color::WHITE.into()),
            ..Default::default()
        })
        .insert(Name::new("Floor"));

    commands.insert_resource(AmbientLight {
        color: Color::WHITE,
        brightness: 0.5,
    });
}

pub fn setup_camera(mut commands: Commands) {
    commands
        .spawn(LookTransformBundle {
            transform: LookTransform {
                eye: Vec3::new(0.0, 8., 2.5),
                target: Vec3::new(0.0, 0.5, 0.0),
            },
            smoother: Smoother::new(0.9),
        })
        .insert(Camera3dBundle {
            transform: Transform::from_xyz(0., 8.0, 2.5)
                .looking_at(Vec3::new(0.0, 0.5, 0.0), Vec3::Y),
            ..default()
        });
}

pub fn camera_follow(
    mut camera_query: Query<&mut LookTransform, (With<Camera>, Without<ControlledPlayer>)>,
    player_query: Query<(&Transform, &Cell), With<ControlledPlayer>>,
) {
    let mut cam_transform = camera_query.single_mut();
    if let Ok((player_transform, cell)) = player_query.get_single() {
        cam_transform.eye.x = player_transform.translation.x;
        cam_transform.eye.z = player_transform.translation.z + 16.5 + cell.size;
        cam_transform.eye.y = player_transform.translation.y + (3.0 * cell.size);
        cam_transform.target = player_transform.translation;
    }
}

// /// set up a simple 3D scene
// pub fn setup_level(
//     mut commands: Commands,
//     mut meshes: ResMut<Assets<Mesh>>,
//     mut materials: ResMut<Assets<StandardMaterial>>,
// ) {
//     // plane
//     commands
//         .spawn(PbrBundle {
//             mesh: meshes.add(Mesh::from(shape::Box::new(10., 1., 10.))),
//             material: materials.add(Color::rgb(0.3, 0.5, 0.3).into()),
//             transform: Transform::from_xyz(0.0, -1.0, 0.0),
//             ..Default::default()
//         })
//         .insert(Collider::cuboid(5., 0.5, 5.));
//     // light
//     commands.spawn(PointLightBundle {
//         point_light: PointLight {
//             intensity: 1500.0,
//             shadows_enabled: true,
//             ..Default::default()
//         },
//         transform: Transform::from_xyz(4.0, 8.0, 4.0),
//         ..Default::default()
//     });
// }

// #[derive(Debug, Component)]
// pub struct Projectile {
//     pub duration: Timer,
// }

// pub fn spawn_fireball(
//     commands: &mut Commands,
//     meshes: &mut ResMut<Assets<Mesh>>,
//     materials: &mut ResMut<Assets<StandardMaterial>>,
//     translation: Vec3,
//     mut direction: Vec3,
// ) -> Entity {
//     if !direction.is_normalized() {
//         direction = Vec3::X;
//     }
//     commands
//         .spawn(PbrBundle {
//             mesh: meshes.add(Mesh::from(shape::Icosphere {
//                 radius: 0.1,
//                 subdivisions: 5,
//             })),
//             material: materials.add(Color::rgb(1.0, 0.0, 0.0).into()),
//             transform: Transform::from_translation(translation),
//             ..Default::default()
//         })
//         .insert(RigidBody::Dynamic)
//         .insert(LockedAxes::ROTATION_LOCKED | LockedAxes::TRANSLATION_LOCKED_Y)
//         .insert(Collider::ball(0.1))
//         .insert(Velocity::linear(direction * 10.))
//         .insert(ActiveEvents::COLLISION_EVENTS)
//         .insert(Projectile {
//             duration: Timer::from_seconds(1.5, TimerMode::Once),
//         })
//         .id()
// }

// A 3D ray, with an origin and direction. The direction is guaranteed to be normalized.
// #[derive(Debug, PartialEq, Copy, Clone, Default)]
// pub struct Ray3d {
//     pub(crate) origin: Vec3,
//     pub(crate) direction: Vec3,
// }

// impl Ray3d {
//     pub fn new(origin: Vec3, direction: Vec3) -> Self {
//         Ray3d { origin, direction }
//     }

//     pub fn from_screenspace(
//         windows: &Res<Windows>,
//         camera: &Camera,
//         camera_transform: &GlobalTransform,
//     ) -> Option<Self> {
//         let window = windows.get_primary().unwrap();
//         let cursor_position = match window.cursor_position() {
//             Some(c) => c,
//             None => return None,
//         };

//         let view = camera_transform.compute_matrix();
//         let screen_size = camera.logical_target_size()?;
//         let projection = camera.projection_matrix();
//         let far_ndc = projection.project_point3(Vec3::NEG_Z).z;
//         let near_ndc = projection.project_point3(Vec3::Z).z;
//         let cursor_ndc = (cursor_position / screen_size) * 2.0 - Vec2::ONE;
//         let ndc_to_world: Mat4 = view * projection.inverse();
//         let near = ndc_to_world.project_point3(cursor_ndc.extend(near_ndc));
//         let far = ndc_to_world.project_point3(cursor_ndc.extend(far_ndc));
//         let ray_direction = far - near;

//         Some(Ray3d::new(near, ray_direction))
//     }

//     pub fn intersect_y_plane(&self, y_offset: f32) -> Option<Vec3> {
//         let plane_normal = Vec3::Y;
//         let plane_origin = Vec3::new(0.0, y_offset, 0.0);
//         let denominator = self.direction.dot(plane_normal);
//         if denominator.abs() > f32::EPSILON {
//             let point_to_point = plane_origin * y_offset - self.origin;
//             let intersect_dist = plane_normal.dot(point_to_point) / denominator;
//             let intersect_position = self.direction * intersect_dist + self.origin;
//             Some(intersect_position)
//         } else {
//             None
//         }
//     }
// }
