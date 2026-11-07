use bevy::prelude::*;

use bevy::time::Stopwatch;

use crate::level::{LevelFinished, LevelLoaded, LevelPhase, LevelRestarted};

#[derive(Component)]
struct TimerText;

#[derive(Resource, Default)]
pub struct LevelTimer {
    pub watch: Stopwatch,
    pub last_time: Option<f32>,
    pub best_time: std::collections::HashMap<usize, f32>,
}

#[derive(Resource)]
struct ResultsCountdown(Timer);

pub struct LevelTimerPlugin;
impl Plugin for LevelTimerPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<LevelTimer>()
            .add_systems(Startup, spawn_timer_hud)
            .add_systems(Update, update_timer_hud)
            .add_systems(Update, tick.run_if(in_state(LevelPhase::Playing)))
            .add_systems(Update, (on_level_reset, on_level_finished))
            .add_systems(OnEnter(LevelPhase::Results), start_results_countdown)
            .add_systems(
                Update,
                tick_results_countdown.run_if(in_state(LevelPhase::Results)),
            );
    }
}

fn start_results_countdown(mut commands: Commands) {
    commands.insert_resource(ResultsCountdown(Timer::from_seconds(3.0, TimerMode::Once)));
}

fn tick_results_countdown(
    time: Res<Time>,
    mut countdown: ResMut<ResultsCountdown>,
    mut next: ResMut<NextState<LevelPhase>>,
) {
    if countdown.0.tick(time.delta()).just_finished() {
        next.set(LevelPhase::Playing);
    }
}

fn spawn_timer_hud(mut commands: Commands) {
    commands.spawn((
        TimerText,
        Text::new("00:00.00"),
        TextFont {
            font_size: FontSize::Px(32.0),
            ..default()
        },
        TextColor(Color::WHITE),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(12.0),
            right: Val::Px(16.0),
            ..default()
        },
    ));
}

fn update_timer_hud(timer: Res<LevelTimer>, mut q: Query<&mut Text, With<TimerText>>) {
    let s = timer.watch.elapsed_secs();
    let text = format!("{:02}:{:05.2}", (s / 60.0) as u32, s % 60.0);
    for mut t in &mut q {
        t.0 = text.clone(); // Text is a newtype around String
    }
}

fn on_level_reset(
    mut loaded: MessageReader<LevelLoaded>,
    mut restarted: MessageReader<LevelRestarted>,
    mut timer: ResMut<LevelTimer>,
) {
    let got_any = loaded.read().count() + restarted.read().count() > 0;
    if got_any {
        timer.watch.reset();
        timer.watch.unpause();
    }
}

fn on_level_finished(mut finished: MessageReader<LevelFinished>, mut t: ResMut<LevelTimer>) {
    for msg in finished.read() {
        t.watch.pause();
        let time = t.watch.elapsed_secs();
        t.last_time = Some(time);
        let best = t.best_time.entry(msg.index).or_insert(time);
        *best = best.min(time);
    }
}

fn tick(time: Res<Time>, mut t: ResMut<LevelTimer>) {
    t.watch.tick(time.delta());
}
