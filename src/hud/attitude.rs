use std::f32::consts::{FRAC_PI_2, PI};

use bevy::camera::visibility::{Layer, RenderLayers};
use bevy::prelude::*;

use crate::hud::HudSubject;
use crate::physics::{RigidBody, SatelliteOf};
use crate::rendering::LayerExt;

const BORDER_RADIUS_TOP: BorderRadius = BorderRadius::top(Val::Px(16.0));
const BORDER_RADIUS_BOTTOM: BorderRadius = BorderRadius::bottom(Val::Px(16.0));

pub(super) struct AttitudePlugin;

impl Plugin for AttitudePlugin {
    fn build(&self, app: &mut App) {
        use bevy_gizmos_ext::GizmoConfigExt;
        use bevy_gizmos_ext::GizmoLineConfigExt;
        // TODO: Remove this.
        app.insert_resource(Knob { one: 0.4, two: 3.6 });
        app.insert_gizmo_config(
            GraduationLineGizmoConfigGroup,
            GizmoConfig::default()
                .with_line(GizmoLineConfig::default().with_width(2.0))
                .with_render_layers(RenderLayers::layer(
                    Layer::ATTITUDE_INDICATOR_GRADUATION_LINES,
                )),
        );
        app.insert_gizmo_config(VectorGizmoConfigGroup, VectorGizmoConfigGroup::config());
        app.insert_gizmo_config(
            BoresightBackgroundGizmoConfigGroup,
            GizmoConfig::default()
                .with_line(
                    GizmoLineConfig::default()
                        .with_width(6.0)
                        .with_joints(GizmoLineJoint::Miter),
                )
                .with_render_layers(RenderLayers::layer(
                    Layer::ATTITUDE_INDICATOR_BORESIGHT_BACKGROUND,
                )),
        );
        app.insert_gizmo_config(
            BoresightGizmoConfigGroup,
            GizmoConfig::default()
                .with_line(
                    GizmoLineConfig::default()
                        .with_width(4.0)
                        .with_joints(GizmoLineJoint::Miter),
                )
                .with_render_layers(RenderLayers::layer(
                    Layer::ATTITUDE_INDICATOR_BORESIGHT_FOREGROUND,
                )),
        );
        app.add_systems(Startup, setup);
        app.add_systems(
            PostUpdate,
            (
                update_artificial_horizon.after(TransformSystems::Propagate),
                draw_pitch_graduation_lines.after(TransformSystems::Propagate),
                draw_path_vectors.after(TransformSystems::Propagate),
                draw_boresight,
            ),
        );
    }
}

#[derive(Component)]
struct AttitudeIndicator;

/// A [`GizmoConfigGroup`] for rendering the boresight foreground.
#[derive(GizmoConfigGroup, Default, Reflect)]
struct BoresightGizmoConfigGroup;

/// A [`GizmoConfigGroup`] for rendering the boresight background.
#[derive(GizmoConfigGroup, Default, Reflect)]
struct BoresightBackgroundGizmoConfigGroup;

/// A [`GizmoConfigGroup`] for rendering the graduation lines.
#[derive(GizmoConfigGroup, Default, Reflect)]
struct GraduationLineGizmoConfigGroup;

/// A [`GizmoConfigGroup`] for rendering the vector indicators.
#[derive(GizmoConfigGroup, Default, Reflect)]
struct VectorGizmoConfigGroup;

impl VectorGizmoConfigGroup {
    fn config() -> GizmoConfig {
        use bevy_gizmos_ext::GizmoConfigExt;
        use bevy_gizmos_ext::GizmoLineConfigExt;
        GizmoConfig::default()
            .with_line(GizmoLineConfig::default().with_width(2.0))
            .with_render_layers(RenderLayers::layer(Layer::ATTITUDE_INDICATOR_VECTOR))
    }
}

#[derive(Component)]
struct Sky;

#[derive(Component)]
struct Ground;

fn camera(order: isize, layer: Layer) -> impl Bundle {
    (
        Camera {
            order,
            clear_color: ClearColorConfig::None,
            ..default()
        },
        Camera2d,
        RenderLayers::layer(layer),
    )
}

/// Spawns cameras used for rendering Gizmos in the attitude indicator.
fn setup(mut commands: Commands) {
    commands.spawn(camera(2, Layer::ATTITUDE_INDICATOR_GRADUATION_LINES));
    commands.spawn(camera(3, Layer::ATTITUDE_INDICATOR_VECTOR));
    commands.spawn(camera(4, Layer::ATTITUDE_INDICATOR_BORESIGHT_BACKGROUND));
    commands.spawn(camera(5, Layer::ATTITUDE_INDICATOR_BORESIGHT_FOREGROUND));
}

pub(super) fn attitude_indicator() -> impl Bundle {
    (
        AttitudeIndicator,
        Name::new("Attitude indicator"),
        Node {
            width: px(172.0),
            height: px(172.0),
            flex_direction: FlexDirection::Column,
            ..default()
        },
        ZIndex(2),
        children![
            (
                Sky,
                Node {
                    width: percent(100.0),
                    height: percent(50.0),
                    border_radius: BorderRadius::px(16.0, 16.0, 0.0, 0.0),
                    ..default()
                },
                BackgroundColor::from(Color::srgb(0.02, 0.59, 1.0)),
                // BackgroundGradient::from(LinearGradient {
                //     stops: vec![
                //         // Color::srgb(0.0, 0.69, 0.95).into(),
                //         Color::srgb(0.0, 0.84, 0.99).into(),
                //         Color::srgb(0.0, 0.69, 0.95).into(),
                //     ],
                //     ..default()
                // }),
            ),
            (
                Ground,
                Node {
                    width: percent(100.0),
                    height: percent(50.0),
                    border_radius: BorderRadius::px(0.0, 0.0, 16.0, 16.0),
                    ..default()
                },
                BackgroundColor::from(Color::srgb(0.6, 0.28, 0.06)),
                // BackgroundGradient::from(LinearGradient {
                //     stops: vec![
                //         Color::srgb(0.77, 0.39, 0.15).into(),
                //         Color::srgb(1.0, 0.57, 0.14).into(),
                //         // Color::srgb(0.77, 0.39, 0.15).into(),
                //     ],
                //     ..default()
                // }),
            ),
        ],
    )
}

fn to_display_angle(angle_degrees: f32) -> f32 {
    match angle_degrees {
        -360.0..-270.0 => 360.0 - angle_degrees.abs(),
        -270.0..-180.0 => -(180.0 - angle_degrees.abs()),
        -180.0 => 0.0,
        -180.0..-90.0 => -(180.0 - angle_degrees.abs()),
        -90.0..0.0 => angle_degrees,
        0.0 => 0.0,
        0.0..90.0 => angle_degrees,
        90.0..180.0 => 180.0 - angle_degrees,
        _ => angle_degrees.abs(),
    }
}

#[derive(Resource, Reflect, Debug)]
#[reflect(Resource)]
struct Knob {
    one: f32,
    two: f32,
}

fn draw_boresight(
    mut gizmos: Gizmos<BoresightGizmoConfigGroup>,
    mut background_gizmos: Gizmos<BoresightBackgroundGizmoConfigGroup>,
    indicator_transform: Single<&UiGlobalTransform, With<AttitudeIndicator>>,
    window: Single<&Window>,
) {
    let origin = Vec2::new(-window.width() * 0.5, window.height() * 0.5);
    let x = origin.x + indicator_transform.translation.x;
    let y = origin.y - indicator_transform.translation.y;
    background_gizmos.linestrip_2d(
        vec![
            Vec2::new(x - 32.0, y),
            Vec2::new(x - 10.0, y),
            Vec2::new(x + 00.0, y - 10.0),
            Vec2::new(x + 10.0, y),
            Vec2::new(x + 32.0, y),
        ],
        Color::WHITE,
    );
    gizmos.linestrip_2d(
        vec![
            Vec2::new(x - 30.0, y),
            Vec2::new(x - 10.0, y),
            Vec2::new(x + 00.0, y - 10.0),
            Vec2::new(x + 10.0, y),
            Vec2::new(x + 30.0, y),
        ],
        Color::BLACK,
    );
    gizmos.circle_2d(vec2(x, y), 2.0, Color::WHITE);
    gizmos.circle_2d(vec2(x, y), 1.0, Color::BLACK);
}

fn draw_pitch_graduation_lines(
    mut gizmos: Gizmos<GraduationLineGizmoConfigGroup>,
    subject_transform: Single<&Transform, With<HudSubject>>,
    indicator: Single<(&UiGlobalTransform, &ComputedNode), With<AttitudeIndicator>>,
    window: Single<&Window>,
    knob: Res<Knob>,
) {
    use bevy_gizmos_ext::GizmoBufferExt;
    let (indicator_transform, indicator_computed_node) = indicator.into_inner();
    let rotation = subject_transform.rotation;
    let (_axis, angle) = rotation.to_axis_angle();
    let angle_degrees = angle.to_degrees() - 90.0;

    let origin = Vec2::new(-window.width() * 0.5, window.height() * 0.5);
    let x = origin.x + indicator_transform.translation.x;
    let y = origin.y - indicator_transform.translation.y;

    for i in -144..=72 {
        let width_modifier = if i32::rem_euclid(i, 36) == 0 {
            0.30
        } else {
            match i32::rem_euclid(i, 4) {
                0 => 0.20,
                2 => 0.10,
                _ => 0.01,
            }
        };
        if i32::rem_euclid(i, 2) != 0 {
            continue;
        }

        // TODO: Clean this up.
        let y_final = y + (i as f32 + angle_degrees * knob.one) * knob.two;

        let half_indicator_node_height = indicator_computed_node.size.y * 0.5;
        if ((y - half_indicator_node_height)..(y + half_indicator_node_height)).contains(&y_final) {
            let x1 = x - indicator_computed_node.size.x * width_modifier;
            let x2 = x + indicator_computed_node.size.x * width_modifier;
            let display_angle = to_display_angle(i as f32 * 180.0 / 72.0);
            let text = match display_angle.rem_euclid(20.0) {
                0.0 => format!("{display_angle:.0}"),
                _ => String::new(),
            };
            gizmos.line_2d(Vec2::new(x1, y_final), Vec2::new(x2, y_final), Color::WHITE);
            if i32::rem_euclid(i, 4) == 0 {
                gizmos.text_2d(
                    Isometry2d::from_xy(x1 - 20.0, y_final),
                    &text,
                    8.0,
                    Vec2::ZERO,
                    Color::WHITE,
                );
                gizmos.text_2d(
                    Isometry2d::from_xy(x2 + 20.0, y_final),
                    &text,
                    8.0,
                    Vec2::ZERO,
                    Color::WHITE,
                );
            }
        }
    }
}

#[allow(clippy::type_complexity)]
fn update_artificial_horizon(
    subject_transform: Single<&Transform, With<HudSubject>>,
    mut sky: Single<&mut Node, (With<Sky>, Without<AttitudeIndicator>, Without<Ground>)>,
    mut ground: Single<&mut Node, (With<Ground>, Without<AttitudeIndicator>, Without<Sky>)>,
    indicator: Single<&mut Node, (With<AttitudeIndicator>, Without<Ground>, Without<Sky>)>,
) {
    const FRAC_3PI_2: f32 = 1.5 * PI;
    let mut indicator_node = indicator.into_inner();
    let mut rot = subject_transform
        .rotation
        .angle_between(Quat::from_axis_angle(Vec3::Z, FRAC_PI_2));
    if rot > FRAC_PI_2 {
        rot = PI - rot;
    }
    let (_axis, angle) = subject_transform.rotation.to_axis_angle();
    if (FRAC_PI_2..FRAC_3PI_2).contains(&angle) {
        rot = -rot;
    }
    if angle > PI {
        indicator_node.flex_direction = FlexDirection::ColumnReverse;
        ground.border_radius = BORDER_RADIUS_TOP;
        sky.border_radius = BORDER_RADIUS_BOTTOM;
    } else {
        indicator_node.flex_direction = FlexDirection::Column;
        sky.border_radius = BORDER_RADIUS_TOP;
        ground.border_radius = BORDER_RADIUS_BOTTOM;
    }

    sky.height = percent(50.0 + 50.0 * (rot * 1.5 / FRAC_PI_2).clamp(-1.0, 1.0));
    ground.height = percent(50.0 - 50.0 * (rot * 1.5 / FRAC_PI_2).clamp(-1.0, 1.0));
}

#[extension(trait Rot2Ext)]
impl Rot2 {
    const FRAC_TAU_3: Rot2 = Rot2 {
        cos: -0.5,
        sin: 0.866_025_4,
    };
    const FRAC_2TAU_3: Rot2 = Rot2 {
        cos: -0.5,
        sin: -0.866_025_4,
    };
    const NEG_FRAC_PI_2: Rot2 = Rot2::FRAC_PI_2.inverse();
}

#[extension(trait AttitudeIndicatorGizmosExt)]
impl<'w, 's, Config, Clear> Gizmos<'w, 's, Config, Clear>
where
    Config: GizmoConfigGroup,
    Clear: 'static + Send + Sync,
{
    fn prograde_vector(&mut self, position: Vec2) {
        // let color = Color::srgb(0.22, 0.4, 0.29);
        use bevy_gizmos_ext::GizmoBufferExt;
        const COLOR: Color = Color::BLACK;
        const RADIUS: f32 = 8.0;
        self.circle_2d(position, RADIUS, COLOR);
        self.radial_2d(position, RADIUS, RADIUS * 2.0, COLOR);
        self.radial_2d(
            Isometry2d::new(position, Rot2::FRAC_PI_2),
            RADIUS,
            RADIUS * 2.0,
            COLOR,
        );
        self.radial_2d(
            Isometry2d::new(position, Rot2::NEG_FRAC_PI_2),
            RADIUS,
            RADIUS * 2.0,
            COLOR,
        );
    }

    /// Draws a retrograde vector indicator at the given position.
    fn retrograde_vector(&mut self, position: Vec2) {
        use bevy_gizmos_ext::GizmoBufferExt;
        const COLOR: Color = Color::BLACK;
        const RADIUS: f32 = 8.0;
        self.circle_2d(position, RADIUS, COLOR);
        self.cross_2d(Isometry2d::new(position, Rot2::FRAC_PI_4), RADIUS, COLOR);
        for rotation in [Rot2::IDENTITY, Rot2::FRAC_TAU_3, Rot2::FRAC_2TAU_3] {
            self.radial_2d(
                Isometry2d::new(position, rotation),
                RADIUS,
                RADIUS * 2.0,
                COLOR,
            );
        }
    }
}

/// Draws indicators representing prograde and retrograde velocity vectors.
fn draw_path_vectors(
    subject: Single<(&Transform, &RigidBody, &SatelliteOf), With<HudSubject>>,
    mut gizmos: Gizmos<VectorGizmoConfigGroup>,
    primary_query: Query<&RigidBody, Without<HudSubject>>,
    indicator: Single<(&UiGlobalTransform, &ComputedNode), With<AttitudeIndicator>>,
    window: Single<&Window>,
) {
    let (subject_transform, subject_rigidbody, satellite_of) = subject.into_inner();
    let primary_rigidbody = primary_query.get(satellite_of.primary()).unwrap();
    let velocity = subject_rigidbody.velocity - primary_rigidbody.velocity;

    let (indicator_transform, indicator_computed_node) = indicator.into_inner();

    let angle = velocity
        .xy()
        .angle_to((subject_transform.rotation * Vec3::Y).xy());
    let angle_degrees = angle.to_degrees();

    let origin = Vec2::new(-window.width() * 0.5, window.height() * 0.5);
    let x = origin.x + indicator_transform.translation.x;
    let y = origin.y - indicator_transform.translation.y;

    // TODO: Radial-in / radial-out.
    let half_indicator_node_height = indicator_computed_node.size.y * 0.5;
    if (-half_indicator_node_height..half_indicator_node_height).contains(&angle_degrees) {
        gizmos.prograde_vector(vec2(x, y + angle_degrees));
    }
    let mut offset = (angle + PI).to_degrees();
    if (-half_indicator_node_height..half_indicator_node_height).contains(&offset) {
        gizmos.retrograde_vector(vec2(x, y + offset));
    }
    offset = (angle - PI).to_degrees();
    if (-half_indicator_node_height..half_indicator_node_height).contains(&offset) {
        gizmos.retrograde_vector(vec2(x, y + offset));
    }
}
