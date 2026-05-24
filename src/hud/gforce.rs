use bevy::color::palettes::css::BLACK;
use bevy::prelude::*;

use crate::hud;
use crate::hud::HudSubject;
use crate::physics::{RigidBody, SatelliteOf};

const WIDGET_PADDING: UiRect = UiRect::axes(Val::Px(1.0), Val::Px(5.0));

/// Force per unit mass due to gravity at sea level on Earth, in N/kg.
const G_FORCE: f32 = 9.80665;

pub(super) struct GForcePlugin;

impl Plugin for GForcePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup);
        app.add_systems(Update, update);
    }
}

/// Marker for g-force UI Node entity.
#[derive(Component)]
#[require(Name::new("g-force"))]
struct GForce;

#[derive(Component)]
#[require(Name::new("g-force needle"))]
pub struct GForceNeedle;

fn gauge_graduation(label: &str, border_color: Color) -> impl Bundle {
    use crate::hud::TextFontExt;
    (
        Node {
            border: UiRect::right(px(3.0)),
            padding: UiRect::horizontal(px(5.0)),
            ..default()
        },
        BorderColor::from(border_color),
        children![(
            Text::new(label),
            TextFont::ui_default(),
            TextColor::from(Color::srgb(0.41, 0.43, 1.0)),
        )],
    )
}

fn setup(mut commands: Commands) {
    use crate::hud::TextFontExt;
    const COLOR_SAFE: Color = Color::srgb(0.33, 0.76, 0.47);
    const COLOR_CAUTION: Color = Color::srgb(0.9, 0.86, 0.6);
    const COLOR_DANGER: Color = Color::srgb(0.78, 0.3, 0.31);
    commands.spawn((
        GForce,
        Node {
            position_type: PositionType::Absolute,
            bottom: px(20.0),
            left: px(510.0),
            width: px(50.0),
            border: hud::BORDER,
            border_radius: BorderRadius::all(px(3.0)),
            padding: WIDGET_PADDING,
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::End,
            ..default()
        },
        hud::BORDER_COLOR,
        BackgroundColor::from(BLACK),
        Outline::new(px(1.0), px(0.0), Color::from(BLACK)),
        children![
            gauge_graduation(" +10", COLOR_DANGER),
            gauge_graduation("    ", COLOR_DANGER),
            gauge_graduation("  +5", COLOR_CAUTION),
            gauge_graduation("    ", COLOR_SAFE),
            gauge_graduation("   0", COLOR_SAFE),
            gauge_graduation("    ", COLOR_SAFE),
            gauge_graduation("  -5", COLOR_CAUTION),
            gauge_graduation("    ", COLOR_DANGER),
            gauge_graduation(" -10", COLOR_DANGER),
            (
                GForceNeedle,
                Node {
                    position_type: PositionType::Absolute,
                    padding: UiRect::right(px(8.0)),
                    ..default()
                },
                Text::default(),
                TextFont::ui_default(),
            ),
        ],
    ));
}

fn update(
    subject: Single<(&RigidBody, &SatelliteOf, &GlobalTransform), With<HudSubject>>,
    transform_query: Query<&GlobalTransform, Without<HudSubject>>,
    widget_node: Single<&ComputedNode, With<GForce>>,
    mut needle: Single<(&mut Node, &ComputedNode, &mut Text), With<GForceNeedle>>,
) {
    let (subject_rigidbody, subject_satellite_of, subject_transform) = *subject;
    let primary_transform = transform_query.get(subject_satellite_of.primary()).unwrap();
    let direction = subject_transform.translation() - primary_transform.translation();
    // TODO: Fix this dummy implementation.
    let inertial_frame_acceleration = direction.normalize() * G_FORCE;
    let acceleration_relative_to_inertial_frame =
        subject_rigidbody.acceleration + inertial_frame_acceleration;
    let acceleration_relative_to_subject_orientation =
        acceleration_relative_to_inertial_frame.dot(subject_transform.rotation() * Vec3::Y);
    let g_force = acceleration_relative_to_inertial_frame.length() / G_FORCE
        * acceleration_relative_to_subject_orientation.signum();
    needle.2.0 = format!("{g_force:.1}");

    if let Val::Px(padding_top) = WIDGET_PADDING.top
        && let Val::Px(padding_bottom) = WIDGET_PADDING.bottom
    {
        let gauge_height = widget_node.content_size.y - padding_top - padding_bottom - 10.0;
        let half_needle_height = needle.1.size.y * 0.5;
        let offset = (g_force / 10.0).clamp(-1.0, 1.0) * 0.5;
        needle.0.bottom =
            px(((gauge_height - half_needle_height) * (0.5 + offset)) + padding_bottom);
    }
}
