use std::time::Duration;

use bevy::prelude::*;
use leafwing_input_manager::{
    Actionlike,
    plugin::InputManagerPlugin,
    prelude::{ActionState, InputMap},
};

use crate::{
    vessel::Vessel,
};

pub const TIME_WARPS: [f32; 15] = [
    1.0, 2.0, 3.0, 4.0, 10.0, 50.0, 100.0, 500.0, 1e3, 1e4, 1e5, 1e6, 1e7, 1e8, 1e9,
];

pub struct TimeWarpPlugin;

impl Plugin for TimeWarpPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(InputManagerPlugin::<TimeWarpAction>::default());
        app.init_resource::<ActionState<TimeWarpAction>>();
        app.insert_resource(TimeWarpAction::default_input_map());
        app.insert_resource(TimeWarp { value: 1.0 });
        app.add_systems(Startup, setup_timewarp);
        app.add_systems(Update, timewarp_control);
    }
}
// #[reflect(Resource, Default)]
#[derive(Resource, Debug, Copy, Clone, Reflect)]
pub struct TimeWarp {
    pub value: f32,
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

fn timewarp_shift_from_action_state(action_state: &Res<ActionState<TimeWarpAction>>) -> i8 {
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
) {
    if action_state.just_pressed(&TimeWarpAction::ToggleTimewarpPause) {
        match virtual_time.is_paused() {
            true => virtual_time.unpause(),
            false => virtual_time.pause(),
        }
    }
    let timewarp_shift = timewarp_shift_from_action_state(&action_state);
    if timewarp_shift != 0 {
        let relative_speed = timewarp.value;
        let idx = TIME_WARPS
            .iter()
            .position(|&i| i == relative_speed)
            .unwrap();
        let new_timewarp =
            TIME_WARPS[(idx as i8 + timewarp_shift).clamp(0, TIME_WARPS.len() as i8 - 1) as usize];
        if new_timewarp > 4.0 && vessels.iter().any(|v| v.throttle > 0.0) {
            info!("Timewarp limited to 4x while performing burn");
        } else {
            timewarp.value = new_timewarp;
            virtual_time.set_relative_speed(timewarp.value);
            fixed_time.set_timestep_seconds(virtual_time.relative_speed_f64() / 64.0);
        }
    }
}
