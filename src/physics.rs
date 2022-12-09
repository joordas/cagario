use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

use crate::{cells::Cell, player::Player};

#[derive(Bundle)]
pub struct PhysicsBundle {
    flags: ActiveEvents,
    active_collition_types: ActiveCollisionTypes,
    collider: Collider,
    colliding_entities: CollidingEntities,
    rigid_body: RigidBody,
    rotation_contraint: LockedAxes,
    velocity: Velocity,
}

impl PhysicsBundle {
    pub fn moving_entity(size: Vec3) -> Self {
        Self {
            flags: ActiveEvents::COLLISION_EVENTS,
            active_collition_types: ActiveCollisionTypes::default()
                | ActiveCollisionTypes::KINEMATIC_KINEMATIC,
            collider: Collider::cuboid(size.x / 2., size.y / 2., size.z / 2.),
            colliding_entities: CollidingEntities::default(),
            rigid_body: RigidBody::KinematicPositionBased,
            rotation_contraint: LockedAxes::ROTATION_LOCKED,
            velocity: Velocity::zero(),
        }
    }
}

fn player_collision_detection(
    mut commands: Commands,
    mut player_query: Query<(Entity, &mut Player), With<Player>>,
    mut colliding_entities_query: Query<(Entity, &CollidingEntities, &Cell), With<Cell>>,
) {
    for (cell_entity, colliding_entities, cell) in colliding_entities_query.iter_mut() {
        for (player_entity, mut player) in player_query.iter_mut() {
            if colliding_entities.contains(player_entity) && player.size > cell.size {
                player.size += cell.size;
                commands.entity(cell_entity).despawn_recursive();
            }
        }
    }
}

pub struct PhysicsPlugin;

impl Plugin for PhysicsPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(player_collision_detection);
    }
}
