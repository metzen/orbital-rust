use bevy::{math::NormedVectorSpace, prelude::*, render::view::RenderLayers};
use big_space::{reference_frame, GridCell, ReferenceFrame};

use crate::{camera::HIGH_RES_LAYER, physics::RigidBody, timewarp::TimeWarp, vessel::Vessel};

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
            ),
        );
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

fn setup_hud(mut commands: Commands, asset_server: Res<AssetServer>) {
    let font = asset_server.load("fonts/Roboto-Medium.ttf");
    let text_font = TextFont {
        font: font.clone(),
        font_size: 10.0,
        font_smoothing: bevy::text::FontSmoothing::AntiAliased,
    };
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
                Text::default(),
                TextLayout {
                    justify: JustifyText::Right,
                    linebreak: LineBreak::NoWrap,
                },
                text_font.clone(),
            ))
            .with_children(|time_warp| {
                time_warp.spawn((TextSpan::new("TIME.WARP: "), text_font.clone()));
                time_warp.spawn((TextSpan::default(), TimeWarpText, text_font.clone()));
            });
            root.spawn((
                Text::new("THR: "),
                RenderLayers::layer(HIGH_RES_LAYER),
                text_font.clone(),
            ));
            root.spawn((
                Text::default(),
                ThrottleText,
                RenderLayers::layer(HIGH_RES_LAYER),
                text_font.clone(),
            ));
            root.spawn((
                Text::new("VEL: "),
                RenderLayers::layer(HIGH_RES_LAYER),
                text_font.clone(),
            ));
            root.spawn((
                Text::default(),
                VelocityText,
                RenderLayers::layer(HIGH_RES_LAYER),
                text_font.clone(),
            ));
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
            root.spawn((
                Text::new("ALT: "),
                RenderLayers::layer(HIGH_RES_LAYER),
                text_font.clone(),
            ));
            root.spawn((
                Text::default(),
                AltitudeText,
                RenderLayers::layer(HIGH_RES_LAYER),
                text_font.clone(),
            ));
        });
}

fn update_time_warp(mut query: Query<&mut TextSpan, With<TimeWarpText>>, time_warp: Res<TimeWarp>) {
    let mut text = query.single_mut();
    text.0 = format!("{:.2}", time_warp.value);
}

fn update_throttle(
    mut query: Query<&mut Text, With<ThrottleText>>,
    vessel_query: Query<&Vessel, With<HudSubject>>,
) {
    if let Some(vessel) = vessel_query.iter().next() {
        query.single_mut().0 = format!("{:.2}", vessel.throttle);
    }
}

fn update_velocity(
    mut query: Query<&mut Text, With<VelocityText>>,
    vessel_rigidbody_query: Query<&RigidBody, With<HudSubject>>,
) {
    if let Some(rigidbody) = vessel_rigidbody_query.iter().next() {
        query.single_mut().0 = format!("{:.2}", rigidbody.velocity);
    }
}

fn update_acceleration(
    mut query: Query<&mut Text, With<AccelerationText>>,
    vessel_rigidbody_query: Query<&RigidBody, With<HudSubject>>,
) {
    if let Some(rigidbody) = vessel_rigidbody_query.iter().next() {
        query.single_mut().0 = format!("{:.2}", rigidbody.acceleration);
    }
}

fn update_altitude(
    mut query: Query<&mut Text, With<AltitudeText>>,
    vessel_rigidbody_query: Query<(&Transform, &RigidBody), With<HudSubject>>,
    primary_transform_query: Query<(&Transform, &GridCell<i32>)>,
) {
    if let Some((vessel_transform, vessel_rigidbody)) = vessel_rigidbody_query.iter().next() {
        if vessel_rigidbody.primary.is_some() {
            let Ok((primary_transform, primary_grid_cell)) =
                primary_transform_query.get(vessel_rigidbody.primary.unwrap())
            else {
                return;
            };
            // info!("primary: {:?}", primary_transform.translation);
            // info!("vessel: {:?}", vessel_transform.translation);
            // TODO: This needs to account for BigSpace grid_cell difference.
            // let cell_diff = grid_cell - primary_grid_cell;
            // let cell_distance = IVec3::new(cell_diff.x, cell_diff.y, cell_diff.z).distance(IVec3::ZERO);
            let distance = primary_transform
                .translation
                .distance(vessel_transform.translation);
            query.single_mut().0 = format!("{:.2}", distance);
        }
    }
}

pub fn update_hud_subject(
    mut commands: Commands,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut vessels_query: Query<(Entity, &mut Vessel, Option<&HudSubject>), With<Vessel>>,
    subject_query: Query<Entity, With<HudSubject>>,
) {
    if keyboard_input.just_pressed(KeyCode::BracketLeft) {
        info!("backet left");
        for (entity, mut vessel, hud_subject) in vessels_query.iter_mut() {
            info!("hud subj vessel");
            if hud_subject.is_some() {
                info!("vessel is subject");
                commands.entity(entity).remove::<HudSubject>();
                vessel.controlled = false;
            } else {
                info!("vessel is not subject");
                commands.entity(entity).insert(HudSubject);
                vessel.controlled = true;
            }
        }
    }
    if keyboard_input.just_pressed(KeyCode::BracketRight) {}
}
