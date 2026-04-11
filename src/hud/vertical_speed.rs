use bevy::color::palettes::css::BLACK;
use bevy::math::ops::log10;
use bevy::prelude::*;
use big_space::floating_origins::BigSpace;
use big_space::grid::Grid;
use big_space::prelude::CellCoord;

use crate::hud;
use crate::hud::HudSubject;
use crate::physics::{RigidBody, SatelliteOf};

pub(super) struct VerticalSpeedPlugin;

impl Plugin for VerticalSpeedPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup);
        app.add_systems(FixedUpdate, update);
    }
}

#[derive(Component)]
pub struct VerticalSpeedText;

fn setup(mut commands: Commands) {
    use crate::hud::TextFontExt;
    commands.spawn((
        Name::new("Vertical speed"),
        Node {
            position_type: PositionType::Absolute,
            bottom: Val::Px(20.0),
            left: Val::Px(448.0),
            width: Val::Px(50.0),
            border: hud::BORDER,
            border_radius: BorderRadius::all(Val::Px(3.0)),
            padding: UiRect::all(Val::Px(5.0)),
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::End,
            row_gap: Val::Px(14.0),
            ..default()
        },
        hud::BORDER_COLOR,
        BackgroundColor::from(BLACK),
        Outline::new(Val::Px(1.0), Val::Px(0.0), Color::from(BLACK)),
        children![
            (
                Text::new("+100"),
                TextFont::ui_default(),
                TextColor::from(Color::srgb(105.0 / 255.0, 109.0 / 255.0, 255.0)),
            ),
            (
                Text::new("+10"),
                TextFont::ui_default(),
                TextColor::from(Color::srgb(105.0 / 255.0, 109.0 / 255.0, 255.0)),
            ),
            (
                Text::new("0"),
                TextFont::ui_default(),
                TextColor::from(Color::srgb(105.0 / 255.0, 109.0 / 255.0, 255.0)),
            ),
            (
                Text::new("-10"),
                TextFont::ui_default(),
                TextColor::from(Color::srgb(105.0 / 255.0, 109.0 / 255.0, 255.0)),
            ),
            (
                Text::new("-100"),
                TextFont::ui_default(),
                TextColor::from(Color::srgb(105.0 / 255.0, 109.0 / 255.0, 255.0)),
            ),
            (
                Name::new("vertical speed indicator"),
                Node {
                    position_type: PositionType::Absolute,
                    height: Val::Px(20.0),
                    ..default()
                },
                Text::default(),
                TextFont::ui_default(),
                VerticalSpeedText,
            ),
        ],
    ));
}

// TODO: Use this.
// fn symlog_plot(value: f32, max_value: f32, linear_threshold: f32, linear_scale: f32) {
//     let max_plot = log10(max_value);
// }

fn update(
    subject: Single<(&RigidBody, &CellCoord, &Transform, &SatelliteOf), With<HudSubject>>,
    rigidbody_query: Query<(&RigidBody, &CellCoord, &Transform), Without<HudSubject>>,
    mut vertical_speed_ui: Single<(&mut Node, &mut Text), With<VerticalSpeedText>>,
    grid: Single<&Grid, With<BigSpace>>,
) {
    let (subject_rigidbody, subject_gridcell, subject_transform, subject_satellite_of) = *subject;
    if let Ok(primary) = rigidbody_query.get(subject_satellite_of.primary()) {
        // TODO: direction from center to center.
        let (primary_rigidbody, primary_gridcell, primary_transform) = primary;
        let relative_velocity = subject_rigidbody.velocity - primary_rigidbody.velocity;
        let relative_position = grid.grid_position_double(subject_gridcell, subject_transform)
            - grid.grid_position_double(primary_gridcell, primary_transform);
        let vertical_speed = relative_velocity.dot(relative_position.normalize().as_vec3());

        let vertical_speed_text = if (-10.0..10.0).contains(&vertical_speed) {
            format!("{vertical_speed:+.2}")
        } else if (-100.0..100.0).contains(&vertical_speed) {
            format!("{vertical_speed:+.1}")
        } else {
            format!("{vertical_speed:+.0}")
        };
        vertical_speed_ui.1.0 = vertical_speed_text;
        let position = if vertical_speed.abs() > 10.0 {
            // Log scale.
            (log10(vertical_speed.abs()) / 2.0) * 50.0
        } else {
            // Linear scale.
            (vertical_speed.abs() / 10.0) * 25.0
        } * vertical_speed.signum();
        vertical_speed_ui.0.bottom = Val::Percent((50.0 + position).clamp(0.0, 100.0));
    }
}
