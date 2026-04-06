use bevy::color::palettes::css::BLACK;
use bevy::prelude::*;

use crate::hud::{TextFontExt, BORDER, BORDER_COLOR};
use crate::timewarp::{TimeWarp, TIME_WARPS};

pub struct TimePlugin;

impl Plugin for TimePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup);
        app.add_systems(FixedUpdate, (update_time, update_time_warp));
    }
}

#[derive(Component)]
struct TimeText;

#[derive(Component)]
struct TimeWarpBoxes;

#[derive(Component)]
struct TimeWarpText;

fn setup(mut commands: Commands) {
    commands.spawn((
        Name::new("Time widget"),
        Node {
            margin: UiRect {
                left: Val::Auto,
                right: Val::Auto,
                bottom: Val::Px(20.0),
                top: Val::Auto,
            },
            border: BORDER,
            border_radius: BorderRadius::all(Val::Px(3.0)),
            padding: UiRect {
                top: Val::Px(8.0),
                right: Val::Px(8.0),
                bottom: Val::Px(2.0),
                left: Val::Px(8.0),
            },
            flex_direction: FlexDirection::Column,
            row_gap: Val::Px(4.0),
            ..default()
        },
        BORDER_COLOR,
        BackgroundColor::from(BLACK),
        Outline::new(Val::Px(1.0), Val::Px(0.0), Color::from(BLACK)),
        children![
            (
                Text::default(),
                TimeText,
                children![
                    (
                        TextSpan::new("T+"),
                        TextFont::ui_default(),
                        TextColor::from(Color::srgb(4.0 / 255.0, 152.0 / 255.0, 255.0)),
                    ),
                    (TextSpan::new("000"), TextFont::ui_default()),
                    (
                        TextSpan::new("y "),
                        TextFont::ui_default(),
                        TextColor::from(Color::srgb(4.0 / 255.0, 152.0 / 255.0, 255.0)),
                    ),
                    (TextSpan::new("000"), TextFont::ui_default()),
                    (
                        TextSpan::new("d "),
                        TextFont::ui_default(),
                        TextColor::from(Color::srgb(4.0 / 255.0, 152.0 / 255.0, 255.0)),
                    ),
                    (TextSpan::new("00"), TextFont::ui_default()),
                    (
                        TextSpan::new(":"),
                        TextFont::ui_default(),
                        TextColor::from(Color::srgb(4.0 / 255.0, 152.0 / 255.0, 255.0)),
                    ),
                    (TextSpan::new("00"), TextFont::ui_default()),
                    (
                        TextSpan::new(":"),
                        TextFont::ui_default(),
                        TextColor::from(Color::srgb(4.0 / 255.0, 152.0 / 255.0, 255.0)),
                    ),
                    (TextSpan::new("00"), TextFont::ui_default()),
                ]
            ),
            (
                Node {
                    column_gap: Val::Px(2.0),
                    ..default()
                },
                TimeWarpBoxes,
                Children::spawn(SpawnWith(move |parent: &mut ChildSpawner| {
                    for _ in TIME_WARPS.into_iter() {
                        parent.spawn((
                            Node {
                                width: Val::Px(20.0),
                                height: Val::Px(16.0),
                                justify_content: JustifyContent::Center,
                                ..default()
                            },
                            BackgroundColor::from(Color::srgb(0.446, 0.471, 0.525)),
                            Children::spawn_one((
                                Text::new(">"),
                                TextColor::BLACK,
                                TextFont::ui_default(),
                            )),
                        ));
                    }
                })),
            ),
            (
                Text::default(),
                TextLayout::new_with_justify(Justify::Center),
                children![
                    (
                        TextSpan::new("TIME.WARP= "),
                        TextFont::ui_default(),
                        TextColor::from(Color::srgb(105.0 / 255.0, 109.0 / 255.0, 255.0)),
                    ),
                    (
                        TextSpan::default(),
                        TimeWarpText,
                        TextFont::ui_default(),
                        TextColor::from(Color::srgb(105.0 / 255.0, 109.0 / 255.0, 255.0)),
                    ),
                    (
                        TextSpan::new("x"),
                        TextFont::ui_default(),
                        TextColor::from(Color::srgb(105.0 / 255.0, 109.0 / 255.0, 255.0)),
                    ),
                ],
            ),
        ],
    ));
}

fn update_time(
    text: Single<Entity, With<TimeText>>,
    time: Res<Time<Virtual>>,
    mut writer: TextUiWriter,
) {
    let elapsed_seconds = time.elapsed().as_secs();
    *writer.text(*text, 2) = format!("{:03}", elapsed_seconds / 60 / 60 / 24 / 365);
    *writer.text(*text, 4) = format!("{:03}", elapsed_seconds / 60 / 60 / 24 % 365);
    *writer.text(*text, 6) = format!("{:02}", elapsed_seconds / 60 / 60 % 24);
    *writer.text(*text, 8) = format!("{:02}", elapsed_seconds / 60 % 60);
    *writer.text(*text, 10) = format!("{:02}", elapsed_seconds % 60);
}

fn update_time_warp(
    time_warp: Res<TimeWarp>,
    mut text: Single<&mut TextSpan, With<TimeWarpText>>,
    boxes: Single<&Children, With<TimeWarpBoxes>>,
    mut background_color_query: Query<&mut BackgroundColor>,
) {
    text.0 = format!("{:.0}", time_warp.value);
    if let Some(timewarp_index) = TIME_WARPS.iter().position(|val| *val == time_warp.value) {
        for (i, timewarp_box_id) in boxes.iter().enumerate() {
            if let Ok(mut bg_color) = background_color_query.get_mut(timewarp_box_id) {
                bg_color.0 = if i <= timewarp_index {
                    Color::srgb(0.027, 0.69, 0.286)
                } else if i <= time_warp.max_allowed_index {
                    Color::srgb(0.439, 0.451, 0.525)
                } else {
                    Color::srgb(0.14, 0.13, 0.16)
                }
            }
        }
    }
}
