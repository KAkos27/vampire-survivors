use bevy::prelude::*;
use bevy_rapier2d::{
    dynamics::RigidBody,
    geometry::{ActiveCollisionTypes, ActiveEvents, Collider, Sensor},
    pipeline::CollisionEvent,
};
use rand::seq::IndexedRandom;

use crate::{
    enemy::{DamageEnemy, Enemy},
    game::{BaseStats, GameState},
    player::Player,
};

pub struct MeteorPlugin;

impl Plugin for MeteorPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<MeteorStrike>();
        app.add_systems(Startup, setup_meteor);
        app.add_systems(
            Update,
            (
                strike_meteor,
                on_meteor_strike,
                update_meteor_aura,
                despawn_expired_meteor_auras,
            )
                .run_if(in_state(GameState::Playing)),
        );
    }
}

#[derive(Component, Clone, Copy)]
pub struct Meteor {
    level: u32,
}

#[derive(Component)]
pub struct MeteorAura {
    lifetime: Timer,
}

#[derive(Resource)]
pub struct MeteorCooldown(pub Timer);

#[derive(Resource)]
pub struct MeteorAuraDamageTimer(pub Timer);

#[derive(Message)]
pub struct MeteorStrike {
    pub position: Vec3,
}

pub struct MeteorStats {
    cooldown: f32,
    strike_damage_multiplier: f32,
    aura_time: f32,
    aura_damage_multiplier: f32,
    aura_tick_time: f32,
    aura_radius: f32,
}

impl Meteor {
    pub fn get_stats(self) -> MeteorStats {
        match self.level {
            1 => MeteorStats {
                cooldown: 2.0,
                strike_damage_multiplier: 0.75,
                aura_time: 3.0,
                aura_damage_multiplier: 0.2,
                aura_tick_time: 0.5,
                aura_radius: 50.0,
            },
            2 => MeteorStats {
                cooldown: 2.0,
                strike_damage_multiplier: 0.75,
                aura_time: 4.0,
                aura_damage_multiplier: 0.3,
                aura_tick_time: 0.5,
                aura_radius: 50.0,
            },
            3 => MeteorStats {
                cooldown: 2.0,
                strike_damage_multiplier: 0.80,
                aura_time: 4.0,
                aura_damage_multiplier: 0.4,
                aura_tick_time: 0.4,
                aura_radius: 75.0,
            },
            4 => MeteorStats {
                cooldown: 2.0,
                strike_damage_multiplier: 0.80,
                aura_time: 5.0,
                aura_damage_multiplier: 0.4,
                aura_tick_time: 0.3,
                aura_radius: 75.0,
            },
            _ => MeteorStats {
                cooldown: 1.5,
                strike_damage_multiplier: 0.9,
                aura_time: 6.0,
                aura_damage_multiplier: 0.5,
                aura_tick_time: 0.3,
                aura_radius: 100.0,
            },
        }
    }
}

fn setup_meteor(mut commands: Commands) {
    let meteor_ability = Meteor { level: 1 };
    let stats = meteor_ability.get_stats();

    commands.insert_resource(MeteorCooldown(Timer::from_seconds(
        stats.cooldown,
        TimerMode::Repeating,
    )));
    commands.insert_resource(MeteorAuraDamageTimer(Timer::from_seconds(
        stats.aura_tick_time,
        TimerMode::Repeating,
    )));
    commands.spawn(meteor_ability);
}

fn strike_meteor(
    mut message_writer: MessageWriter<DamageEnemy>,
    mut meteor_message_writer: MessageWriter<MeteorStrike>,
    mut cooldown: ResMut<MeteorCooldown>,
    time: Res<Time>,
    meteor_query: Query<&Meteor>,
    enemy_query: Query<(Entity, &Transform), With<Enemy>>,
    player_query: Query<&BaseStats, With<Player>>,
) {
    if !cooldown.0.tick(time.delta()).just_finished() {
        return;
    }
    let enemies: Vec<(Entity, &Transform)> = enemy_query
        .iter()
        .map(|(entity, transform)| (entity, transform))
        .collect();

    let Some((target, transform)) = enemies.choose(&mut rand::rng()) else {
        return;
    };

    let Ok(meteor) = meteor_query.single() else {
        return;
    };

    let Ok(stats) = player_query.single() else {
        return;
    };

    meteor_message_writer.write(MeteorStrike {
        position: transform.translation,
    });
    message_writer.write(DamageEnemy {
        target: *target,
        amount: meteor.get_stats().strike_damage_multiplier * stats.damage,
    });
}

fn on_meteor_strike(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut message_reader: MessageReader<MeteorStrike>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    meteor_query: Query<&Meteor>,
) {
    let Ok(meteor) = meteor_query.single() else {
        return;
    };
    let stats = meteor.get_stats();
    for message in message_reader.read() {
        commands.spawn((
            MeteorAura {
                lifetime: Timer::from_seconds(stats.aura_time, TimerMode::Once),
            },
            Mesh2d(meshes.add(Circle::new(1.0))),
            MeshMaterial2d(materials.add(Color::srgb(1.0, 0.5, 0.5))),
            Transform::from_translation(message.position)
                .with_scale(Vec3::splat(meteor.get_stats().aura_radius)),
            RigidBody::KinematicPositionBased,
            Collider::ball(1.0),
            Sensor,
            ActiveEvents::COLLISION_EVENTS,
            ActiveCollisionTypes::KINEMATIC_KINEMATIC,
        ));
    }
}

fn update_meteor_aura(
    mut collision_events: MessageReader<CollisionEvent>,
    mut demage_messages: MessageWriter<DamageEnemy>,
    mut damage_timer: ResMut<MeteorAuraDamageTimer>,
    aura_query: Query<&mut MeteorAura>,
    meteor_query: Query<&Meteor>,
    time: Res<Time>,
    enemy_query: Query<Entity, With<Enemy>>,
    player_query: Query<&BaseStats, (With<Player>, Without<Enemy>)>,
) {
    if !damage_timer.0.tick(time.delta()).just_finished() {
        return;
    }

    let Ok(stats) = player_query.single() else {
        return;
    };

    let Ok(meteor) = meteor_query.single() else {
        return;
    };

    for event in collision_events.read() {
        let CollisionEvent::Started(first_col, second_col, _) = event else {
            continue;
        };

        let other_entity =
            if aura_query.get(*first_col).is_ok() && enemy_query.contains(*second_col) {
                *second_col
            } else if aura_query.get(*second_col).is_ok() && enemy_query.contains(*first_col) {
                *first_col
            } else {
                continue;
            };

        if let Ok(enemy_entity) = enemy_query.get(other_entity) {
            demage_messages.write(DamageEnemy {
                target: enemy_entity,
                amount: stats.damage * meteor.get_stats().aura_damage_multiplier,
            });
        }
    }
}

fn despawn_expired_meteor_auras(
    mut commands: Commands,
    mut aura_query: Query<(Entity, &mut MeteorAura)>,
    time: Res<Time>,
) {
    for (entity, mut aura) in &mut aura_query {
        aura.lifetime.tick(time.delta());

        if aura.lifetime.is_finished() {
            commands.entity(entity).despawn();
        }
    }
}
