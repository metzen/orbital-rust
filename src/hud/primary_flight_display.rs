use bevy::color::palettes::css::BLACK;
use bevy::prelude::*;

use crate::hud::altitude::{AltitudePlugin, altitude_tape};
use crate::hud::attitude::{AttitudePlugin, attitude_indicator};
use crate::hud::speed::{SpeedPlugin, speed_tape};
use crate::hud::{BORDER, BORDER_COLOR};

pub(super) struct PrimaryFlightDisplayPlugin;

impl Plugin for PrimaryFlightDisplayPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup);
        app.add_plugins((AltitudePlugin, AttitudePlugin, SpeedPlugin));
    }
}

fn setup(mut commands: Commands) {
    commands.spawn((
        Name::new("Primary Flight Display"),
        Node {
            position_type: PositionType::Absolute,
            left: px(100.0),
            bottom: px(80.0),
            border: BORDER,
            border_radius: BorderRadius::all(px(3.0)),
            padding: UiRect::all(px(8.0)),
            column_gap: px(8.0),
            ..default()
        },
        BORDER_COLOR,
        BackgroundColor::from(BLACK),
        Outline::new(px(1.0), px(0.0), Color::from(BLACK)),
        children![speed_tape(), attitude_indicator(), altitude_tape()],
    ));
}
