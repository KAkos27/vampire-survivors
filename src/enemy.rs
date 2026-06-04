use bevy::prelude::*;

use crate::{
    game::{BaseStats, GameState},
    player::{GainXp, Player},
};

pub struct EnemeyPlugin;

impl Plugin for EnemeyPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(AttackCooldown(Timer::from_seconds(
            2.0,
            TimerMode::Repeating,
        )));
        app.add_message::<DamageEnemy>();
        app.add_systems(
            Update,
            (update_enemies, separate_enemies, damage_enemies).run_if(in_state(GameState::Playing)),
        );
    }
}

#[derive(Component)]
pub struct Enemy {
    pub xp_drop: f32,
    pub death_processed: bool,
}

#[derive(Resource)]
pub struct AttackCooldown(pub Timer);

#[derive(Message)]
pub struct DamageEnemy {
    pub target: Entity,
    pub amount: f32,
}

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

pub fn damage_enemies(
    mut commands: Commands,
    mut damage_messages: MessageReader<DamageEnemy>,
    mut message_writer: MessageWriter<GainXp>,
    mut stats_query: Query<(&mut BaseStats, &mut Enemy)>,
) {
    for message in damage_messages.read() {
        if let Ok((mut stats, mut enemy)) = stats_query.get_mut(message.target) {
            if enemy.death_processed {
                continue;
            }
            println!("damage taken: {}", message.amount);
            stats.health.take_damage(message.amount);

            if stats.health.is_dead() {
                commands.entity(message.target).despawn();
                enemy.death_processed = true;
                message_writer.write(GainXp {
                    amount: enemy.xp_drop,
                });
            }
        }
    }
}
