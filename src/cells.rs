use bevy::prelude::*;
use rand::*;

use crate::{physics::PhysicsBundle, Game, GameState, FIELD_SIZE};

#[derive(Resource)]
pub struct MaxSpheres(usize);

pub struct CellsPlugin;

#[derive(Component)]
pub struct Cell {
    pub size: f32,
}
#[derive(Component)]
pub struct NpcCell;

impl Plugin for CellsPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(setup)
            .add_system_set(SystemSet::on_update(GameState::InGame).with_system(spawn_spheres));
    }
}

fn setup(mut commands: Commands) {
    // set the maximum number of spheres to spawn
    commands.insert_resource(MaxSpheres(100));
}

// define the system that will spawn the spheres
fn spawn_spheres(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    time: Res<Time>,
    max_spheres: Res<MaxSpheres>,
    // mut cell_query: Query<(&mut Transform, With<Cell>)>,
    mut game: ResMut<Game>,
) {
    // create a random number generator
    let mut rng = rand::thread_rng();

    game.cell_spawn_timer.tick(time.delta());
    // for (mut transform, mut spawner) in cell_query.iter_mut() {
    // check if the maximum number of spheres has been reached
    // if cell_query.len() >= max_spheres.0 {
    //     continue;
    // }

    if game.cell_spawn_timer.just_finished() {
        // generate random x, y, and z coordinates for the sphere's position
        let x = rng.gen_range(-FIELD_SIZE / 2.0..FIELD_SIZE / 2.0) as f32;
        let z = rng.gen_range(-FIELD_SIZE / 2.0..FIELD_SIZE / 2.0) as f32;
        // let y = rng.gen_range(-10.0..10.0) as f32;

        let size = rng.gen_range(0.4..1.4) as f32;
        commands
            .spawn(PbrBundle {
                transform: Transform::from_translation(Vec3::new(x, -size / 2.0, z)),
                mesh: meshes.add(Mesh::from(shape::Icosphere {
                    radius: size,
                    subdivisions: 4,
                })),
                material: materials.add(Color::rgb(x, z, size).into()),
                ..Default::default()
            })
            .insert(Name::new("Cell"))
            .insert(NpcCell)
            .insert(Cell { size })
            .insert(PhysicsBundle::moving_entity(Vec3::new(
                size * 1.5,
                size * 1.5,
                size * 1.5,
            )));
    }
}
