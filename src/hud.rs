use bevy::prelude::*;
use leafwing_input_manager::Actionlike;
use leafwing_input_manager::common_conditions::action_just_pressed;
use leafwing_input_manager::plugin::InputManagerPlugin;
use leafwing_input_manager::prelude::{ActionState, InputMap};

use crate::camera::{Autofollow, InGameCamera};
use crate::hud::heading::HeadingPlugin;
use crate::hud::hovered::HoveredPlugin;
use crate::hud::orbit_info::OrbitInfoPlugin;
use crate::hud::orbits::OrbitGizmoPlugin;
use crate::hud::primary_flight_display::PrimaryFlightDisplayPlugin;
use crate::hud::sas_selector::SasSelectorPlugin;
use crate::hud::staging::StagingPlugin;
use crate::hud::subject::SubjectPlugin;
use crate::hud::throttle::ThrottlePlugin;
use crate::hud::time::TimePlugin;
use crate::hud::vertical_speed::VerticalSpeedPlugin;
use crate::vessel::Vessel;

mod altitude;
mod attitude;
mod heading;
mod hovered;
mod orbit_info;
mod orbits;
mod primary_flight_display;
mod sas_selector;
mod speed;
mod staging;
mod subject;
mod throttle;
mod time;
mod vertical_speed;

pub struct HudPlugin;

impl Plugin for HudPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                next_vessel.run_if(action_just_pressed(HudAction::NextVessel)),
                previous_vessel.run_if(action_just_pressed(HudAction::PreviousVessel)),
            ),
        );
        app.add_plugins((
            HeadingPlugin,
            HoveredPlugin,
            InputManagerPlugin::<HudAction>::default(),
            OrbitInfoPlugin,
            OrbitGizmoPlugin,
            PrimaryFlightDisplayPlugin,
            SasSelectorPlugin,
            StagingPlugin,
            SubjectPlugin,
            ThrottlePlugin,
            TimePlugin,
            VerticalSpeedPlugin,
        ));
        app.init_resource::<ActionState<HudAction>>();
        app.insert_resource(HudAction::default_input_map());
    }
}

#[derive(Component)]
pub struct HudSubject;

const BORDER: UiRect = UiRect::px(1.0, 1.0, 1.0, 1.0);
// TODO: Use old value? BorderColor::from(Color::srgb(105.0 / 255.0, 109.0 / 255.0, 255.0))
const BORDER_COLOR: BorderColor = BorderColor {
    top: Color::srgb(0.184, 0.188, 0.251),
    right: Color::srgb(0.184, 0.188, 0.251),
    bottom: Color::srgb(0.298, 0.310, 0.478),
    left: Color::srgb(0.184, 0.188, 0.251),
};

#[extension(trait TextFontExt)]
impl TextFont {
    fn ui_default() -> Self {
        Self {
            font_size: 12.0,
            ..default()
        }
    }
}

#[derive(Actionlike, PartialEq, Eq, Clone, Copy, Hash, Debug, Reflect)]
enum HudAction {
    NextVessel,
    PreviousVessel,
}

impl HudAction {
    fn default_input_map() -> InputMap<Self> {
        InputMap::default()
            .with(Self::NextVessel, KeyCode::BracketRight)
            .with(Self::NextVessel, GamepadButton::DPadRight)
            .with(Self::PreviousVessel, KeyCode::BracketLeft)
            .with(Self::PreviousVessel, GamepadButton::DPadLeft)
    }
}

fn change_hud_subject(
    mut commands: Commands,
    subject: Single<Entity, With<HudSubject>>,
    vessel_entities: Query<Entity, With<Vessel>>,
    mut vessels: Query<&mut Vessel>,
    mut autofollow: Single<&mut Autofollow, With<InGameCamera>>,
    modifier: isize,
) {
    let old_subject = *subject;
    let items: Vec<Entity> = vessel_entities.iter().sort::<Entity>().collect();
    let index = items.binary_search(&old_subject).unwrap();
    #[allow(clippy::cast_possible_wrap)]
    let new_index = (index as isize + modifier).rem_euclid(items.len() as isize) as usize;
    let new_subject = items[new_index];
    commands.entity(old_subject).remove::<HudSubject>();
    commands.entity(new_subject).insert(HudSubject);
    vessels.get_mut(old_subject).unwrap().controlled = false;
    vessels.get_mut(new_subject).unwrap().controlled = true;
    autofollow.target = Some(new_subject);
}

fn next_vessel(
    commands: Commands,
    subject: Single<Entity, With<HudSubject>>,
    vessels: Query<Entity, With<Vessel>>,
    vessel_query: Query<&mut Vessel>,
    autofollow: Single<&mut Autofollow, With<InGameCamera>>,
) {
    change_hud_subject(commands, subject, vessels, vessel_query, autofollow, 1);
}

fn previous_vessel(
    commands: Commands,
    subject: Single<Entity, With<HudSubject>>,
    vessels: Query<Entity, With<Vessel>>,
    vessel_query: Query<&mut Vessel>,
    autofollow: Single<&mut Autofollow, With<InGameCamera>>,
) {
    change_hud_subject(commands, subject, vessels, vessel_query, autofollow, -1);
}

fn humanize_distance(altitude: f64) -> (f64, String) {
    let (value, units) = match altitude.abs() {
        // AU..INFINITY => (altitude / AU, "au"),
        0.0..1e6 => (altitude, "m "),
        1e6..1e9 => (altitude / 1e3, "km"),
        1e9..1e12 => (altitude / 1e6, "Mm"),
        _ => (altitude / 1e9, "Gm"),
    };
    (value, units.into())
}
