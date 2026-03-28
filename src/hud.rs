mod sas_selector;

use bevy::camera::primitives::Aabb;
use bevy::camera::visibility::RenderLayers;
use bevy::color::palettes::css::{BLACK, LIGHT_GRAY};
use bevy::ecs::query::QueryData;
use bevy::ecs::system::lifetimeless::Read;
use bevy::input::common_conditions::input_toggle_active;
use bevy::math::DVec2;
use bevy::math::ops::log10;
use bevy::picking::pointer::PointerInteraction;
use bevy::prelude::*;
use bevy::text::LineHeight;
use big_space::floating_origins::BigSpace;
use big_space::grid::Grid;
use big_space::grid::cell::CellCoord;
use leafwing_input_manager::Actionlike;
use leafwing_input_manager::plugin::InputManagerPlugin;
use leafwing_input_manager::prelude::{ActionState, InputMap};

use crate::camera::{Autofollow, HIGH_RES_LAYER, InGameCamera, InGamePointer};
use crate::physics::{NoGravity, Orbit, OrbitShape, RigidBody, SatelliteOf};
use crate::timewarp::{TIME_WARPS, TimeWarp};
use crate::vessel::Vessel;

pub struct HudPlugin;

impl Plugin for HudPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, (setup_hud, setup_gizmos, sas_selector::setup));
        app.add_systems(
            FixedUpdate,
            (
                update_time_warp,
                update_throttle,
                update_velocity,
                update_altitude,
                update_hud_subject,
                update_time,
                update_vertical_speed,
                update_hover_text,
                update_orbital_info,
                update_sas_indicator_widget,
                sas_selector::update,
            ),
        );
        app.add_systems(
            PostUpdate,
            draw_orbits
                .after(TransformSystems::Propagate)
                .run_if(input_toggle_active(true, KeyCode::F8)),
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
struct AltitudeUnitsText;

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
pub struct HoverText;

#[derive(Component)]
pub struct VerticalSpeedText;

#[derive(Component)]
pub struct ApoapsisText;

#[derive(Component)]
pub struct PeriapsisText;

#[derive(Component)]
struct SasIndicator;

const BORDER: UiRect = UiRect::new(Val::Px(1.0), Val::Px(1.0), Val::Px(1.0), Val::Px(1.0));
// TODO: Use old value? BorderColor::from(Color::srgb(105.0 / 255.0, 109.0 / 255.0, 255.0))
const BORDER_COLOR: BorderColor = BorderColor {
    top: Color::srgb(0.184, 0.188, 0.251),
    right: Color::srgb(0.184, 0.188, 0.251),
    bottom: Color::srgb(0.298, 0.310, 0.478),
    left: Color::srgb(0.184, 0.188, 0.251),
};

trait TextFontExt {
    fn ui_default() -> Self;
}

impl TextFontExt for TextFont {
    fn ui_default() -> Self {
        Self {
            font_size: 12.0,
            ..default()
        }
    }
}

fn setup_gizmos(mut config_store: ResMut<GizmoConfigStore>) {
    let (config, _config_group) = config_store.config_mut::<DefaultGizmoConfigGroup>();
    config.line.width = 1.0;
    // config.render_layers = RenderLayers::layer(1);
}

fn setup_throttle_widget(commands: &mut Commands) {
    commands.spawn((
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
            border_radius: BorderRadius::all(Val::Px(3.0)),
            padding: UiRect::all(Val::Px(5.0)),
            ..default()
        },
        BORDER_COLOR,
        BackgroundColor::from(BLACK),
        Outline::new(Val::Px(1.0), Val::Px(0.0), Color::from(BLACK)),
        children![
            (
                Node {
                    height: Val::Percent(0.0),
                    width: Val::Percent(100.0),
                    ..default()
                },
                ThrottleBar,
                BackgroundColor::from(Color::srgb(0.0, 0.8, 0.32)),
            ),
            (Text::default(), ThrottleText, TextFont::ui_default()),
        ],
    ));
}

fn setup_time_widget(commands: &mut Commands) {
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

fn setup_velocity_widget(commands: &mut Commands) {
    let widget_color = Color::srgb(213.0 / 255.0, 175.0 / 255.0, 3.0 / 255.0);
    commands.spawn((
        Name::new("Velocity widget"),
        Node {
            position_type: PositionType::Absolute,
            left: Val::Px(70.0),
            bottom: Val::Px(130.0),
            border: UiRect::px(1.0, 1.0, 1.0, 3.0),
            border_radius: BorderRadius::all(Val::Px(3.0)),
            padding: UiRect::all(Val::Px(5.0)),
            flex_direction: FlexDirection::Column,
            width: Val::Px(80.0),
            row_gap: Val::Px(6.0),
            ..default()
        },
        BorderColor::from(widget_color),
        BackgroundColor::from(BLACK),
        Outline::new(Val::Px(1.0), Val::Px(0.0), Color::from(BLACK)),
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
                    VelocityText,
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
                        Text::new("V"),
                        TextColor::from(widget_color),
                        TextFont::ui_default(),
                        LineHeight::RelativeToFont(1.0),
                    ),
                    (
                        Text::new("E"),
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
                ],
            ),
        ],
    ));
}

fn setup_altitude_widget(commands: &mut Commands) {
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
                                Text::new(char as char),
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

fn setup_orbital_info_widget(commands: &mut Commands) {
    commands.spawn((
        Name::new("Orbit info"),
        Node {
            position_type: PositionType::Absolute,
            left: Val::Px(145.0),
            bottom: Val::Px(20.0),
            border: BORDER,
            border_radius: BorderRadius::all(Val::Px(3.0)),
            padding: UiRect::axes(Val::Px(10.0), Val::Px(6.0)),
            flex_direction: FlexDirection::Column,
            ..default()
        },
        BORDER_COLOR,
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

fn setup_staging_widget(commands: &mut Commands) {
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

fn setup_vertical_speed_widget(commands: &mut Commands) {
    commands.spawn((
        Name::new("Vertical speed"),
        Node {
            position_type: PositionType::Absolute,
            bottom: Val::Px(20.0),
            left: Val::Px(448.0),
            width: Val::Px(50.0),
            border: BORDER,
            border_radius: BorderRadius::all(Val::Px(3.0)),
            padding: UiRect::all(Val::Px(5.0)),
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::End,
            row_gap: Val::Px(14.0),
            ..default()
        },
        BORDER_COLOR,
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

fn setup_rotation_widget(commands: &mut Commands) {
    commands.spawn((
        Name::new("Rotation widget"),
        Node {
            position_type: PositionType::Absolute,
            bottom: Val::Px(80.0),
            left: Val::Px(156.0),
            width: Val::Px(184.0),
            height: Val::Px(184.0),
            border: BORDER,
            border_radius: BorderRadius::all(Val::Percent(50.0)),
            padding: UiRect::all(Val::Px(5.0)),
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::End,
            row_gap: Val::Px(14.0),
            ..default()
        },
        BORDER_COLOR,
        BackgroundColor::from(BLACK),
        Outline::new(Val::Px(1.0), Val::Px(0.0), Color::from(BLACK)),
    ));
}

fn setup_hover_text(commands: &mut Commands) {
    commands.spawn((
        Name::new("hover text"),
        Node {
            margin: UiRect {
                left: Val::Auto,
                right: Val::Auto,
                top: Val::Px(100.0),
                bottom: Val::Auto,
            },
            ..default()
        },
        Text::default(),
        TextFont::ui_default(),
        HoverText,
    ));
}

fn setup_subject_widget(commands: &mut Commands) {
    commands.spawn((
        Name::new("Subject text"),
        Node {
            top: px(20.0),
            left: px(20.0),
            padding: px(10.0).into(),
            ..default()
        },
        RenderLayers::layer(HIGH_RES_LAYER),
        BackgroundColor::from(Srgba::new(0.05, 0.11, 0.15, 1.0)),
        children![(
            Text::default(),
            TextLayout::new_with_justify(Justify::Center),
            HubSubjectText,
            TextFont::ui_default(),
        ),],
    ));
}

fn setup_sas_indicator_widget(commands: &mut Commands) {
    commands.spawn((
        Node {
            position_type: PositionType::Absolute,
            left: px(250.0),
            bottom: px(100.0),
            ..default()
        },
        Text::new("SAS"),
        TextFont::ui_default().with_font_size(14.0),
        BackgroundColor::from(Srgba::new(0.0, 0.1, 0.0, 1.0)),
        TextColor::from(Srgba::new(0.0, 0.9, 0.0, 1.0)),
        SasIndicator,
        ZIndex(2),
    ));
}

fn update_sas_indicator_widget(
    mut widget: Single<(&mut BackgroundColor, &mut TextColor), With<SasIndicator>>,
    vessel: Single<&Vessel, With<HudSubject>>,
) {
    widget.0.0 = Color::from(match vessel.sas_enabled {
        true => Srgba::new(0.0, 0.9, 0.0, 1.0),
        false => Srgba::new(0.0, 0.1, 0.0, 1.0),
    });
    widget.1.0 = Color::from(match vessel.sas_enabled {
        true => Srgba::new(0.0, 0.0, 0.0, 1.0),
        false => Srgba::new(0.0, 0.3, 0.0, 1.0),
    });
}

fn setup_hud(mut commands: Commands) {
    setup_subject_widget(&mut commands);
    setup_rotation_widget(&mut commands);
    setup_hover_text(&mut commands);
    setup_throttle_widget(&mut commands);
    setup_staging_widget(&mut commands);
    setup_orbital_info_widget(&mut commands);
    setup_time_widget(&mut commands);
    setup_velocity_widget(&mut commands);
    setup_altitude_widget(&mut commands);
    setup_vertical_speed_widget(&mut commands);
    setup_sas_indicator_widget(&mut commands);
}

// TODO: Use this.
// fn symlog_plot(value: f32, max_value: f32, linear_threshold: f32, linear_scale: f32) {
//     let max_plot = log10(max_value);
// }

fn update_vertical_speed(
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
                } else if i <= time_warp.max_allowed_index {
                    Color::srgb(0.439, 0.451, 0.525)
                } else {
                    Color::srgb(0.14, 0.13, 0.16)
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
    subject_rigidbody: Query<(&RigidBody, &SatelliteOf), With<HudSubject>>,
    primary_body_query: Query<&RigidBody>,
) {
    if let Ok((rigidbody, satellite_of)) = subject_rigidbody.single()
        && let Ok(primary_body) = primary_body_query.get(satellite_of.primary())
    {
        let relative_velocity = (rigidbody.velocity - primary_body.velocity).length();
        text.0 = format!(
            "{:.*}",
            // Show one digit of decimal precision when velocity is low.
            if relative_velocity < 10_000.0 { 1 } else { 0 },
            relative_velocity
        );
    } else {
        text.0 = String::new();
    };
}

fn update_altitude(
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
        let (humanized_altitude, units) = humanize_distance(altitude);
        text.0 = format!("{:.0}", humanized_altitude);
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
    subject_vessel_query: Query<Entity, With<HudSubject>>,
    mut hud_subject_text: Single<&mut Text, With<HubSubjectText>>,
    name_query: Query<(&Name, Option<&SatelliteOf>)>,
) {
    if let Ok(entity) = subject_vessel_query.single() {
        let mut parent = Some(entity);
        let mut parts = Vec::new();
        while let Some(entity) = parent
            && let Ok((name, satellite_of)) = name_query.get(entity)
        {
            parts.push(format!("{}", name));
            parent = satellite_of.map(|x| x.primary());
        }
        parts.reverse();
        hud_subject_text.0 = parts.join(" / ");
    } else {
        hud_subject_text.0 = String::from("none");
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

fn humanize_distance(altitude: f64) -> (f64, String) {
    let (value, units) = match altitude.abs() {
        // AU..INFINITY => (altitude / AU, "au"),
        0.0..1e6 => (altitude, "m "),
        1e6..1e9 => (altitude / 1e3, "km"),
        1e9..1e12 => (altitude / 1e6, "Mm"),
        _ => (altitude / 1e9, "Gm"),
    };
    (value, units.into())
}

fn update_orbital_info(
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
            let (humanized_ap, ap_units) = humanize_distance(ap);
            *writer.text(*ap_text, 2) = format!("{:>7.0}", humanized_ap);
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
            let (humanized_pe, pe_units) = humanize_distance(pe);
            *writer.text(*pe_text, 2) = format!("{:>7.0}", humanized_pe);
            *writer.text(*pe_text, 4) = pe_units;
            let time_to_pe = orbit.time_to_periapsis().as_secs();
            *writer.text(*pe_text, 7) = format!("{:02}", time_to_pe / 60 / 60);
            *writer.text(*pe_text, 9) = format!("{:02}", time_to_pe / 60 % 60);
            *writer.text(*pe_text, 11) = format!("{:02}", time_to_pe % 60);
            break;
        }
    }
}

fn update_hover_text(
    interactions: Query<&PointerInteraction, With<InGamePointer>>,
    names: Query<&Name>,
    mut text: Single<&mut Text, With<HoverText>>,
) {
    for interaction in interactions.iter() {
        if let Some((entity, _hit)) = interaction.get_nearest_hit() {
            if let Ok(name) = names.get(*entity) {
                text.0 = name.to_string();
            } else {
                text.0.clear();
            }
        } else {
            text.0.clear();
        }
    }
}

trait RenderOrbit {
    fn render(&self, gizmos: &mut Gizmos, translation: &Vec2, color: Color, fade: bool);
    fn render_ellipse(&self, gizmos: &mut Gizmos, translation: &Vec2, color: Color, fade: bool);
    fn render_hyperbola(&self, gizmos: &mut Gizmos, translation: &Vec2, color: Color, fade: bool);
}

impl RenderOrbit for Orbit {
    fn render(&self, gizmos: &mut Gizmos, translation: &Vec2, color: Color, fade: bool) {
        match self.shape() {
            OrbitShape::Circle => self.render_ellipse(gizmos, translation, color, fade),
            OrbitShape::Ellipse => self.render_ellipse(gizmos, translation, color, fade),
            OrbitShape::Parabola => self.render_hyperbola(gizmos, translation, color, fade),
            OrbitShape::Hyperbola => self.render_hyperbola(gizmos, translation, color, fade),
        }
    }

    fn render_ellipse(&self, gizmos: &mut Gizmos, translation: &Vec2, color: Color, fade: bool) {
        let angle = DVec2::X.angle_to(-self.eccentricity_vector) as f32;
        if fade {
            use crate::gizmos::GizmosExt;
            gizmos
                .ellipse_gradient_2d(
                    Isometry2d::new(translation + self.center().as_vec2(), Rot2::radians(angle)),
                    DVec2::new(self.semi_major_axis, self.semi_minor_axis).as_vec2(),
                    self.eccentric_anomaly().as_radians_f64() as f32,
                    color.with_alpha(0.01),
                    color.with_alpha(0.3),
                )
                .resolution(2000);
        } else {
            gizmos
                .ellipse_2d(
                    Isometry2d::new(translation + self.center().as_vec2(), Rot2::radians(angle)),
                    DVec2::new(self.semi_major_axis, self.semi_minor_axis).as_vec2(),
                    color.with_alpha(0.3),
                )
                .resolution(2000);
        }
    }

    fn render_hyperbola(&self, gizmos: &mut Gizmos, translation: &Vec2, color: Color, _fade: bool) {
        use crate::gizmos::GizmosExt;
        let angle = DVec2::X.angle_to(-self.eccentricity_vector) as f32;
        gizmos
            .hyperbola_2d(
                Isometry2d::new(translation - self.center().as_vec2(), Rot2::radians(angle)),
                DVec2::new(self.semi_major_axis, self.semi_minor_axis).as_vec2(),
                color.with_alpha(0.2),
            )
            .resolution(2000);
    }
}

#[derive(QueryData)]
struct OrbitSatelliteQueryData {
    cell: Read<CellCoord>,
    transform: Read<Transform>,
    global_transform: Read<GlobalTransform>,
    rigidbody: Read<RigidBody>,
    satellite_of: Read<SatelliteOf>,
    mesh_material: Read<MeshMaterial2d<ColorMaterial>>,
    vessel: Option<Read<Vessel>>,
}

#[derive(QueryData)]
struct OrbitPrimaryQueryData {
    cell: Read<CellCoord>,
    transform: Read<Transform>,
    global_transform: Read<GlobalTransform>,
    rigidbody: Read<RigidBody>,
}

fn draw_orbits(
    satellite_query: Query<OrbitSatelliteQueryData, Without<NoGravity>>,
    primary_query: Query<OrbitPrimaryQueryData>,
    grid: Single<&Grid, With<BigSpace>>,
    projection: Single<&Projection, With<InGameCamera>>,
    mut gizmos: Gizmos,
    materials: Res<Assets<ColorMaterial>>,
) {
    let apsis_radius = if let Projection::Orthographic(orthographic) = projection.into_inner() {
        orthographic.scale
    } else {
        1.0
    };
    for secondary in &satellite_query {
        let primary = primary_query.get(secondary.satellite_of.primary()).unwrap();
        let primary_position = grid.grid_position_double(primary.cell, primary.transform);
        let secondary_position = grid.grid_position_double(secondary.cell, secondary.transform);
        let orbit = Orbit::new(
            (secondary_position - primary_position).xy(),
            (secondary.rigidbody.velocity - primary.rigidbody.velocity)
                .xy()
                .as_dvec2(),
            primary.rigidbody.mass,
            secondary.rigidbody.mass,
        );
        let translation = primary.global_transform.translation().xy();
        let color = match materials.get(secondary.mesh_material) {
            Some(material) => material.color,
            None => Color::from(LIGHT_GRAY),
        };
        orbit.render(&mut gizmos, &translation, color, secondary.vessel.is_none());
        // AP and PE markers for controlled vessel.
        let perifocal_unit_vec = orbit.eccentricity_vector / orbit.eccentricity;
        let ap_vec = orbit.apoapsis * -perifocal_unit_vec;
        let pe_vec = orbit.periapsis * perifocal_unit_vec;
        if let Some(vessel) = secondary.vessel
            && vessel.controlled
        {
            gizmos.circle_2d(
                Isometry2d::from_translation(translation + ap_vec.as_vec2()),
                apsis_radius,
                color.with_alpha(1.0),
            );
            gizmos.circle_2d(
                Isometry2d::from_translation(translation + pe_vec.as_vec2()),
                apsis_radius,
                color.with_alpha(1.0),
            );
        }
    }
}
