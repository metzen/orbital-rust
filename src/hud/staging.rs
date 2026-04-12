use bevy::color::palettes::css::BLACK;
use bevy::prelude::*;

use crate::hud::{BORDER, BORDER_COLOR};

pub(super) struct StagingPlugin;

impl Plugin for StagingPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup);
    }
}

fn setup(mut commands: Commands) {
    commands.spawn((
        Node {
            margin: UiRect {
                left: auto(),
                right: px(20.0),
                bottom: px(20.0),
                top: auto(),
            },
            border: BORDER,
            border_radius: BorderRadius::all(px(3.0)),
            padding: UiRect::all(px(5.0)),
            flex_direction: FlexDirection::Column,
            ..default()
        },
        BORDER_COLOR,
        BackgroundColor::from(BLACK),
        Outline::new(px(1.0), px(0.0), Color::from(BLACK)),
        children![(
            Node {
                width: px(200.0),
                height: px(50.0),
                border_radius: BorderRadius::all(px(3.0)),
                ..default()
            },
            BackgroundColor::from(Color::srgb(0.0, 0.8, 0.32)),
        ),],
    ));
}
