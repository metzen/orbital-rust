use bevy::prelude::*;

const TIME_WARPS: [f32; 15] = [
    1.0, 2.0, 3.0, 4.0, 10.0, 50.0, 100.0, 500.0, 1e3, 1e4, 1e5, 1e6, 1e7, 1e8, 1e9,
];

pub struct TimeWarpPlugin;

impl Plugin for TimeWarpPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(TimeWarp { value: 1.0 });
        app.add_systems(Update, timewarp_control);
    }
}
// #[reflect(Resource, Default)]
#[derive(Resource, Debug, Copy, Clone, Reflect)]
pub struct TimeWarp {
    pub value: f32,
}

pub fn timewarp_control(keyboard_input: Res<ButtonInput<KeyCode>>, mut timewarp: ResMut<TimeWarp>) {
    if keyboard_input.just_pressed(KeyCode::Period) {
        let relative_speed = (*timewarp).value;
        let idx = TIME_WARPS.iter().position(|&i| i == relative_speed);
        if idx.unwrap() < TIME_WARPS.len() - 1 {
            let new_time_warp = TIME_WARPS[(idx.unwrap() + 1).min(TIME_WARPS.len())];
            info!("Setting time warp: {}", new_time_warp);
            timewarp.value = new_time_warp;
        }
    }
    if keyboard_input.just_pressed(KeyCode::Comma) {
        let relative_speed = timewarp.value;
        let idx = TIME_WARPS.iter().position(|&i| i == relative_speed);
        if idx.unwrap() > 0 {
            let new_time_warp = TIME_WARPS[(idx.unwrap() - 1).max(0)];
            info!("Setting time warp: {}", new_time_warp);
            timewarp.value = new_time_warp;
        }
    }
}
