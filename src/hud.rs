use crate::{
    camera::{Autofollow, HIGH_RES_LAYER, InGameCamera},
    physics::RigidBody,
    timewarp::TimeWarp,
    vessel::Vessel,
};
use bevy::{ecs::query::QuerySingleError, prelude::*, render::view::RenderLayers};
use big_space::grid::cell::GridCell;
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

fn setup_hud(mut commands: Commands) {
    let text_font = TextFont {
        font_size: 10.0,
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
    let mut text = query.single_mut().unwrap();
    text.0 = format!("{:.2}", time_warp.value);
}

fn update_throttle(
    mut query: Query<&mut Text, With<ThrottleText>>,
    vessel_query: Query<&Vessel, With<HudSubject>>,
) {
    if let Some(vessel) = vessel_query.iter().next() {
        query.single_mut().unwrap().0 = format!("{:.2}", vessel.throttle);
    }
}

fn update_velocity(
    mut query: Query<&mut Text, With<VelocityText>>,
    vessel_rigidbody_query: Query<&RigidBody, With<HudSubject>>,
) {
    if let Some(rigidbody) = vessel_rigidbody_query.iter().next() {
        query.single_mut().unwrap().0 = format!("{:.2}", rigidbody.velocity);
    }
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
    mut query: Query<&mut Text, With<AltitudeText>>,
    vessel_rigidbody_query: Query<(&Transform, &RigidBody), With<HudSubject>>,
    primary_transform_query: Query<(&Transform, &GridCell)>,
) {
    if let Some((vessel_transform, vessel_rigidbody)) = vessel_rigidbody_query.iter().next()
        && vessel_rigidbody.primary.is_some()
    {
        let Ok((primary_transform, _primary_grid_cell)) =
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
        query.single_mut().unwrap().0 = format!("{:.2}", distance);
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
