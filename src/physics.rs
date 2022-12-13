use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
use bevy_renet::renet::RenetServer;

use crate::{
    cells::{Cell, NpcCell},
    Player, PlayerInput, ServerChannel, ServerLobby, ServerMessages,
};

#[derive(Bundle)]
pub struct PhysicsBundle {
    flags: ActiveEvents,
    active_collition_types: ActiveCollisionTypes,

    colliding_entities: CollidingEntities,
    rigid_body: RigidBody,
    rotation_contraint: LockedAxes,
    velocity: Velocity,
}

impl PhysicsBundle {
    pub fn moving_entity() -> Self {
        Self {
            flags: ActiveEvents::COLLISION_EVENTS,
            active_collition_types: ActiveCollisionTypes::default()
                | ActiveCollisionTypes::KINEMATIC_KINEMATIC,
            colliding_entities: CollidingEntities::default(),
            rigid_body: RigidBody::KinematicPositionBased,
            rotation_contraint: LockedAxes::ROTATION_LOCKED,
            velocity: Velocity::zero(),
        }
    }
}

fn cell_collision_detection(
    mut commands: Commands,
    mut player_query: Query<
        (Entity, &mut Player, &mut Cell),
        (With<Player>, With<Cell>, Without<NpcCell>),
    >,
    mut colliding_entities_query: Query<
        (Entity, &CollidingEntities, &mut Cell),
        (With<Cell>, With<NpcCell>),
    >,
    mut server: ResMut<RenetServer>,
) {
    for (cell_entity, colliding_entities, cell) in colliding_entities_query.iter_mut() {
        for (player_entity, mut _player, mut player_cell) in player_query.iter_mut() {
            if colliding_entities.contains(player_entity) {
                if player_cell.size > cell.size {
                    let new_size = player_cell.size + cell.size / 2.0;
                    player_cell.size = new_size;
                    commands.entity(cell_entity).despawn_recursive();
                    let message = ServerMessages::UpdateEntityCell {
                        entity: player_entity,
                        size: new_size,
                    };
                    let message = bincode::serialize(&message).unwrap();
                    server.broadcast_message(ServerChannel::ServerMessages, message);
                }
            }
        }
    }
}

fn cell_on_removal_system(mut server: ResMut<RenetServer>, removed_cells: RemovedComponents<Cell>) {
    for entity in removed_cells.iter() {
        let message = ServerMessages::DespawnEntity { entity };
        let message = bincode::serialize(&message).unwrap();

        server.broadcast_message(ServerChannel::ServerMessages, message);
    }
}

fn player_to_player_collision_detection(
    mut commands: Commands,
    mut collision_events: EventReader<CollisionEvent>,
    mut lobby: ResMut<ServerLobby>,
    cell_query: Query<Option<&Cell>>,
    mut player_query: Query<(Entity, &mut Player), With<PlayerInput>>,
    mut server: ResMut<RenetServer>,
) {
    for collision_event in collision_events.iter() {
        if let CollisionEvent::Started(entity1, entity2, _) = collision_event {
            if let Ok(Some(cell_1)) = cell_query.get(*entity1) {
                if let Ok(Some(cell_2)) = cell_query.get(*entity2) {
                    if cell_1.size > cell_2.size {
                        for (player_entity, player) in player_query.iter_mut() {
                            if player_entity == *entity1 {
                                let new_size = cell_1.size + cell_2.size / 2.0;
                                let message = ServerMessages::UpdateEntityCell {
                                    entity: *entity1,
                                    size: new_size,
                                };
                                let message = bincode::serialize(&message).unwrap();
                                server.broadcast_message(ServerChannel::ServerMessages, message);
                            } else if player_entity == *entity2 {
                                let new_size = cell_2.size + cell_1.size / 2.0;
                                let message = ServerMessages::UpdateEntityCell {
                                    entity: *entity2,
                                    size: new_size,
                                };
                                let message = bincode::serialize(&message).unwrap();
                                server.broadcast_message(ServerChannel::ServerMessages, message);

                                if let Some(player_entity) = lobby.players.remove(&player.id) {
                                    commands.entity(player_entity).despawn();
                                }

                                let message = bincode::serialize(&ServerMessages::PlayerRemove {
                                    id: player.id,
                                })
                                .unwrap();
                                server.broadcast_message(ServerChannel::ServerMessages, message);
                            }
                        }
                    }
                }
            }
        }
    }
}

pub struct PhysicsPlugin;

impl Plugin for PhysicsPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(cell_collision_detection);
        app.add_system(player_to_player_collision_detection);
        app.add_system_to_stage(CoreStage::PostUpdate, cell_on_removal_system);
        // app.add_system_to_stage(CoreStage::PostUpdate, player_on_removal_system);
    }
}
