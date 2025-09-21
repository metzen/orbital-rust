use bevy::prelude::*;
use leafwing_input_manager::{
    plugin::InputManagerPlugin,
    prelude::{ActionState, InputMap},
    Actionlike,
};

pub const TIME_WARPS: [f32; 15] = [
    1.0, 2.0, 3.0, 4.0, 10.0, 50.0, 100.0, 500.0, 1e3, 1e4, 1e5, 1e6, 1e7, 1e8, 1e9,
];

pub struct TimeWarpPlugin;

impl Plugin for TimeWarpPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(TimeWarp { value: 1.0 });
        app.add_systems(Update, timewarp_control);
        app.add_plugins(InputManagerPlugin::<TimeWarpAction>::default());
        app.init_resource::<ActionState<TimeWarpAction>>();
        app.insert_resource(TimeWarpAction::default_input_map());
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
}

impl TimeWarpAction {
    fn default_input_map() -> InputMap<Self> {
        InputMap::default()
            .with(Self::DecreaseTimewarp, KeyCode::Comma)
            .with(Self::DecreaseTimewarp, GamepadButton::West)
            .with(Self::IncreaseTimewarp, KeyCode::Period)
            .with(Self::IncreaseTimewarp, GamepadButton::East)
    }
}

fn timewarp_control(
    action_state: Res<ActionState<TimeWarpAction>>,
    mut timewarp: ResMut<TimeWarp>,
) {
    if action_state.just_pressed(&TimeWarpAction::IncreaseTimewarp) {
        let relative_speed = (*timewarp).value;
        let idx = TIME_WARPS.iter().position(|&i| i == relative_speed);
        if idx.unwrap() < TIME_WARPS.len() - 1 {
            timewarp.value = TIME_WARPS[(idx.unwrap() + 1).min(TIME_WARPS.len())];
        }
    }
    if action_state.just_pressed(&TimeWarpAction::DecreaseTimewarp) {
        let relative_speed = timewarp.value;
        let idx = TIME_WARPS.iter().position(|&i| i == relative_speed);
        if idx.unwrap() > 0 {
            timewarp.value = TIME_WARPS[(idx.unwrap() - 1).max(0)];
        }
    }
}
