use bevy::{core_pipeline::clear_color::ClearColorConfig, prelude::*, render::view::RenderLayers};
use bevy_egui::EguiPlugin;
use bevy_inspector_egui::WorldInspectorPlugin;
use simula_action::ActionPlugin;
use simula_camera::{flycam::*, orbitcam::*};
use simula_video::rt;

use bevy_rapier3d::{
    prelude::{NoUserData, RapierPhysicsPlugin},
};

pub const HEIGHT: f32 = 720.0;
pub const WIDTH: f32 = 1280.0;

pub const FIELD_SIZE: f32 = 800.0;

mod main_menu;
mod player;
mod cells;
mod physics;

use main_menu::*;
use player::*;
use cells::*;
use physics::*;

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
    cell_spawn_timer: Timer,
}

#[derive(Reflect, Component, Default)]
#[reflect(Component)]
pub struct FollowCamera {
    height: f32,
    distance: f32,
    speed: f32,
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
        .add_plugin(RapierPhysicsPlugin::<NoUserData>::default())
        // .add_plugin(RapierDebugRenderPlugin::default())
        .add_plugin(EguiPlugin)
        .add_plugin(ActionPlugin)
        .add_plugin(OrbitCameraPlugin)
        // .add_plugin(FlyCameraPlugin)
        .add_plugin(WorldInspectorPlugin::new())
        .add_state(GameState::InGame)
        .insert_resource(Game {
            cell_spawn_timer: Timer::from_seconds(0.1, TimerMode::Repeating),
        })
        // my plugins
        .add_plugin(MainMenuPlugin)
        .add_plugin(PlayerPlugin)
        .add_plugin(CellsPlugin)
        .add_plugin(PhysicsPlugin)
        .add_startup_system_to_stage(StartupStage::PreStartup, asset_loading)
        .add_startup_system(spawn_scene)
        .add_startup_system(setup)
        .add_system_set(
            SystemSet::on_update(GameState::InGame)
                .with_system(update_camera),
        )
        .run();
}

fn spawn_scene(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
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

//     
fn setup(mut commands: Commands, player_query: Query<&Transform, With<Player>>, mut images: ResMut<Assets<Image>>) {
    let rt_image = images.add(rt::common_render_target_image(UVec2 { x: 256, y: 256 }));
    // first check if there is a player
    // let player_transform = player_query.single();
    commands
        .spawn(Camera3dBundle {
            transform: Transform::from_xyz(4.4, 77.3, -91.180)
                .looking_at(Vec3::new(0.0, 1.0, 0.0), Vec3::Y),
                
            ..default()
        })
        .insert(RenderLayers::all())
        .with_children(|parent| {
            let mut _child = parent.spawn(Camera3dBundle {
                camera_3d: Camera3d {
                    clear_color: ClearColorConfig::Custom(Color::BLACK),
                    ..default()
                },
                camera: Camera {
                    priority: -1,
                    target: bevy::render::camera::RenderTarget::Image(rt_image.clone()),
                    ..default()
                },
                ..default()
            });
        })
        .insert(FollowCamera {
                distance: 60.0,
                height: 50.0,
                speed: 1.0,
            })
        .insert(FlyCamera::default());
}

// update camera position following player from a distance
fn update_camera(
    time: Res<Time>,
    mut camera_query: Query<(&mut Transform, &FollowCamera), With<FollowCamera>>,
    player_query: Query<(&Transform, &Cell), (With<Player>, Without<FollowCamera>)>,
) {
    for (mut transform, follow_camera) in camera_query.iter_mut() {
        let (player_transform, cell) = player_query.single();

        // if player_transform  {
            let mut camera_pos = transform.translation;
            let player_pos = player_transform.translation;

            let mut camera_dir = camera_pos - player_pos;
            camera_dir = camera_dir.normalize();

            camera_pos = player_pos + camera_dir * (follow_camera.distance + cell.size);
            camera_pos.y = (cell.size * 2.0) + follow_camera.height;


            let t = (time.delta_seconds() * follow_camera.speed).min(1.0); // adjust the speed of the transition using the `follow_camera.speed` value
            transform.translation = transform.translation.lerp(camera_pos, t);
            transform.look_at(player_pos, Vec3::Y);
        // }
    }
}



fn asset_loading(mut commands: Commands, 
    // assets: Res<AssetServer>
) {
    commands.insert_resource(GameAssets {
        // tower_base_scene: assets.load("TowerBase.glb#Scene0"),
  
        
    });
}
