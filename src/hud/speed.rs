use bevy::color::palettes::css::BLACK;
use bevy::prelude::*;
use bevy::text::LineHeight;

use crate::hud::HudSubject;
use crate::physics::{RigidBody, SatelliteOf};

const SPEED_TAPE_COLOR: Color = Color::srgb(0.44, 0.44, 0.2);

pub(super) struct SpeedPlugin;

impl Plugin for SpeedPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(FixedUpdate, update);
    }
}

#[derive(Component)]
struct SpeedText;

fn speed_indicator() -> impl Bundle {
    use crate::hud::TextFontExt;
    let widget_color = Color::srgb(213.0 / 255.0, 175.0 / 255.0, 3.0 / 255.0);
    (
        Name::new("Speed widget"),
        Node {
            position_type: PositionType::Absolute,
            right: px(10.0),
            top: auto(),
            bottom: auto(),
            border: UiRect::px(1.0, 1.0, 1.0, 3.0),
            border_radius: BorderRadius::all(px(3.0)),
            padding: UiRect::all(px(5.0)),
            flex_direction: FlexDirection::Column,
            width: px(80.0),
            row_gap: px(6.0),
            ..default()
        },
        BorderColor::from(widget_color),
        BackgroundColor::from(BLACK),
        Outline::new(px(1.0), px(0.0), Color::from(BLACK)),
        ZIndex(1),
        children![
            (
                Node {
                    column_gap: px(2.0),
                    justify_content: JustifyContent::End,
                    ..default()
                },
                Children::spawn(
                    "SURFACE"
                        .chars()
                        .map(|char| {
                            (
                                Text::new(char),
                                TextColor::from(widget_color),
                                BackgroundColor::from(Color::srgb(44.0 / 255.0, 35.0 / 255.0, 0.0)),
                                TextFont::ui_default().with_font_size(11.0),
                                LineHeight::RelativeToFont(1.0),
                            )
                        })
                        .collect::<Vec<_>>()
                )
            ),
            (
                Text::default(),
                TextLayout::new_with_justify(Justify::Right),
                TextFont::ui_default(),
                children![(
                    TextSpan::default(),
                    TextFont::ui_default().with_font_size(18.0),
                    LineHeight::RelativeToFont(1.0),
                    SpeedText,
                )]
            ),
            (
                Node {
                    column_gap: px(2.0),
                    justify_content: JustifyContent::End,
                    ..default()
                },
                Children::spawn(
                    "   m/s"
                        .chars()
                        .map(|char| {
                            (
                                Text::new(char),
                                TextLayout::new_with_justify(Justify::Right),
                                TextColor::from(widget_color),
                                BackgroundColor::from(Color::srgb(44.0 / 255.0, 35.0 / 255.0, 0.0)),
                                TextFont::ui_default(),
                                LineHeight::RelativeToFont(1.0),
                            )
                        })
                        .collect::<Vec<_>>()
                ),
            ),
            (
                Node {
                    position_type: PositionType::Absolute,
                    flex_direction: FlexDirection::Column,
                    left: px(-1.0),
                    top: px(14.0),
                    row_gap: px(2.0),
                    padding: UiRect::vertical(px(1.0)),
                    ..default()
                },
                BackgroundColor::from(Color::BLACK),
                children![
                    (
                        Text::new("S"),
                        TextColor::from(widget_color),
                        TextFont::ui_default(),
                        LineHeight::RelativeToFont(1.0),
                    ),
                    (
                        Text::new("P"),
                        TextColor::from(widget_color),
                        TextFont::ui_default(),
                        LineHeight::RelativeToFont(1.0),
                    ),
                    (
                        Text::new("D"),
                        TextColor::from(widget_color),
                        TextFont::ui_default(),
                        LineHeight::RelativeToFont(1.0),
                    ),
                ],
            ),
        ],
    )
}

fn tape_graduation(label: impl Into<String>) -> impl Bundle {
    (
        Text::new(label),
        TextColor::from(SPEED_TAPE_COLOR),
        TextFont::from_font_size(12.0),
    )
}

pub(super) fn speed_tape() -> impl Bundle {
    (
        Name::new("Speed tape"),
        Node {
            width: px(50.0),
            height: px(172.0),
            border: UiRect::px(0.0, 1.0, 1.0, 1.0),
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::End,
            justify_content: JustifyContent::SpaceEvenly,
            ..default()
        },
        BackgroundColor::from(Color::srgb(0.09, 0.09, 0.09)),
        BorderColor::from(SPEED_TAPE_COLOR),
        children![
            speed_indicator(),
            tape_graduation(" 20 --"),
            tape_graduation("    --"),
            tape_graduation(" 10 --"),
            tape_graduation("    --"),
            tape_graduation("  0 --"),
            tape_graduation("    --"),
            tape_graduation("-10 --"),
            tape_graduation("    --"),
            tape_graduation("-20 --"),
        ],
    )
}

fn update(
    mut text: Single<&mut TextSpan, With<SpeedText>>,
    subject_rigidbody: Query<(&RigidBody, &SatelliteOf), With<HudSubject>>,
    primary_body_query: Query<&RigidBody>,
) {
    if let Ok((rigidbody, satellite_of)) = subject_rigidbody.single()
        && let Ok(primary_body) = primary_body_query.get(satellite_of.primary())
    {
        let relative_speed = (rigidbody.velocity - primary_body.velocity).length();
        text.0 = format!(
            "{:.*}",
            // Show one digit of decimal precision when speed is low.
            if relative_speed < 10_000.0 { 1 } else { 0 },
            relative_speed
        );
    } else {
        text.0 = String::new();
    };
}
