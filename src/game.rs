use bevy::prelude::*;

use crate::abilities::fire_aura;
use crate::abilities::projectile;
use crate::enemy;
use crate::enemy_spawner;
use crate::player;
use crate::player::Experience;
use crate::player::Player;
use crate::resolution;

pub struct GamePlugin;

#[derive(States, Debug, Clone, Copy, Eq, PartialEq, Hash, Default)]
pub enum GameState {
    #[default]
    Playing,
    LevelUp,
    Paused,
    MainMenu,
}

#[derive(Component)]
pub struct Direction(pub Vec3);

#[derive(Component)]
pub struct GameTimerText;

#[derive(Resource)]
pub struct GameTimer(pub Timer);

#[derive(Reflect, Clone)]
pub struct Health {
    pub max: f32,
    pub current: f32,
}

#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct BaseStats {
    pub damage: f32,
    pub speed: f32,
    pub health: Health,
    pub base_damage: f32,
    pub base_speed: f32,
    pub base_health: f32,
}

impl Health {
    pub fn full(max: f32) -> Self {
        Self { max, current: max }
    }

    pub fn is_dead(&self) -> bool {
        self.current <= 0.0
    }

    pub fn take_damage(&mut self, amount: f32) {
        self.current = (self.current - amount).max(0.0);
    }
}

impl BaseStats {
    pub fn new(damage: f32, speed: f32, health: f32) -> Self {
        Self {
            damage,
            speed,
            health: Health::full(health),
            base_damage: damage,
            base_speed: speed,
            base_health: health,
        }
    }
}

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<Player>();
        app.register_type::<Health>();
        app.register_type::<Experience>();
        app.register_type::<BaseStats>();
        app.init_state::<GameState>();
        app.insert_resource(GameTimer(Timer::from_seconds(30.0 * 60.0, TimerMode::Once)));
        app.add_plugins((
            player::PlayerPlugin,
            enemy::EnemeyPlugin,
            projectile::ProjectilePlugin,
            fire_aura::FireAuraPlugin,
            resolution::ResolutionPlugin,
            enemy_spawner::EnemySpawner,
        ));
        app.add_systems(Startup, setup_scene);
        app.add_systems(Update, (update_timer).run_if(in_state(GameState::Playing)));
    }
}

fn setup_scene(mut commands: Commands) {
    commands.spawn(Camera2d { ..default() });
    commands.spawn((
        GameTimerText,
        Text::new("30:00"),
        Node {
            position_type: PositionType::Absolute,
            top: px(5),
            right: px(5),
            ..default()
        },
    ));
}

fn update_timer(
    time: Res<Time>,
    mut game_timer: ResMut<GameTimer>,
    mut text_query: Query<&mut Text, With<GameTimerText>>,
) {
    game_timer.0.tick(time.delta());

    let remaining = game_timer.0.duration().as_secs_f32() - game_timer.0.elapsed_secs();
    let minutes = (remaining / 60.0) as u32;
    let seconds = (remaining % 60.0) as u32;

    if let Ok(mut text) = text_query.single_mut() {
        **text = format!("{:02}:{:02}", minutes, seconds);
    }
}
