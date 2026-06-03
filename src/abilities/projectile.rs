use bevy::prelude::*;
use bevy_rapier2d::{
    dynamics::RigidBody,
    geometry::{ActiveCollisionTypes, ActiveEvents, Collider, Sensor},
    pipeline::CollisionEvent,
};

use crate::{
    enemy::{Enemy, damage_enemy},
    game::BaseStats,
    player::{Experience, Player},
};

pub struct ProjectilePlugin;

impl Plugin for ProjectilePlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(ShootTimer(Timer::from_seconds(1.0, TimerMode::Repeating)))
            .add_systems(Update, (shoot, update_projectile, on_projectile_hit));
    }
}

#[derive(Component)]
pub struct Projectile {
    pub direction: Vec3,
    pub damage: f32,
    pub speed: f32,
}

#[derive(Resource)]
pub struct ShootTimer(pub Timer);

fn shoot(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    time: Res<Time>,
    mut shoot_timer: ResMut<ShootTimer>,
    player_query: Query<(&Transform, &BaseStats), With<Player>>,
    enemy_query: Query<&Transform, (Without<Player>, With<Enemy>)>,
) {
    if !shoot_timer.0.tick(time.delta()).just_finished() {
        return;
    }
    let Ok((player_transform, stats)) = player_query.single() else {
        return;
    };

    let nearest = enemy_query.iter().min_by(|a, b| {
        let dist_a = a.translation.distance(player_transform.translation);
        let dist_b = b.translation.distance(player_transform.translation);
        dist_a.partial_cmp(&dist_b).unwrap()
    });

    let Some(enemy_transform) = nearest else {
        return;
    };

    let direction =
        (enemy_transform.translation - player_transform.translation).normalize_or_zero();
    let angle = direction.x.atan2(-direction.y);

    commands.spawn((
        Projectile {
            direction,
            damage: stats.damage,
            speed: 300.0,
        },
        Sprite {
            image: asset_server.load("bullet.png"),
            ..Default::default()
        },
        Transform::from_translation(player_transform.translation)
            .with_rotation(Quat::from_rotation_z(angle)),
        RigidBody::KinematicPositionBased,
        Collider::ball(5.0),
        Sensor,
        ActiveEvents::COLLISION_EVENTS,
        ActiveCollisionTypes::KINEMATIC_KINEMATIC,
    ));
}

fn update_projectile(projectile_query: Query<(&Projectile, &mut Transform)>, time: Res<Time>) {
    for (projectile, mut transform) in projectile_query {
        transform.translation += projectile.direction * projectile.speed * time.delta_secs();
    }
}

fn on_projectile_hit(
    mut commands: Commands,
    mut collision_events: MessageReader<CollisionEvent>,
    projectile_query: Query<&Projectile>,
    mut enemy_query: Query<(Entity, &mut BaseStats, &Enemy)>,
    mut experience_query: Query<(&mut Experience, &mut BaseStats), (With<Player>, Without<Enemy>)>,
) {
    for event in collision_events.read() {
        let CollisionEvent::Started(first_col, second_col, _) = event else {
            continue;
        };

        let (projectile_entity, other_entity, projectile) =
            if let Ok(projectile) = projectile_query.get(*first_col) {
                (*first_col, *second_col, projectile)
            } else if let Ok(projectile) = projectile_query.get(*second_col) {
                (*second_col, *first_col, projectile)
            } else {
                continue;
            };

        if let Ok((enemy_entity, mut stats, enemy)) = enemy_query.get_mut(other_entity) {
            damage_enemy(
                &mut commands,
                &mut experience_query,
                &mut stats,
                enemy_entity,
                enemy,
                projectile.damage,
            );
            commands.entity(projectile_entity).despawn();
        }
    }
}
