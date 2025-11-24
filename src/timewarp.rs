use std::error::Error;
use std::fmt;
use std::time::Duration;

use bevy::prelude::*;
use big_space::grid::Grid;
use big_space::prelude::{BigSpace, CellCoord};
use leafwing_input_manager::Actionlike;
use leafwing_input_manager::common_conditions::action_just_pressed;
use leafwing_input_manager::plugin::InputManagerPlugin;
use leafwing_input_manager::prelude::{ActionState, InputMap};

use crate::audio::SineAudio;
use crate::physics::{Atmosphere, CelestialBody, RigidBody};
use crate::vessel::Vessel;

pub const TIME_WARPS: [f32; 13] = [
    1.0, 2.0, 4.0, 10.0, 50.0, 100.0, 1e3, 1e4, 1e5, 1e6, 1e7, 1e8, 1e9,
];
const TIME_WARP_MAX: f32 = 1e9;

#[derive(Debug)]
struct OutOfRange {
    reason: String,
}

impl Error for OutOfRange {}

impl fmt::Display for OutOfRange {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.reason)
    }
}

#[derive(Resource, Debug, Reflect)]
#[reflect(Resource)]
pub struct TimeWarp {
    pub value: f32,
    pub index: usize,
    pub max_allowed_index: usize,
    pub max_allowed_reason: String,
}

impl TimeWarp {
    fn shift_warp(&mut self, shift: isize) -> Result<usize, OutOfRange> {
        let index = self.index as isize + shift;
        if (0..(self.max_allowed_index + 1) as isize).contains(&index) {
            self.index = index as usize;
            self.value = TIME_WARPS[self.index];
            Ok(self.index)
        } else {
            Err(OutOfRange {
                reason: if index > 0 {
                    self.max_allowed_reason.clone()
                } else {
                    String::new()
                },
            })
        }
    }

    fn increase_warp(&mut self) -> Result<usize, OutOfRange> {
        self.shift_warp(1)
    }

    fn decrease_warp(&mut self) -> Result<usize, OutOfRange> {
        self.shift_warp(-1)
    }

    fn set_max_allowed_warp(&mut self, max_allowed_warp: f32, reason: String) {
        let index = TIME_WARPS.iter().rev().position(|v| *v < max_allowed_warp);
        self.max_allowed_index = TIME_WARPS.len() - index.unwrap_or(0);
        self.max_allowed_reason = reason;
        self.index = self.index.clamp(0, self.max_allowed_index);
        self.value = TIME_WARPS[self.index];
    }
}

impl Default for TimeWarp {
    fn default() -> Self {
        Self {
            value: 1.0,
            index: 0,
            max_allowed_index: 0,
            max_allowed_reason: String::default(),
        }
    }
}

#[derive(Event)]
struct TimeWarpChangeEvent {
    // The index of the new time warp.
    index: usize,
}

impl TimeWarpChangeEvent {
    fn new(index: usize) -> Self {
        Self { index }
    }
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
            .with(Self::ToggleTimewarpPause, GamepadButton::Start)
    }
}

fn setup_timewarp(mut virtual_time: ResMut<Time<Virtual>>) {
    virtual_time.set_max_delta(Duration::MAX);
}

/// Updates the maximum allowed warp factor based on the current game state.
fn update_max_allowed_timewarp(
    mut timewarp: ResMut<TimeWarp>,
    vessels: Query<(&Vessel, &CellCoord, &Transform, &RigidBody)>,
    position_query: Query<(&CellCoord, &Transform, &CelestialBody, &Atmosphere)>,
    grid: Single<&Grid, With<BigSpace>>,
) {
    let (limit, reason) = vessels
        .iter()
        .map(|(vessel, vessel_cell, vessel_transform, rigidbody)| {
            if vessel.throttle > 0.0 {
                (
                    50.0,
                    String::from("Time Warp limited to 50x while vessel burn active"),
                )
            } else if let Some(primary_id) = rigidbody.primary
                && let Ok((
                    primary_cell,
                    primary_transform,
                    primary_celestial_body,
                    primary_atmosphere,
                )) = position_query.get(primary_id)
            {
                let vessel_position = grid.grid_position_double(vessel_cell, vessel_transform);
                let primary_position = grid.grid_position_double(primary_cell, primary_transform);
                let distance = vessel_position.distance(primary_position) as f32;
                let altitude = distance - primary_celestial_body.radius;
                // TODO: This needs to work for bodies with no atmosphere.
                let warp_limits_per_atmosphere_height_factor = [
                    (f32::INFINITY, TIME_WARP_MAX),
                    (8.0, 10_000.0),
                    (6.0, 1_000.0),
                    (4.0, 100.0),
                    (2.0, 50.0),
                    // (1.0, 4.0),
                ];
                let (boundary, limit) = warp_limits_per_atmosphere_height_factor
                    .into_iter()
                    .map(|(f, l)| (f * primary_atmosphere.height, l))
                    .take_while(|(boundary, _)| altitude < *boundary)
                    .last()
                    .unwrap();

                (
                    limit,
                    format!(
                        "Time Warp limited to {:.0}x while vessel altitude below {:.0}m",
                        limit, boundary
                    ),
                )
            } else {
                (TIME_WARP_MAX, String::new())
            }
        })
        .fold((TIME_WARP_MAX, String::new()), |acc, value| {
            if acc.0 < value.0 { acc } else { value }
        });
    timewarp.set_max_allowed_warp(limit, reason);
}

/// Toggles the paused state of the virtual clock.
fn toggle_pause(mut virtual_time: ResMut<Time<Virtual>>) {
    match virtual_time.is_paused() {
        true => virtual_time.unpause(),
        false => virtual_time.pause(),
    }
}

/// Increases the TimeWarp by one step.
fn increase_timewarp(mut timewarp: ResMut<TimeWarp>, mut commands: Commands) {
    match timewarp.increase_warp() {
        Ok(index) => commands.trigger(TimeWarpChangeEvent::new(index)),
        Err(error) => info!("{}", error.reason),
    }
}

/// Decreases the TimeWarp by one step.
fn decrease_timewarp(mut timewarp: ResMut<TimeWarp>, mut commands: Commands) {
    match timewarp.decrease_warp() {
        Ok(index) => commands.trigger(TimeWarpChangeEvent::new(index)),
        Err(error) => info!("{}", error.reason),
    }
}

/// Applies the current TimeWarp settings to the virtual clock.
fn apply_timewarp_settings_to_virtual_clock(
    timewarp: Res<TimeWarp>,
    mut virtual_time: ResMut<Time<Virtual>>,
    mut fixed_time: ResMut<Time<Fixed>>,
) {
    virtual_time.set_relative_speed(timewarp.value);
    fixed_time.set_timestep_seconds(virtual_time.relative_speed_f64() / 64.0);
}

/// Plays a sound when the user changes the warp factor.
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

pub struct TimeWarpPlugin;

impl Plugin for TimeWarpPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(InputManagerPlugin::<TimeWarpAction>::default());
        app.init_resource::<ActionState<TimeWarpAction>>();
        app.init_resource::<TimeWarp>();
        app.insert_resource(TimeWarpAction::default_input_map());
        app.add_systems(Startup, setup_timewarp);
        app.add_systems(FixedUpdate, update_max_allowed_timewarp);
        app.add_systems(
            Update,
            (
                toggle_pause.run_if(action_just_pressed(TimeWarpAction::ToggleTimewarpPause)),
                increase_timewarp.run_if(action_just_pressed(TimeWarpAction::IncreaseTimewarp)),
                decrease_timewarp.run_if(action_just_pressed(TimeWarpAction::DecreaseTimewarp)),
                apply_timewarp_settings_to_virtual_clock,
            )
                .chain(),
        );
        app.add_observer(play_sound_on_timewarp_change);
    }
}
