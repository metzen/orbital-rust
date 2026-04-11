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
                left: Val::Auto,
                right: Val::Px(20.0),
                bottom: Val::Px(20.0),
                top: Val::Auto,
            },
            border: BORDER,
            border_radius: BorderRadius::all(Val::Px(3.0)),
            padding: UiRect::all(Val::Px(5.0)),
            flex_direction: FlexDirection::Column,
            ..default()
        },
        BORDER_COLOR,
        BackgroundColor::from(BLACK),
        Outline::new(Val::Px(1.0), Val::Px(0.0), Color::from(BLACK)),
        children![(
            Node {
                width: Val::Px(200.0),
                height: Val::Px(50.0),
                border_radius: BorderRadius::all(Val::Px(3.0)),
                ..default()
            },
            BackgroundColor::from(Color::srgb(0.0, 0.8, 0.32)),
        ),],
    ));
}
