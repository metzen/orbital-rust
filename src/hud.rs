mod altitude;
mod orbit_info;
mod sas_selector;
mod subject;
mod throttle;
mod time;
mod velocity;
mod vertical_speed;

use bevy::color::palettes::css::{BLACK, LIGHT_GRAY};
use bevy::ecs::query::QueryData;
use bevy::ecs::system::lifetimeless::Read;
use bevy::input::common_conditions::input_toggle_active;
use bevy::math::DVec2;
use bevy::picking::pointer::PointerInteraction;
use bevy::prelude::*;
use big_space::floating_origins::BigSpace;
use big_space::grid::Grid;
use big_space::grid::cell::CellCoord;
use leafwing_input_manager::Actionlike;
use leafwing_input_manager::common_conditions::action_just_pressed;
use leafwing_input_manager::plugin::InputManagerPlugin;
use leafwing_input_manager::prelude::{ActionState, InputMap};

use crate::camera::{Autofollow, InGameCamera, InGamePointer};
use crate::hud::altitude::AltitudePlugin;
use crate::hud::orbit_info::OrbitInfoPlugin;
use crate::hud::sas_selector::SasSelectorPlugin;
use crate::hud::subject::SubjectPlugin;
use crate::hud::throttle::ThrottlePlugin;
use crate::hud::time::TimePlugin;
use crate::hud::velocity::VelocityPlugin;
use crate::hud::vertical_speed::VerticalSpeedPlugin;
use crate::physics::{NoGravity, Orbit, OrbitShape, RigidBody, SatelliteOf};
use crate::vessel::Vessel;

pub struct HudPlugin;

impl Plugin for HudPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, (setup_hud, setup_gizmos));
        app.add_systems(
            FixedUpdate,
            (update_hover_text, update_sas_indicator_widget),
        );
        app.add_systems(
            Update,
            (
                next_vessel.run_if(action_just_pressed(HudAction::NextVessel)),
                previous_vessel.run_if(action_just_pressed(HudAction::PreviousVessel)),
            ),
        );
        app.add_systems(
            PostUpdate,
            draw_orbits
                .after(TransformSystems::Propagate)
                .run_if(input_toggle_active(true, KeyCode::F8)),
        );
        app.add_plugins((
            AltitudePlugin,
            InputManagerPlugin::<HudAction>::default(),
            OrbitInfoPlugin,
            SasSelectorPlugin,
            SubjectPlugin,
            ThrottlePlugin,
            TimePlugin,
            VelocityPlugin,
            VerticalSpeedPlugin,
        ));
        app.init_resource::<ActionState<HudAction>>();
        app.insert_resource(HudAction::default_input_map());
    }
}

#[derive(Component)]
pub struct HudSubject;

#[derive(Component)]
pub struct HoverText;

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
    setup_rotation_widget(&mut commands);
    setup_hover_text(&mut commands);
    setup_staging_widget(&mut commands);
    setup_sas_indicator_widget(&mut commands);
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

fn change_hud_subject(
    mut commands: Commands,
    mut vessels_query: Query<(Entity, &mut Vessel, Option<&HudSubject>), With<Vessel>>,
    mut camera_autofollow: Single<&mut Autofollow, With<InGameCamera>>,
    modifier: i32,
) {
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

    let new_subject_index = (current_subject_index + modifier).rem_euclid(i);
    let new_subject = entities[new_subject_index as usize];
    commands.entity(new_subject).insert(HudSubject);
    if let Ok((entity, mut vessel, _hud_subject)) = vessels_query.get_mut(new_subject) {
        vessel.controlled = true;
        camera_autofollow.target = Some(entity);
    }
}

fn next_vessel(
    commands: Commands,
    vessels_query: Query<(Entity, &mut Vessel, Option<&HudSubject>), With<Vessel>>,
    camera_autofollow: Single<&mut Autofollow, With<InGameCamera>>,
) {
    change_hud_subject(commands, vessels_query, camera_autofollow, 1);
}

fn previous_vessel(
    commands: Commands,
    vessels_query: Query<(Entity, &mut Vessel, Option<&HudSubject>), With<Vessel>>,
    camera_autofollow: Single<&mut Autofollow, With<InGameCamera>>,
) {
    change_hud_subject(commands, vessels_query, camera_autofollow, -1);
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
            use bevy_gizmos_ext::GizmosExt;
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
        use bevy_gizmos_ext::GizmosExt;
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
