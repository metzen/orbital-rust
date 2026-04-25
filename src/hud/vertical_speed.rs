use bevy::color::palettes::css::BLACK;
use bevy::math::ops::log10;
use bevy::prelude::*;
use big_space::floating_origins::BigSpace;
use big_space::grid::Grid;
use big_space::prelude::CellCoord;

use crate::hud;
use crate::hud::HudSubject;
use crate::physics::{RigidBody, SatelliteOf};

const WIDGET_PADDING: UiRect = UiRect::axes(Val::Px(1.0), Val::Px(5.0));

pub(super) struct VerticalSpeedPlugin;

impl Plugin for VerticalSpeedPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup);
        app.add_systems(FixedUpdate, update);
    }
}

/// Marker for Vertical Speed UI Node entity.
#[derive(Component)]
#[require(Name::new("VerticalSpeed"))]
struct VerticalSpeed;

#[derive(Component)]
#[require(Name::new("VerticalSpeedNeedle"))]
pub struct VerticalSpeedNeedle;

fn setup(mut commands: Commands) {
    use crate::hud::TextFontExt;
    commands.spawn((
        VerticalSpeed,
        Node {
            position_type: PositionType::Absolute,
            bottom: px(20.0),
            left: px(448.0),
            width: px(50.0),
            border: hud::BORDER,
            border_radius: BorderRadius::all(px(3.0)),
            padding: WIDGET_PADDING,
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::End,
            // row_gap: px(14.0),
            ..default()
        },
        hud::BORDER_COLOR,
        BackgroundColor::from(BLACK),
        Outline::new(px(1.0), px(0.0), Color::from(BLACK)),
        Children::spawn((
            SpawnIter(
                [
                    ("+100", BorderColor::from(Color::BLACK)),
                    ("    ", BorderColor::from(Color::BLACK)),
                    (" +10", BorderColor::from(Color::BLACK)),
                    ("    ", BorderColor::from(Color::BLACK)),
                    ("   0", BorderColor::from(Color::srgb(0.33, 0.76, 0.47))),
                    ("    ", BorderColor::from(Color::srgb(0.9, 0.86, 0.6))),
                    (" -10", BorderColor::from(Color::srgb(0.9, 0.86, 0.6))),
                    ("    ", BorderColor::from(Color::srgb(0.78, 0.3, 0.31))),
                    ("-100", BorderColor::from(Color::srgb(0.78, 0.3, 0.31))),
                ]
                .into_iter()
                .map(|(label, border_color)| {
                    (
                        Node {
                            border: UiRect::right(px(3.0)),
                            padding: UiRect::horizontal(px(5.0)),
                            ..default()
                        },
                        border_color,
                        children![(
                            Text::new(label),
                            TextFont::ui_default(),
                            TextColor::from(Color::srgb(0.41, 0.43, 1.0)),
                        )],
                    )
                }),
            ),
            Spawn((
                VerticalSpeedNeedle,
                Node {
                    position_type: PositionType::Absolute,
                    padding: UiRect::right(px(8.0)),
                    ..default()
                },
                Text::default(),
                TextFont::ui_default(),
            )),
        )),
    ));
}

// TODO: Use this.
// fn symlog_plot(value: f32, max_value: f32, linear_threshold: f32, linear_scale: f32) {
//     let max_plot = log10(max_value);
// }

fn update(
    subject: Single<(&RigidBody, &CellCoord, &Transform, &SatelliteOf), With<HudSubject>>,
    rigidbody_query: Query<(&RigidBody, &CellCoord, &Transform), Without<HudSubject>>,
    grid: Single<&Grid, With<BigSpace>>,
    widget_node: Single<&ComputedNode, With<VerticalSpeed>>,
    needle: Single<(&mut Node, &ComputedNode, &mut Text), With<VerticalSpeedNeedle>>,
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
        let (mut needle_node, needle_computed_node, mut needle_text) = needle.into_inner();
        let offset = if vertical_speed.abs() > 10.0 {
            // Log scale.
            ((log10(vertical_speed.abs()) / 2.0) * 0.5).clamp(0.0, 0.5)
        } else {
            // Linear scale.
            (vertical_speed.abs() / 10.0) * 0.25
        } * vertical_speed.signum();
        if let Val::Px(padding_top) = WIDGET_PADDING.top
            && let Val::Px(padding_bottom) = WIDGET_PADDING.bottom
        {
            let gauge_height = widget_node.content_size.y - padding_top - padding_bottom - 10.0;
            let half_needle_height = needle_computed_node.size.y * 0.5;
            needle_node.bottom =
                px(((gauge_height - half_needle_height) * (0.5 + offset)) + padding_bottom);
        }
        needle_text.0 = vertical_speed_text;
    }
}
