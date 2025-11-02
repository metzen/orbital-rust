use std::time::Duration;

use bevy::prelude::*;
use leafwing_input_manager::Actionlike;
use leafwing_input_manager::plugin::InputManagerPlugin;
use leafwing_input_manager::prelude::{ActionState, InputMap};

use crate::audio::SineAudio;
use crate::vessel::Vessel;

pub const TIME_WARPS: [f32; 15] = [
    1.0, 2.0, 3.0, 4.0, 10.0, 50.0, 100.0, 500.0, 1e3, 1e4, 1e5, 1e6, 1e7, 1e8, 1e9,
];

pub struct TimeWarpPlugin;

impl Plugin for TimeWarpPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(InputManagerPlugin::<TimeWarpAction>::default());
        app.init_resource::<ActionState<TimeWarpAction>>();
        app.init_resource::<TimeWarp>();
        app.insert_resource(TimeWarpAction::default_input_map());
        app.add_systems(Startup, setup_timewarp);
        app.add_systems(Update, timewarp_control);
        app.add_observer(play_sound_on_timewarp_change);
    }
}

#[derive(Resource, Debug, Copy, Clone, Reflect)]
#[reflect(Resource)]
pub struct TimeWarp {
    pub value: f32,
    pub index: isize,
}

impl Default for TimeWarp {
    fn default() -> Self {
        Self {
            value: 1.0,
            index: 0,
        }
    }
}

#[derive(Event)]
struct TimeWarpChangeEvent {
    // The index of the new time warp.
    index: isize,
}

#[derive(Actionlike, PartialEq, Eq, Clone, Copy, Hash, Debug, Reflect)]
enum TimeWarpAction {
    DecreaseTimewarp,
    IncreaseTimewarp,
    ToggleTimewarpPause,
}

impl TimeWarpAction {
    fn default_input_map() -> InputMap<Self> {
        InputMap::default()
            .with(Self::DecreaseTimewarp, KeyCode::Comma)
            .with(Self::DecreaseTimewarp, GamepadButton::West)
            .with(Self::IncreaseTimewarp, KeyCode::Period)
            .with(Self::IncreaseTimewarp, GamepadButton::East)
            .with(Self::ToggleTimewarpPause, KeyCode::Slash)
    }
}

fn setup_timewarp(mut virtual_time: ResMut<Time<Virtual>>) {
    virtual_time.set_max_delta(Duration::MAX);
}

fn timewarp_shift_from_action_state(action_state: &Res<ActionState<TimeWarpAction>>) -> isize {
    if action_state.just_pressed(&TimeWarpAction::IncreaseTimewarp) {
        1
    } else if action_state.just_pressed(&TimeWarpAction::DecreaseTimewarp) {
        -1
    } else {
        0
    }
}

fn timewarp_control(
    action_state: Res<ActionState<TimeWarpAction>>,
    mut timewarp: ResMut<TimeWarp>,
    mut virtual_time: ResMut<Time<Virtual>>,
    mut fixed_time: ResMut<Time<Fixed>>,
    vessels: Query<&Vessel>,
    mut commands: Commands,
) {
    if action_state.just_pressed(&TimeWarpAction::ToggleTimewarpPause) {
        match virtual_time.is_paused() {
            true => virtual_time.unpause(),
            false => virtual_time.pause(),
        }
    }
    let timewarp_shift = timewarp_shift_from_action_state(&action_state);
    if timewarp_shift != 0 {
        let new_index = (timewarp.index + timewarp_shift).clamp(0, TIME_WARPS.len() as isize - 1);
        let new_timewarp = TIME_WARPS[new_index as usize];
        if new_timewarp > 4.0 && vessels.iter().any(|v| v.throttle > 0.0) {
            info!("Timewarp limited to 4x while performing burn");
        } else {
            timewarp.value = new_timewarp;
            timewarp.index = new_index;
            virtual_time.set_relative_speed(timewarp.value);
            fixed_time.set_timestep_seconds(virtual_time.relative_speed_f64() / 64.0);
            commands.trigger(TimeWarpChangeEvent { index: new_index });
        }
    }
}

fn play_sound_on_timewarp_change(
    event: On<TimeWarpChangeEvent>,
    mut commands: Commands,
    mut assets: ResMut<Assets<SineAudio>>,
) {
    commands.spawn((
        AudioPlayer(assets.add(SineAudio::new(550.0 + event.index as f32 * 100.0))),
        PlaybackSettings::DESPAWN.with_duration(Duration::from_millis(30)),
    ));
}
