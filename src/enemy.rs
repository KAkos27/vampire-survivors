use bevy::prelude::*;

use crate::{
    game::BaseStats,
    player::{Experience, Player, gain_xp},
};

pub struct EnemeyPlugin;

impl Plugin for EnemeyPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(AttackCooldown(Timer::from_seconds(
            2.0,
            TimerMode::Repeating,
        )));
        app.add_systems(Update, (update_enemies, separate_enemies));
    }
}

#[derive(Component)]
pub struct Enemy {
    pub xp_drop: f32,
}

#[derive(Resource)]
pub struct AttackCooldown(pub Timer);

fn update_enemies(
    mut enemy_query: Query<(&mut Transform, &BaseStats), With<Enemy>>,
    player_query: Query<&Transform, (Without<Enemy>, With<Player>)>,
    time: Res<Time>,
) {
    let Ok(player_transform) = player_query.single() else {
        return;
    };

    for (mut enemy_transform, stats) in &mut enemy_query {
        let direction = player_transform.translation - enemy_transform.translation;
        enemy_transform.translation += direction.normalize() * stats.speed * time.delta_secs();
    }
}

fn separate_enemies(
    mut param_set: ParamSet<(
        Query<(Entity, &Transform), With<Enemy>>,
        Query<&mut Transform, With<Enemy>>,
    )>,
    time: Res<Time>,
) {
    let enemies: Vec<(Entity, Vec3)> = param_set
        .p0()
        .iter()
        .map(|(e, t)| (e, t.translation))
        .collect();

    for (entity, pos) in &enemies {
        let mut push = Vec3::ZERO;
        for (other_entity, other_pos) in &enemies {
            if entity == other_entity {
                continue;
            }
            let diff = *pos - *other_pos;
            let dist = diff.length();
            if dist < 24.0 && dist > 0.0 {
                push += diff.normalize() * (24.0 - dist) * 50.0 * time.delta_secs();
            }
        }
        if let Ok(mut transform) = param_set.p1().get_mut(*entity) {
            transform.translation += push;
        }
    }
}

pub fn damage_enemy(
    commands: &mut Commands,
    experience_query: &mut Query<(&mut Experience, &mut BaseStats), (With<Player>, Without<Enemy>)>,
    stats: &mut BaseStats,
    enemy_entity: Entity,
    enemy: &Enemy,
    amount: f32,
) {
    stats.health.take_damage(amount);
    if stats.health.is_dead() {
        commands.entity(enemy_entity).despawn();
        gain_xp(experience_query, enemy.xp_drop);
    }
}
