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
        app.insert_resource(AuraDamageTimer(Timer::from_seconds(
            1.0,
            TimerMode::Repeating,
        )));
        app.add_systems(Startup, setup_aura);
        app.add_systems(
            Update,
            (update_aura_position, on_enemy_hit).run_if(in_state(GameState::Playing)),
        );
    }
}

#[derive(Component)]
pub struct FireAura {
    radius: f32,
}

#[derive(Resource)]
pub struct AuraDamageTimer(pub Timer);

fn setup_aura(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let aura = FireAura { radius: 100.0 };

    commands.spawn((
        Mesh2d(meshes.add(Circle::new(1.0))),
        MeshMaterial2d(materials.add(Color::srgb(1.0, 0.5, 0.5))),
        Transform::from_scale(Vec3::splat(aura.radius)),
        RigidBody::KinematicPositionBased,
        Collider::ball(1.0),
        Sensor,
        ActiveEvents::COLLISION_EVENTS,
        ActiveCollisionTypes::KINEMATIC_KINEMATIC,
        aura,
    ));
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
            demage_messages.write(DamageEnemy {
                target: enemy_entity,
                amount: stats.damage * 0.25,
            });
        }
    }
}
