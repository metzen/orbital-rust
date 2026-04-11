use bevy::camera::primitives::Aabb;
use bevy::color::palettes::css::BLACK;
use bevy::prelude::*;
use big_space::floating_origins::BigSpace;
use big_space::grid::Grid;
use big_space::grid::cell::CellCoord;

use crate::hud;
use crate::physics::{Orbit, RigidBody, SatelliteOf};
use crate::vessel::Vessel;

pub(super) struct OrbitInfoPlugin;

impl Plugin for OrbitInfoPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup);
        app.add_systems(FixedUpdate, update);
    }
}

#[derive(Component)]
pub struct ApoapsisText;

#[derive(Component)]
pub struct PeriapsisText;

fn setup(mut commands: Commands) {
    use crate::hud::TextFontExt;
    commands.spawn((
        Name::new("Orbit info"),
        Node {
            position_type: PositionType::Absolute,
            left: Val::Px(145.0),
            bottom: Val::Px(20.0),
            border: hud::BORDER,
            border_radius: BorderRadius::all(Val::Px(3.0)),
            padding: UiRect::axes(Val::Px(10.0), Val::Px(6.0)),
            flex_direction: FlexDirection::Column,
            ..default()
        },
        hud::BORDER_COLOR,
        BackgroundColor::from(BLACK),
        Outline::new(Val::Px(1.0), Val::Px(0.0), Color::from(BLACK)),
        children![
            (
                ApoapsisText,
                Node {
                    column_gap: Val::Px(5.0),
                    ..default()
                },
                Text::default(),
                children![
                    (
                        TextSpan::new("AP "),
                        TextColor::from(Color::srgb(0.643, 0.427, 0.518)),
                        TextFont::ui_default(),
                    ),
                    (TextSpan::new("000,000"), TextFont::ui_default()),
                    (TextSpan::new(" "), TextFont::ui_default()),
                    (
                        TextSpan::new("m"),
                        TextColor::from(Color::srgb(0.718, 0.588, 0.376)),
                        TextFont::ui_default(),
                    ),
                    (TextSpan::new(" in "), TextFont::ui_default()),
                    (
                        TextSpan::new("T-"),
                        TextColor::from(Color::srgb(0.718, 0.588, 0.376)),
                        TextFont::ui_default(),
                    ),
                    (TextSpan::new("00"), TextFont::ui_default()),
                    (
                        TextSpan::new(":"),
                        TextColor::from(Color::srgb(0.718, 0.588, 0.376)),
                        TextFont::ui_default(),
                    ),
                    (TextSpan::new("00"), TextFont::ui_default()),
                    (
                        TextSpan::new(":"),
                        TextColor::from(Color::srgb(0.718, 0.588, 0.376)),
                        TextFont::ui_default(),
                    ),
                    (TextSpan::new("00"), TextFont::ui_default()),
                ],
            ),
            (
                PeriapsisText,
                Node {
                    column_gap: Val::Px(5.0),
                    ..default()
                },
                Text::default(),
                children![
                    (
                        TextSpan::new("PE "),
                        TextColor::from(Color::srgb(0.125, 0.506, 0.63)),
                        TextFont::ui_default(),
                    ),
                    (TextSpan::new("000000"), TextFont::ui_default()),
                    (TextSpan::new(" "), TextFont::ui_default()),
                    (
                        TextSpan::new("m"),
                        TextColor::from(Color::srgb(0.718, 0.588, 0.376)),
                        TextFont::ui_default(),
                    ),
                    (TextSpan::new(" in "), TextFont::ui_default()),
                    (
                        TextSpan::new("T-"),
                        TextColor::from(Color::srgb(0.718, 0.588, 0.376)),
                        TextFont::ui_default(),
                    ),
                    (TextSpan::new("00"), TextFont::ui_default()),
                    (
                        TextSpan::new(":"),
                        TextColor::from(Color::srgb(0.718, 0.588, 0.376)),
                        TextFont::ui_default(),
                    ),
                    (TextSpan::new("00"), TextFont::ui_default()),
                    (
                        TextSpan::new(":"),
                        TextColor::from(Color::srgb(0.718, 0.588, 0.376)),
                        TextFont::ui_default(),
                    ),
                    (TextSpan::new("00"), TextFont::ui_default()),
                ]
            ),
        ],
    ));
}

fn update(
    ap_text: Single<Entity, With<ApoapsisText>>,
    pe_text: Single<Entity, With<PeriapsisText>>,
    vessels: Query<(&Vessel, &CellCoord, &Transform, &RigidBody, &SatelliteOf)>,
    primary_query: Query<(&CellCoord, &Transform, &RigidBody, &Aabb)>,
    mut writer: TextUiWriter,
    grid: Single<&Grid, With<BigSpace>>,
) {
    for (vessel, grid_cell, transform, rigidbody, satellite_of) in &vessels {
        if vessel.controlled
            && let Ok((primary_grid_cell, primary_transform, primary_rigidbody, primary_aabb)) =
                primary_query.get(satellite_of.primary())
        {
            let orbit = Orbit::new(
                (grid.grid_position_double(grid_cell, transform)
                    - grid.grid_position_double(primary_grid_cell, primary_transform))
                .xy(),
                (rigidbody.velocity - primary_rigidbody.velocity)
                    .xy()
                    .as_dvec2(),
                primary_rigidbody.mass,
                rigidbody.mass,
            );

            let ap = orbit.apoapsis - primary_aabb.half_extents.y as f64;
            let (humanized_ap, ap_units) = hud::humanize_distance(ap);
            *writer.text(*ap_text, 2) = format!("{humanized_ap:>7.0}");
            // *writer.text(*ap_text, 2) = format!(
            //     "{:>7.*}",
            //     usize::saturating_sub(4, humanized_ap.log10() as usize),
            //     humanized_ap
            // );
            *writer.text(*ap_text, 4) = ap_units;
            let time_to_ap = orbit.time_until_apoapsis().as_secs();
            *writer.text(*ap_text, 7) = format!("{:02}", time_to_ap / 60 / 60);
            *writer.text(*ap_text, 9) = format!("{:02}", time_to_ap / 60 % 60);
            *writer.text(*ap_text, 11) = format!("{:02}", time_to_ap % 60);

            let pe = orbit.periapsis - primary_aabb.half_extents.y as f64;
            let (humanized_pe, pe_units) = hud::humanize_distance(pe);
            *writer.text(*pe_text, 2) = format!("{humanized_pe:>7.0}");
            *writer.text(*pe_text, 4) = pe_units;
            let time_to_pe = orbit.time_to_periapsis().as_secs();
            *writer.text(*pe_text, 7) = format!("{:02}", time_to_pe / 60 / 60);
            *writer.text(*pe_text, 9) = format!("{:02}", time_to_pe / 60 % 60);
            *writer.text(*pe_text, 11) = format!("{:02}", time_to_pe % 60);
            break;
        }
    }
}
