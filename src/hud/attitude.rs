use bevy::color::palettes::css::BLACK;
use bevy::prelude::*;

use crate::hud::{BORDER, BORDER_COLOR};

pub(super) struct AttitudePlugin;

impl Plugin for AttitudePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup);
    }
}

fn setup(mut commands: Commands) {
    commands.spawn((
        Name::new("Attitude widget"),
        Node {
            position_type: PositionType::Absolute,
            bottom: px(80.0),
            left: px(156.0),
            width: px(184.0),
            height: px(184.0),
            border: BORDER,
            border_radius: BorderRadius::all(Val::Percent(50.0)),
            padding: UiRect::all(px(5.0)),
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::End,
            row_gap: px(14.0),
            ..default()
        },
        BORDER_COLOR,
        BackgroundColor::from(BLACK),
        Outline::new(px(1.0), px(0.0), Color::from(BLACK)),
    ));
}
