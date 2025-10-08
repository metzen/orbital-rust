use crate::{
    camera::{Autofollow, HIGH_RES_LAYER, InGameCamera},
    physics::{CelestialBody, RigidBody},
    timewarp::{TIME_WARPS, TimeWarp},
    vessel::Vessel,
};
use bevy::{
    camera::{primitives::Aabb, visibility::RenderLayers},
    color::palettes::css::{BLACK, MAGENTA, YELLOW},
    ecs::query::QuerySingleError,
    math::ops::log10,
    prelude::*,
};
use big_space::{
    floating_origins::BigSpace,
    grid::{Grid, cell::CellCoord},
};
use leafwing_input_manager::{
    Actionlike,
    plugin::InputManagerPlugin,
    prelude::{ActionState, InputMap},
};

pub struct HudPlugin;

impl Plugin for HudPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_hud);
        app.add_systems(
            Update,
            (
                update_time_warp,
                update_throttle,
                update_velocity,
                update_acceleration,
                update_altitude,
                update_hud_subject,
                update_time,
                update_vertical_speed,
            ),
        );
        app.add_plugins(InputManagerPlugin::<HudAction>::default());
        app.init_resource::<ActionState<HudAction>>();
        app.insert_resource(HudAction::default_input_map());
    }
}

#[derive(Component)]
struct TimeWarpText;

#[derive(Component)]
struct ThrottleText;

#[derive(Component)]
struct VelocityText;

#[derive(Component)]
struct AccelerationText;

#[derive(Component)]
struct AltitudeText;

#[derive(Component)]
pub struct HudSubject;

#[derive(Component)]
pub struct HubSubjectText;

#[derive(Component)]
pub struct TimeText;

#[derive(Component)]
pub struct TimeWarpBoxes;

#[derive(Component)]
pub struct ThrottleBar;

#[derive(Component)]
pub struct VerticalSpeedText;

const BORDER: UiRect = UiRect::new(Val::Px(1.0), Val::Px(1.0), Val::Px(1.0), Val::Px(2.0));

fn setup_throttle_widget(commands: &mut Commands, text_font: &TextFont) {
    commands
        .spawn((
            Name::new("Throttle Widget"),
            Node {
                left: Val::Px(20.0),
                bottom: Val::Px(20.0),
                height: Val::Px(140.0),
                width: Val::Px(30.0),
                position_type: PositionType::Absolute,
                flex_direction: FlexDirection::ColumnReverse,
                align_items: AlignItems::FlexEnd,
                border: BORDER,
                padding: UiRect::all(Val::Px(5.0)),
                ..default()
            },
            BorderColor::from(Color::srgb(105.0 / 255.0, 109.0 / 255.0, 255.0)),
            BorderRadius::all(Val::Px(3.0)),
            BackgroundColor::from(BLACK),
            Outline::new(Val::Px(1.0), Val::Px(0.0), Color::from(BLACK)),
        ))
        .with_children(|node| {
            node.spawn((
                Node {
                    height: Val::Percent(0.0),
                    width: Val::Percent(100.0),
                    ..default()
                },
                ThrottleBar,
                BackgroundColor::from(Color::srgb(0.0, 0.8, 0.32)),
            ));
            node.spawn((Text::default(), ThrottleText, text_font.clone()));
        });
}

fn setup_time_widget(commands: &mut Commands, text_font: &TextFont) {
    commands
        .spawn((
            Name::new("Time widget"),
            Node {
                margin: UiRect {
                    left: Val::Auto,
                    right: Val::Auto,
                    bottom: Val::Px(20.0),
                    top: Val::Auto,
                },
                border: BORDER,
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
            BorderColor::from(Color::srgb(105.0 / 255.0, 109.0 / 255.0, 255.0)),
            BorderRadius::all(Val::Px(3.0)),
            BackgroundColor::from(BLACK),
            Outline::new(Val::Px(1.0), Val::Px(0.0), Color::from(BLACK)),
        ))
        .with_children(|node| {
            node.spawn((Text::default(), TimeText))
                .with_children(|text| {
                    text.spawn((
                        TextSpan::new("T+"),
                        text_font.clone(),
                        TextColor::from(Color::srgb(4.0 / 255.0, 152.0 / 255.0, 255.0)),
                    ));
                    text.spawn((TextSpan::new("000"), text_font.clone()));
                    text.spawn((
                        TextSpan::new("y "),
                        text_font.clone(),
                        TextColor::from(Color::srgb(4.0 / 255.0, 152.0 / 255.0, 255.0)),
                    ));
                    text.spawn((TextSpan::new("000"), text_font.clone()));
                    text.spawn((
                        TextSpan::new("d "),
                        text_font.clone(),
                        TextColor::from(Color::srgb(4.0 / 255.0, 152.0 / 255.0, 255.0)),
                    ));
                    text.spawn((TextSpan::new("00"), text_font.clone()));
                    text.spawn((
                        TextSpan::new(":"),
                        text_font.clone(),
                        TextColor::from(Color::srgb(4.0 / 255.0, 152.0 / 255.0, 255.0)),
                    ));
                    text.spawn((TextSpan::new("00"), text_font.clone()));
                    text.spawn((
                        TextSpan::new(":"),
                        text_font.clone(),
                        TextColor::from(Color::srgb(4.0 / 255.0, 152.0 / 255.0, 255.0)),
                    ));
                    text.spawn((TextSpan::new("00"), text_font.clone()));
                });
        })
        .with_children(|node| {
            node.spawn((
                Node {
                    column_gap: Val::Px(2.0),
                    ..default()
                },
                TimeWarpBoxes,
            ))
            .with_children(|node| {
                for _ in 0..TIME_WARPS.len() {
                    node.spawn((
                        Node {
                            width: Val::Px(20.0),
                            height: Val::Px(16.0),
                            justify_content: JustifyContent::Center,
                            ..default()
                        },
                        BackgroundColor::from(Color::srgb(0.446, 0.471, 0.525)),
                    ))
                    .with_child((
                        Text::new(">"),
                        text_font.clone(),
                        TextColor::BLACK,
                    ));
                }
            });
        })
        .with_children(|node| {
            node.spawn((
                Text::default(),
                TextLayout::new_with_justify(Justify::Center),
            ))
            .with_children(|text| {
                text.spawn((
                    TextSpan::new("TIME.WARP= "),
                    text_font.clone(),
                    TextColor::from(Color::srgb(105.0 / 255.0, 109.0 / 255.0, 255.0)),
                ));
                text.spawn((
                    TextSpan::default(),
                    TimeWarpText,
                    text_font.clone(),
                    TextColor::from(Color::srgb(105.0 / 255.0, 109.0 / 255.0, 255.0)),
                ));
                text.spawn((
                    TextSpan::new("x"),
                    text_font.clone(),
                    TextColor::from(Color::srgb(105.0 / 255.0, 109.0 / 255.0, 255.0)),
                ));
            });
        });
}

fn setup_velocity_widget(commands: &mut Commands, text_font: &TextFont) {
    commands
        .spawn((
            Name::new("Velocity widget"),
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(70.0),
                bottom: Val::Px(130.0),
                border: UiRect::px(1.0, 1.0, 1.0, 3.0),
                padding: UiRect::all(Val::Px(5.0)),
                flex_direction: FlexDirection::Column,
                width: Val::Px(80.0),
                row_gap: Val::Px(6.0),
                ..default()
            },
            BorderColor::from(Color::srgb(213.0 / 255.0, 175.0 / 255.0, 3.0 / 255.0)),
            BackgroundColor::from(BLACK),
            BorderRadius::all(Val::Px(3.0)),
            Outline::new(Val::Px(1.0), Val::Px(0.0), Color::from(BLACK)),
        ))
        .with_children(|node| {
            node.spawn((
                Text::new("SURFACE"),
                TextLayout::new_with_justify(Justify::Right),
                TextColor::from(Color::srgb(213.0 / 255.0, 175.0 / 255.0, 3.0 / 255.0)),
                BackgroundColor::from(Color::srgb(44.0 / 255.0, 35.0 / 255.0, 0.0)),
                text_font.clone(),
            ));
            node.spawn((
                Text::default(),
                TextLayout::new_with_justify(Justify::Right),
                text_font.clone(),
            ))
            .with_children(|parent| {
                parent.spawn((
                    TextSpan::default(),
                    text_font.clone().with_font_size(16.0),
                    VelocityText,
                ));
            });
            node.spawn((
                Text::new("m/s"),
                TextLayout::new_with_justify(Justify::Right),
                TextColor::from(Color::srgb(213.0 / 255.0, 175.0 / 255.0, 3.0 / 255.0)),
                BackgroundColor::from(Color::srgb(44.0 / 255.0, 35.0 / 255.0, 0.0)),
                text_font.clone(),
            ));
        });
}

fn setup_altitude_widget(commands: &mut Commands, text_font: &TextFont) {
    commands
        .spawn((
            Name::new("Altitude widget"),
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(350.0),
                bottom: Val::Px(130.0),
                width: Val::Px(80.0),
                border: UiRect::px(1.0, 1.0, 1.0, 3.0),
                padding: UiRect::all(Val::Px(5.0)),
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(6.0),
                ..default()
            },
            BackgroundColor::from(BLACK),
            BorderColor::from(Color::srgb(199.0 / 255.0, 70.0 / 255.0, 198.0 / 255.0)),
            BorderRadius::all(Val::Px(3.0)),
            Outline::new(Val::Px(1.0), Val::Px(0.0), Color::from(BLACK)),
        ))
        .with_children(|node| {
            node.spawn((
                Text::new("SEA LVL"),
                TextColor::from(Color::srgb(199.0 / 255.0, 70.0 / 255.0, 198.0 / 255.0)),
                BackgroundColor::from(Color::srgb(36.0 / 255.0, 5.0 / 365.0, 35.0 / 255.0)),
                text_font.clone(),
            ));
            node.spawn((Text::default(), text_font.clone()))
                .with_children(|parent| {
                    parent.spawn((
                        TextSpan::default(),
                        text_font.clone().with_font_size(16.0),
                        AltitudeText,
                    ));
                });
            node.spawn((
                Text::new("m"),
                TextColor::from(Color::srgb(199.0 / 255.0, 70.0 / 255.0, 198.0 / 255.0)),
                BackgroundColor::from(Color::srgb(36.0 / 255.0, 5.0 / 365.0, 35.0 / 255.0)),
                text_font.clone(),
            ));
        });
}

fn setup_orbital_info_widget(commands: &mut Commands, text_font: &TextFont) {
    commands
        .spawn((
            Name::new("Orbit info"),
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(145.0),
                bottom: Val::Px(20.0),
                border: BORDER,
                padding: UiRect::axes(Val::Px(10.0), Val::Px(6.0)),
                flex_direction: FlexDirection::Column,
                ..default()
            },
            BorderColor::from(Color::srgb(105.0 / 255.0, 109.0 / 255.0, 255.0)),
            BorderRadius::all(Val::Px(3.0)),
            BackgroundColor::from(BLACK),
            Outline::new(Val::Px(1.0), Val::Px(0.0), Color::from(BLACK)),
        ))
        .with_children(|node| {
            node.spawn((
                Node {
                    column_gap: Val::Px(5.0),
                    ..default()
                },
                Text::default(),
            ))
            .with_children(|text| {
                text.spawn((
                    TextSpan::new("AP "),
                    text_font.clone(),
                    TextColor::from(Color::srgb(0.643, 0.427, 0.518)),
                ));
                text.spawn((TextSpan::new("000,000"), text_font.clone()));
                text.spawn((
                    TextSpan::new(" m"),
                    text_font.clone(),
                    TextColor::from(Color::srgb(0.718, 0.588, 0.376)),
                ));
                text.spawn((TextSpan::new(" in "), text_font.clone()));
                text.spawn((
                    TextSpan::new("T+"),
                    text_font.clone(),
                    TextColor::from(Color::srgb(0.718, 0.588, 0.376)),
                ));
                text.spawn((TextSpan::new("00"), text_font.clone()));
                text.spawn((
                    TextSpan::new(":"),
                    text_font.clone(),
                    TextColor::from(Color::srgb(0.718, 0.588, 0.376)),
                ));
                text.spawn((TextSpan::new("00"), text_font.clone()));
                text.spawn((
                    TextSpan::new(":"),
                    text_font.clone(),
                    TextColor::from(Color::srgb(0.718, 0.588, 0.376)),
                ));
                text.spawn((TextSpan::new("00"), text_font.clone()));
            });
            node.spawn((
                Text::new("PE"),
                text_font.clone(),
                TextColor::from(Color::srgb(0.125, 0.506, 0.63)),
            ));
        });
}

fn setup_staging_widget(commands: &mut Commands) {
    commands
        .spawn((
            Node {
                margin: UiRect {
                    left: Val::Auto,
                    right: Val::Px(20.0),
                    bottom: Val::Px(20.0),
                    top: Val::Auto,
                },
                border: BORDER,
                padding: UiRect::all(Val::Px(5.0)),
                flex_direction: FlexDirection::Column,
                ..default()
            },
            BorderColor::from(Color::srgb(105.0 / 255.0, 109.0 / 255.0, 255.0)),
            BorderRadius::all(Val::Px(3.0)),
            BackgroundColor::from(BLACK),
            Outline::new(Val::Px(1.0), Val::Px(0.0), Color::from(BLACK)),
        ))
        .with_children(|node| {
            node.spawn((
                Node {
                    width: Val::Px(200.0),
                    height: Val::Px(50.0),
                    ..default()
                },
                BackgroundColor::from(Color::srgb(0.0, 0.8, 0.32)),
                BorderRadius::all(Val::Px(3.0)),
            ));
        });
}

fn setup_vertical_speed_widget(commands: &mut Commands, text_font: &TextFont) {
    commands
        .spawn((
            Name::new("Vertical speed"),
            Node {
                position_type: PositionType::Absolute,
                bottom: Val::Px(20.0),
                left: Val::Px(448.0),
                width: Val::Px(50.0),
                border: BORDER,
                padding: UiRect::all(Val::Px(5.0)),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::End,
                row_gap: Val::Px(14.0),
                ..default()
            },
            BorderColor::from(Color::srgb(105.0 / 255.0, 109.0 / 255.0, 255.0)),
            BorderRadius::all(Val::Px(3.0)),
            BackgroundColor::from(BLACK),
            Outline::new(Val::Px(1.0), Val::Px(0.0), Color::from(BLACK)),
        ))
        .with_children(|node| {
            node.spawn((
                Text::new("+100"),
                text_font.clone(),
                TextColor::from(Color::srgb(105.0 / 255.0, 109.0 / 255.0, 255.0)),
            ));
            node.spawn((
                Text::new("+10"),
                text_font.clone(),
                TextColor::from(Color::srgb(105.0 / 255.0, 109.0 / 255.0, 255.0)),
            ));
            node.spawn((
                Text::new("0"),
                text_font.clone(),
                TextColor::from(Color::srgb(105.0 / 255.0, 109.0 / 255.0, 255.0)),
            ));
            node.spawn((
                Text::new("-10"),
                text_font.clone(),
                TextColor::from(Color::srgb(105.0 / 255.0, 109.0 / 255.0, 255.0)),
            ));
            node.spawn((
                Text::new("-100"),
                text_font.clone(),
                TextColor::from(Color::srgb(105.0 / 255.0, 109.0 / 255.0, 255.0)),
            ));
            node.spawn((
                Name::new("vertical speed indicator"),
                Node {
                    position_type: PositionType::Absolute,
                    height: Val::Px(20.0),
                    ..default()
                },
                Text::default(),
                text_font.clone(),
                VerticalSpeedText,
            ));
        });
}

fn setup_rotation_widget(commands: &mut Commands, text_font: &TextFont) {
    commands.spawn((
        Name::new("Rotation widget"),
        Node {
            position_type: PositionType::Absolute,
            bottom: Val::Px(80.0),
            left: Val::Px(156.0),
            width: Val::Px(184.0),
            height: Val::Px(184.0),
            border: BORDER,
            padding: UiRect::all(Val::Px(5.0)),
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::End,
            row_gap: Val::Px(14.0),
            ..default()
        },
        BorderColor::from(Color::srgb(105.0 / 255.0, 109.0 / 255.0, 255.0)),
        BorderRadius::all(Val::Percent(50.0)),
        BackgroundColor::from(BLACK),
        Outline::new(Val::Px(1.0), Val::Px(0.0), Color::from(BLACK)),
    ));
}

fn setup_hud(mut commands: Commands) {
    let text_font = TextFont {
        font_size: 12.0,
        font_smoothing: bevy::text::FontSmoothing::AntiAliased,
        ..default()
    };
    commands.spawn((
        Node::default(),
        Text::default(),
        HubSubjectText,
        RenderLayers::layer(HIGH_RES_LAYER),
        text_font.clone(),
    ));
    setup_rotation_widget(&mut commands, &text_font);
    setup_throttle_widget(&mut commands, &text_font);
    setup_staging_widget(&mut commands);
    setup_orbital_info_widget(&mut commands, &text_font);
    setup_time_widget(&mut commands, &text_font);
    setup_velocity_widget(&mut commands, &text_font);
    setup_altitude_widget(&mut commands, &text_font);
    setup_vertical_speed_widget(&mut commands, &text_font);
    commands
        .spawn(Node {
            // fill the entire window
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::End,
            // padding: UiRect::all(MARGIN),
            // row_gap: Val::Px(),
            margin: UiRect::new(Val::Px(0.0), Val::Px(0.0), Val::Px(30.0), Val::Px(0.0)),
            ..default()
        })
        .with_children(|root| {
            root.spawn((
                Text::new("ACC: "),
                RenderLayers::layer(HIGH_RES_LAYER),
                text_font.clone(),
            ));
            root.spawn((
                Text::default(),
                AccelerationText,
                RenderLayers::layer(HIGH_RES_LAYER),
                text_font.clone(),
            ));
        });
}

fn symlog_plot(value: f32, max_value: f32, linear_threshold: f32, linear_scale: f32) {
    let max_plot = log10(max_value);
}

fn update_vertical_speed(
    subject: Single<(&RigidBody, &CellCoord, &Transform), With<HudSubject>>,
    rigidbody_query: Query<(&RigidBody, &CellCoord, &Transform), Without<HudSubject>>,
    mut vertical_speed_ui: Single<(&mut Node, &mut Text), With<VerticalSpeedText>>,
    grid: Single<&Grid, With<BigSpace>>,
) {
    let (subject_rigidbody, subject_gridcell, subject_transform) = *subject;
    if let Some(primary_id) = subject_rigidbody.primary
        && let Ok(primary) = rigidbody_query.get(primary_id)
    {
        // TODO: direction from center to center.
        let (primary_rigidbody, primary_gridcell, primary_transform) = primary;
        let relative_velocity = subject_rigidbody.velocity - primary_rigidbody.velocity;
        let relative_position = grid.grid_position_double(subject_gridcell, subject_transform)
            - grid.grid_position_double(primary_gridcell, primary_transform);
        let vertical_speed = relative_velocity.dot(relative_position.normalize().as_vec3());

        let vertical_speed_text = if (-10.0..10.0).contains(&vertical_speed) {
            format!("{:+.2}", vertical_speed)
        } else if (-100.0..100.0).contains(&vertical_speed) {
            format!("{:+.1}", vertical_speed)
        } else {
            format!("{:+.0}", vertical_speed)
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
                } else {
                    Color::srgb(0.439, 0.451, 0.525)
                }
            }
        }
    }
}

fn update_throttle(
    vessel: Single<&Vessel, With<HudSubject>>,
    mut throttle_bar_node: Single<&mut Node, With<ThrottleBar>>,
    mut throttle_text: Single<&mut Text, With<ThrottleText>>,
) {
    throttle_bar_node.height = Val::Percent(vessel.throttle * 100.0);
    throttle_text.0 = format!("{:.0}", vessel.throttle * 100.0);
}

fn update_velocity(
    mut text: Single<&mut TextSpan, With<VelocityText>>,
    subject_rigidbody: Query<&RigidBody, With<HudSubject>>,
    primary_body_query: Query<&RigidBody>,
) {
    if let Ok(rigidbody) = subject_rigidbody.single()
        && let Some(primary_body_entity) = rigidbody.primary
        && let Ok(primary_body) = primary_body_query.get(primary_body_entity)
    {
        text.0 = format!(
            "{:.2}",
            (rigidbody.velocity - primary_body.velocity).length()
        );
    } else {
        text.0 = String::new();
    };
}

fn update_acceleration(
    mut query: Query<&mut Text, With<AccelerationText>>,
    vessel_rigidbody_query: Query<&RigidBody, With<HudSubject>>,
) {
    if let Some(rigidbody) = vessel_rigidbody_query.iter().next() {
        query.single_mut().unwrap().0 = format!("{:.2}", rigidbody.acceleration);
    }
}

fn update_altitude(
    mut text: Single<&mut TextSpan, With<AltitudeText>>,
    grid: Single<&Grid, With<BigSpace>>,
    subject_query: Query<(&Transform, &CellCoord, &RigidBody, &Aabb), With<HudSubject>>,
    primary_body_query: Query<(&Transform, &CellCoord, &CelestialBody)>,
) {
    if let Ok((subject_transform, subject_grid_cell, subject_rigidbody, subject_aabb)) =
        subject_query.single()
        && let Some(primary_body) = subject_rigidbody.primary
        && let Ok((primary_transform, primary_grid_cell, celestial_body)) =
            primary_body_query.get(primary_body)
    {
        let primary_position = grid.grid_position_double(primary_grid_cell, primary_transform);
        let subject_position = grid.grid_position_double(subject_grid_cell, subject_transform);
        let distance = primary_position.distance(subject_position);
        // TODO: Calculate from lowest vertex of mesh instead of the Aabb.
        let altitude = distance - celestial_body.radius as f64 - subject_aabb.half_extents.y as f64;
        text.0 = format!("{:.0}", altitude);
    } else {
        text.0 = String::new();
    }
}

#[derive(Actionlike, PartialEq, Eq, Clone, Copy, Hash, Debug, Reflect)]
enum HudAction {
    NextVessel,
    PreviousVessel,
}

impl HudAction {
    fn default_input_map() -> InputMap<Self> {
        InputMap::default()
            .with(Self::NextVessel, KeyCode::BracketRight)
            .with(Self::NextVessel, GamepadButton::DPadRight)
            .with(Self::PreviousVessel, KeyCode::BracketLeft)
            .with(Self::PreviousVessel, GamepadButton::DPadLeft)
    }
}

fn update_hud_subject(
    mut commands: Commands,
    action_state: Res<ActionState<HudAction>>,
    mut vessels_query: Query<(Entity, &mut Vessel, Option<&HudSubject>), With<Vessel>>,
    mut camera_autofollow: Single<&mut Autofollow, With<InGameCamera>>,
    subject_vessel_query: Query<&Name, With<HudSubject>>,
    mut hud_subject_text: Single<&mut Text, With<HubSubjectText>>,
) {
    match subject_vessel_query.single() {
        Ok(name) => {
            hud_subject_text.0 = format!("Subject: {}", name);
        }
        Err(QuerySingleError::NoEntities(_)) => {
            info!("no subject?");
        }
        Err(QuerySingleError::MultipleEntities(_)) => {
            info!("multi subject");
        }
    }
    if action_state.just_pressed(&HudAction::NextVessel)
        || action_state.just_pressed(&HudAction::PreviousVessel)
    {
        let mut current_subject_index: i32 = -1;
        let mut i = 0;
        let mut entities = Vec::new();
        for (entity, mut vessel, hud_subject) in vessels_query.iter_mut().sort::<Entity>() {
            entities.push(entity);
            info!("hud subj vessel");
            if hud_subject.is_some() {
                info!("vessel is subject");
                current_subject_index = i;
                commands.entity(entity).remove::<HudSubject>();
                vessel.controlled = false;
            }
            i += 1;
        }
        let modifier = if action_state.just_pressed(&HudAction::NextVessel) {
            1
        } else if action_state.just_pressed(&HudAction::PreviousVessel) {
            -1
        } else {
            0
        };
        let new_subject_index = (current_subject_index + modifier).rem_euclid(i);
        let new_subject = entities[new_subject_index as usize];
        commands.entity(new_subject).insert(HudSubject);
        if let Ok((entity, mut vessel, _hud_subject)) = vessels_query.get_mut(new_subject) {
            vessel.controlled = true;
            camera_autofollow.target = Some(entity);
        }
    }
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
