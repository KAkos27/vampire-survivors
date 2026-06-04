use bevy::prelude::*;
use bevy_rapier2d::{
    dynamics::RigidBody,
    geometry::{ActiveCollisionTypes, ActiveEvents, Collider, Sensor},
};

use crate::{
    enemy::Enemy,
    game::{BaseStats, GameState, GameTimer},
    resolution::Resolution,
};

pub struct EnemySpawner;

const SPAWN_TIME: f32 = 1.0;

impl Plugin for EnemySpawner {
    fn build(&self, app: &mut App) {
        app.insert_resource(SpawnTimer(Timer::from_seconds(
            SPAWN_TIME,
            TimerMode::Repeating,
        )));
        app.add_systems(Update, (spawn_enemy).run_if(in_state(GameState::Playing)));
    }
}

#[derive(Resource)]
pub struct SpawnTimer(pub Timer);

fn spawn_enemy(
    mut commands: Commands,
    mut spawn_timer: ResMut<SpawnTimer>,
    time: Res<Time>,
    game_time: Res<GameTimer>,
    asset_server: Res<AssetServer>,
    resolution: Res<Resolution>,
) {
    if !spawn_timer.0.tick(time.delta()).just_finished() {
        return;
    }

    let elapsed_minutes = game_time.0.elapsed_secs() / 60.0;
    let difficulity = 1.0 + elapsed_minutes * 0.5;

    let alien_texture = asset_server.load("alien.png");
    let sc = resolution.pixel_ratio;

    commands.spawn((
        Enemy {
            xp_drop: 12.0 * difficulity,
            death_processed: false,
        },
        BaseStats::new(25.0 * difficulity, 60.0, 100.0 * difficulity),
        Sprite {
            image: alien_texture.clone(),
            ..Default::default()
        },
        Transform::from_xyz(-100.0, 100.0, 0.0).with_scale(Vec3::new(sc, sc, 1.0)),
        RigidBody::KinematicPositionBased,
        Collider::ball(8.0),
        Sensor,
        ActiveEvents::COLLISION_EVENTS,
        ActiveCollisionTypes::KINEMATIC_KINEMATIC,
    ));
}
