use bevy::camera::primitives::Aabb;
use bevy::color::palettes::css::BLACK;
use bevy::prelude::*;
use bevy::text::LineHeight;
use big_space::floating_origins::BigSpace;
use big_space::grid::Grid;
use big_space::grid::cell::CellCoord;

use crate::hud::HudSubject;
use crate::physics::SatelliteOf;

pub(super) struct AltitudePlugin;

impl Plugin for AltitudePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup);
        app.add_systems(FixedUpdate, update);
    }
}

#[derive(Component)]
struct AltitudeText;

#[derive(Component)]
struct AltitudeUnitsText;

fn setup(mut commands: Commands) {
    use crate::hud::TextFontExt;
    let widget_color = Color::srgb(199.0 / 255.0, 70.0 / 255.0, 198.0 / 255.0);
    commands.spawn((
        Name::new("Altitude widget"),
        Node {
            position_type: PositionType::Absolute,
            left: Val::Px(350.0),
            bottom: Val::Px(130.0),
            width: Val::Px(80.0),
            border: UiRect::px(1.0, 1.0, 1.0, 3.0),
            border_radius: BorderRadius::all(Val::Px(3.0)),
            padding: UiRect::all(Val::Px(5.0)),
            flex_direction: FlexDirection::Column,
            row_gap: Val::Px(6.0),
            ..default()
        },
        BackgroundColor::from(BLACK),
        BorderColor::from(widget_color),
        Outline::new(Val::Px(1.0), Val::Px(0.0), Color::from(BLACK)),
        children![
            (
                Node {
                    column_gap: px(2.0),
                    ..default()
                },
                Children::spawn(
                    "SEA LVL"
                        .chars()
                        .map(|char| {
                            (
                                Text::new(char),
                                TextColor::from(widget_color),
                                BackgroundColor::from(Color::srgb(0.153, 0.055, 0.149)),
                                TextFont::ui_default().with_font_size(11.0),
                                LineHeight::RelativeToFont(1.0),
                            )
                        })
                        .collect::<Vec<_>>()
                )
            ),
            (
                Text::default(),
                TextFont::ui_default(),
                children![(
                    TextSpan::default(),
                    TextFont::ui_default().with_font_size(18.0),
                    LineHeight::RelativeToFont(1.0),
                    AltitudeText,
                ),]
            ),
            (
                Node {
                    column_gap: px(2.0),
                    ..default()
                },
                TextFont::ui_default(),
                AltitudeUnitsText,
                Children::spawn(
                    "      "
                        .chars()
                        .map(|_| {
                            (
                                Text::default(),
                                TextColor::from(widget_color),
                                TextFont::ui_default().with_font_size(12.0),
                                LineHeight::RelativeToFont(1.0),
                                BackgroundColor::from(Color::srgb(0.153, 0.055, 0.149)),
                            )
                        })
                        .collect::<Vec<_>>()
                ),
            ),
            (
                Node {
                    position_type: PositionType::Absolute,
                    flex_direction: FlexDirection::Column,
                    right: px(-1.0),
                    top: px(14.0),
                    row_gap: px(2.0),
                    padding: UiRect::vertical(px(1.0)),
                    ..default()
                },
                BackgroundColor::from(Color::BLACK),
                children![
                    (
                        Text::new("A"),
                        TextColor::from(widget_color),
                        TextFont::ui_default(),
                        LineHeight::RelativeToFont(1.0),
                    ),
                    (
                        Text::new("L"),
                        TextColor::from(widget_color),
                        TextFont::ui_default(),
                        LineHeight::RelativeToFont(1.0),
                    ),
                    (
                        Text::new("T"),
                        TextColor::from(widget_color),
                        TextFont::ui_default(),
                        LineHeight::RelativeToFont(1.0),
                    ),
                ],
            ),
        ],
    ));
}

fn update(
    mut text: Single<&mut TextSpan, (With<AltitudeText>, Without<AltitudeUnitsText>)>,
    altitude_units_boxes: Single<&Children, (With<AltitudeUnitsText>, Without<AltitudeText>)>,
    grid: Single<&Grid, With<BigSpace>>,
    subject_query: Query<(&Transform, &CellCoord, &Aabb, &SatelliteOf), With<HudSubject>>,
    primary_body_query: Query<(&Transform, &CellCoord, &Aabb)>,
    mut text_query: Query<&mut Text>,
) {
    if let Ok((subject_transform, subject_grid_cell, subject_aabb, subject_satellite_of)) =
        subject_query.single()
        && let Ok((primary_transform, primary_grid_cell, primary_aabb)) =
            primary_body_query.get(subject_satellite_of.primary())
    {
        let primary_position = grid.grid_position_double(primary_grid_cell, primary_transform);
        let subject_position = grid.grid_position_double(subject_grid_cell, subject_transform);
        let distance = primary_position.distance(subject_position);
        // TODO: Calculate from lowest vertex of mesh instead of the Aabb.
        // TODO: Use CelestialBody.radius after it's defined for all planets.
        let altitude =
            distance - primary_aabb.half_extents.y as f64 - subject_aabb.half_extents.y as f64;
        let (humanized_altitude, units) = super::humanize_distance(altitude);
        text.0 = format!("{humanized_altitude:.0}");
        // TODO: This is super hacky; clean it up.
        let units_chars = units.chars().collect::<Vec<char>>();
        for (i, char_box) in altitude_units_boxes.into_iter().enumerate() {
            if let Ok(mut text) = text_query.get_mut(*char_box) {
                text.0 = if i < units_chars.len() {
                    units_chars[i].into()
                } else {
                    " ".into()
                };
            }
        }
    } else {
        text.0 = String::new();
        // TODO: Fixme.
        // units_text.0 = String::new();
    }
}
