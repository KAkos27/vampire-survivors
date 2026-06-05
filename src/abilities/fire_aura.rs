use std::time::Duration;

use bevy::prelude::*;
use bevy_rapier2d::{
    dynamics::RigidBody,
    geometry::{ActiveCollisionTypes, ActiveEvents, Collider, Sensor},
    pipeline::CollisionEvent,
};

use crate::{
    enemy::{DamageEnemy, Enemy},
    game::{BaseStats, GameState},
    player::Player,
};

pub struct FireAuraPlugin;

impl Plugin for FireAuraPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<UpgradeFireAura>();
        app.add_systems(Startup, setup_aura);
        app.add_systems(
            Update,
            (update_aura_position, on_enemy_hit, upgrade_fire_aura)
                .run_if(in_state(GameState::Playing)),
        );
    }
}

#[derive(Component)]
pub struct FireAura {
    level: u32,
}

#[derive(Message)]
pub struct UpgradeFireAura;

struct FireAuraStats {
    radius: f32,
    damage_muliplier: f32,
    cooldown: f32,
}

#[derive(Resource)]
pub struct AuraDamageTimer(pub Timer);

impl FireAura {
    fn get_stats(&self) -> FireAuraStats {
        match self.level {
            1 => FireAuraStats {
                radius: 100.0,
                damage_muliplier: 0.25,
                cooldown: 1.0,
            },
            2 => FireAuraStats {
                radius: 120.0,
                damage_muliplier: 0.25,
                cooldown: 1.0,
            },
            3 => FireAuraStats {
                radius: 120.0,
                damage_muliplier: 0.5,
                cooldown: 1.0,
            },
            4 => FireAuraStats {
                radius: 150.0,
                damage_muliplier: 0.5,
                cooldown: 1.0,
            },
            _ => FireAuraStats {
                radius: 200.0,
                damage_muliplier: 0.8,
                cooldown: 0.8,
            },
        }
    }
}

fn setup_aura(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let aura = FireAura { level: 1 };
    let stats = aura.get_stats();

    commands.insert_resource(AuraDamageTimer(Timer::from_seconds(
        stats.cooldown,
        TimerMode::Repeating,
    )));

    commands.spawn((
        Mesh2d(meshes.add(Circle::new(1.0))),
        MeshMaterial2d(materials.add(Color::srgb(1.0, 0.5, 0.5))),
        Transform::from_scale(Vec3::splat(stats.radius)),
        RigidBody::KinematicPositionBased,
        Collider::ball(1.0),
        Sensor,
        ActiveEvents::COLLISION_EVENTS,
        ActiveCollisionTypes::KINEMATIC_KINEMATIC,
        aura,
    ));
}

fn upgrade_fire_aura(
    mut messages: MessageReader<UpgradeFireAura>,
    mut aura_query: Query<(&mut FireAura, &mut Transform, &mut Collider)>,
    mut damage_timer: ResMut<AuraDamageTimer>,
) {
    for _ in messages.read() {
        let Ok((mut aura, mut transform, mut collider)) = aura_query.single_mut() else {
            return;
        };

        aura.level += 1;

        let stats = aura.get_stats();

        transform.scale = Vec3::splat(stats.radius);
        *collider = Collider::ball(1.0);

        damage_timer
            .0
            .set_duration(Duration::from_secs_f32(stats.cooldown));
    }
}

fn update_aura_position(
    aura_query: Query<&mut Transform, With<FireAura>>,
    player_query: Query<&Transform, (With<Player>, Without<FireAura>)>,
) {
    let Ok(player_transform) = player_query.single() else {
        return;
    };

    for mut aura_transform in aura_query {
        aura_transform.translation = player_transform.translation;
    }
}

fn on_enemy_hit(
    mut collision_events: MessageReader<CollisionEvent>,
    mut demage_messages: MessageWriter<DamageEnemy>,
    mut damage_timer: ResMut<AuraDamageTimer>,
    aura_query: Query<&mut FireAura>,
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
            let Ok(aura) = aura_query.single() else {
                return;
            };
            demage_messages.write(DamageEnemy {
                target: enemy_entity,
                amount: stats.damage * aura.get_stats().damage_muliplier,
            });
        }
    }
}
