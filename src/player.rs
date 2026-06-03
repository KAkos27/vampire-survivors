use bevy::prelude::*;
use bevy_rapier2d::{
    dynamics::RigidBody,
    geometry::{ActiveCollisionTypes, ActiveEvents, Collider, Sensor},
    pipeline::CollisionEvent,
};

use crate::{
    enemy::{AttackCooldown, Enemy},
    game::{BaseStats, Direction},
};

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_player).add_systems(
            Update,
            (
                update_player,
                on_player_hit,
                update_hp_bar,
                update_debug_text,
            ),
        );
    }
}

#[derive(Component)]
pub struct Player;

#[derive(Component)]
pub struct Experience {
    pub current: f32,
    pub level: u32,
    pub xp_to_next_level: f32,
}

#[derive(Component)]
pub struct HpBar;

#[derive(Component)]
pub struct HpBarFill;

#[derive(Component)]
pub struct DebugText;

fn setup_player(mut commands: Commands, asset_server: Res<AssetServer>) {
    let player_texture = asset_server.load("player.png");
    commands.spawn((
        Player,
        BaseStats::new(50.0, 200.0, 50.0),
        Experience {
            current: 0.0,
            level: 1,
            xp_to_next_level: 30.0,
        },
        Sprite {
            image: player_texture,
            ..Default::default()
        },
        Transform::from_xyz(0.0, 0.0, 0.0),
        Direction(Vec3::ZERO),
        RigidBody::KinematicPositionBased,
        Collider::ball(5.0),
        Sensor,
        ActiveEvents::COLLISION_EVENTS,
        ActiveCollisionTypes::KINEMATIC_KINEMATIC,
    ));
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                bottom: Val::Px(20.0),
                left: Val::Px(20.0),
                width: Val::Px(200.0),
                height: Val::Px(16.0),
                ..default()
            },
            BackgroundColor(Color::srgb(0.2, 0.2, 0.2)),
            HpBar,
        ))
        .with_children(|parent| {
            parent.spawn((
                Node {
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    ..default()
                },
                BackgroundColor(Color::srgb(0.8, 0.1, 0.1)),
                HpBarFill,
            ));
        });
    commands.spawn((
        DebugText,
        Text::new(""),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(20.0),
            right: Val::Px(5.0),
            ..default()
        },
    ));
}

fn update_debug_text(
    player_query: Query<(&BaseStats, &Experience), With<Player>>,
    mut text_query: Query<&mut Text, With<DebugText>>,
) {
    let Ok((stats, experience)) = player_query.single() else {
        return;
    };
    let Ok(mut text) = text_query.single_mut() else {
        return;
    };

    **text = format!(
        "Level: {}\nXP: {:.0} / {:.0}\n\nHP: {:.0} / {:.0}\nDamage: {:.0}\nSpeed: {:.0}",
        experience.level,
        experience.current,
        experience.xp_to_next_level,
        stats.health.current,
        stats.health.max,
        stats.damage,
        stats.speed,
    );
}

fn update_player(
    key: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    direction: Query<(&mut Direction, &mut Transform, &BaseStats), With<Player>>,
) {
    for (mut direction, mut tranform, stats) in direction {
        direction.0 = Vec3::ZERO;

        if key.pressed(KeyCode::KeyW) {
            direction.0.y = 1.0;
        }
        if key.pressed(KeyCode::KeyS) {
            direction.0.y = -1.0;
        }
        if key.pressed(KeyCode::KeyA) {
            direction.0.x = -1.0;
        }
        if key.pressed(KeyCode::KeyD) {
            direction.0.x = 1.0;
        }

        tranform.translation += direction.0.normalize_or_zero() * time.delta_secs() * stats.speed;
    }
}

fn on_player_hit(
    mut commands: Commands,
    mut collision_events: MessageReader<CollisionEvent>,
    mut player_query: Query<(Entity, &mut BaseStats), (Without<Enemy>, With<Player>)>,
    mut attack_cooldown: ResMut<AttackCooldown>,
    time: Res<Time>,
    enemy_query: Query<Entity, With<Enemy>>,
) {
    if !attack_cooldown.0.tick(time.delta()).just_finished() {
        return;
    }
    for event in collision_events.read() {
        let CollisionEvent::Started(first_col, second_col, _) = event else {
            continue;
        };

        let player_entity = if enemy_query.contains(*first_col) {
            *second_col
        } else if enemy_query.contains(*second_col) {
            *first_col
        } else {
            continue;
        };

        if let Ok((player, mut stats)) = player_query.get_mut(player_entity) {
            stats.health.take_damage(25.0);

            if stats.health.is_dead() {
                commands.entity(player).despawn();
            }
        }
    }
}

fn update_hp_bar(
    player_query: Query<&BaseStats, With<Player>>,
    mut bar_query: Query<&mut Node, With<HpBarFill>>,
) {
    let Ok(stats) = player_query.single() else {
        return;
    };
    let Ok(mut node) = bar_query.single_mut() else {
        return;
    };

    let percent = (stats.health.current / stats.health.max * 100.0).clamp(0.0, 100.0);
    node.width = Val::Percent(percent);
}

fn update_stats(stats: &mut BaseStats, level: u32) {
    let multiplier = 1.0 + (level as f32 - 1.0) * 0.1;
    let health_ratio = stats.health.current / stats.health.max;

    stats.damage = stats.base_damage * multiplier;
    stats.speed = stats.base_speed * multiplier;
    stats.health.max = stats.base_health * multiplier;
    stats.health.current = stats.health.max * health_ratio;
}

pub fn gain_xp(
    experience_query: &mut Query<(&mut Experience, &mut BaseStats), (With<Player>, Without<Enemy>)>,
    xp_drop: f32,
) {
    if let Ok((mut experience, mut stats)) = experience_query.single_mut() {
        experience.current += xp_drop;

        if experience.current >= experience.xp_to_next_level {
            let remaining = experience.current - experience.xp_to_next_level;
            experience.level += 1;
            experience.current = remaining;
            experience.xp_to_next_level *= 2.0;

            print!(
                "Level up!\n lvl: {},\n exp: {},\n exp to level: {}\n",
                experience.level, experience.current, experience.xp_to_next_level
            );
            update_stats(&mut stats, experience.level);
        }
    }
}
