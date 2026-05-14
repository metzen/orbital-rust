use bevy::asset::uuid::Uuid;
use bevy::camera::primitives::Aabb;
use bevy::camera::visibility::{Layer, RenderLayers};
use bevy::camera::{ImageRenderTarget, RenderTarget};
use bevy::ecs::message::MessageCursor;
use bevy::math::DVec2;
use bevy::picking::PickingSystems;
use bevy::picking::pointer::{Location, PointerId, PointerInput, PointerInteraction};
use bevy::post_process::bloom::Bloom;
use bevy::prelude::*;
use bevy::render::render_resource::{
    Extent3d, TextureDescriptor, TextureDimension, TextureFormat, TextureUsages,
};
use bevy::window::{PrimaryWindow, WindowResized};
use bevy_egui::input::egui_wants_any_pointer_input;
use bevy_egui::{EguiGlobalSettings, EguiStartupSet, PrimaryEguiContext};
use big_space::floating_origins::{BigSpace, FloatingOrigin};
use big_space::grid::Grid;
use big_space::grid::cell::CellCoord;
use either::Either;
use leafwing_input_manager::prelude::*;

use crate::physics::{CelestialBody, Orbit, RigidBody, SatelliteOf};
use crate::rendering::LayerExt;
use crate::vessel::Vessel;

/// In-game resolution width.
const RES_WIDTH: u32 = 16 * 20;

/// In-game resolution height.
const RES_HEIGHT: u32 = 10 * 20;

const PIXEL_CAM_POINTER_ID: PointerId =
    PointerId::Custom(Uuid::from_u128(0x230e8400e29b41d1a716446655446439));

/// The maximum rate at which the camera zooms (in scale factor per second).
const CAMERA_ZOOM_RATE_MAX: f32 = 5.0;

#[derive(Default, PartialEq, Copy, Clone, Debug)]
enum CameraViewMode {
    // The camera is aligned with the body (planet, moon, or sun) you are in orbit of, keeping it "below" you in the view.
    Free,
    // The camera rotates with the craft's attitude.
    Locked,
    // The camera follows the surface-based prograde direction.
    Chase,
    // The camera is aligned with a fixed cardinal orientation in space (like a map), rather than the planet.
    Orbital,
    // The camera switches between free and orbital when vessel is in a stable or hyperbolic orbit.
    #[default]
    Auto,
}

impl CameraViewMode {
    fn next(self) -> CameraViewMode {
        match self {
            CameraViewMode::Free => CameraViewMode::Locked,
            CameraViewMode::Locked => CameraViewMode::Chase,
            CameraViewMode::Chase => CameraViewMode::Orbital,
            CameraViewMode::Orbital => CameraViewMode::Auto,
            CameraViewMode::Auto => CameraViewMode::Free,
        }
    }
}

/// Camera that renders the pixel-perfect world to the [`Canvas`].
#[derive(Component, Default)]
pub struct InGameCamera {
    view_mode: CameraViewMode,
}

#[derive(Component)]
pub struct InGamePointer;

/// Camera that renders the [`Canvas`] (and other graphics on [`Layer::MAIN`]) to the screen.
#[derive(Component)]
struct OuterCamera;

/// Marker for entities that should have their projection scaled via camera zoom actions.
#[derive(Component)]
struct ProjectionScaleZoom;

/// Entities with this component will scale up to achieve a minimum rendered size
/// as specified when the scaling of the viewport is changed.
#[derive(Component, Reflect)]
pub struct Autoscale {
    pub minimum_size: f32,
}

impl Autoscale {
    /// Create an Autoscale component with the given minimum scaling size.
    pub fn new(minimum_size: f32) -> Self {
        Self { minimum_size }
    }
}

impl Default for Autoscale {
    fn default() -> Self {
        Self { minimum_size: 1.0 }
    }
}

/// Low-resolution texture that contains the pixel-perfect world.
/// Canvas itself is rendered to the high-resolution world.
#[derive(Component)]
struct Canvas;

/// Entities with this component are able to hold camera focus.
#[derive(Component, Default)]
pub struct Focusable;

#[derive(Actionlike, PartialEq, Eq, Clone, Copy, Hash, Debug, Reflect)]
enum CameraAction {
    #[actionlike(DualAxis)]
    Pan,
    #[actionlike(Axis)]
    Zoom,
    ZoomIn,
    ZoomOut,
    NextViewMode,
    FocusNext,
    FocusPrev,
    FocusControlledVessel,
    FocusNone,
}

impl CameraAction {
    fn default_input_map() -> InputMap<Self> {
        InputMap::default()
            .with_dual_axis(Self::Pan, VirtualDPad::arrow_keys())
            .with_axis(Self::Zoom, VirtualAxis::new(KeyCode::Minus, KeyCode::Equal))
            .with_axis(
                Self::Zoom,
                GamepadControlAxis::RIGHT_Y.with_deadzone_symmetric(0.2),
            )
            .with_axis(
                Self::Zoom,
                VirtualAxis::new(KeyCode::NumpadSubtract, KeyCode::NumpadAdd),
            )
            .with(Self::ZoomIn, MouseScrollDirection::UP)
            .with(Self::ZoomOut, MouseScrollDirection::DOWN)
            .with(Self::NextViewMode, KeyCode::KeyV)
            .with(Self::FocusNext, KeyCode::Tab)
            .with(
                Self::FocusPrev,
                ButtonlikeChord::new([KeyCode::ShiftLeft, KeyCode::Tab]),
            )
            .with(Self::FocusControlledVessel, KeyCode::Backquote)
            .with(Self::FocusNone, KeyCode::F4)
    }
}

pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(InputManagerPlugin::<CameraAction>::default());
        app.init_resource::<ActionState<CameraAction>>();
        app.insert_resource(CameraAction::default_input_map());
        app.register_type::<Autoscale>();
        app.add_systems(
            PreStartup,
            setup_outer_camera.before(EguiStartupSet::InitContexts),
        );
        app.add_systems(Startup, setup_pointer);
        app.add_systems(PostStartup, setup_camera);
        app.add_systems(
            First,
            relay_pointer_input_messages
                .in_set(PickingSystems::PostInput)
                .run_if(not(egui_wants_any_pointer_input)),
        );
        app.add_systems(Update, (fit_canvas, change_focus, change_focus_on_click));
        app.add_systems(
            PostUpdate,
            (
                update_camera_position_for_autofollow.before(TransformSystems::Propagate),
                camera_control.before(TransformSystems::Propagate),
                zoom.before(TransformSystems::Propagate),
                scale_entities.before(TransformSystems::Propagate),
            ),
        );
    }
}

struct RenderOrder(isize);

#[extension(trait CameraExt)]
impl Camera {
    fn new(order: RenderOrder, clear_color: ClearColorConfig) -> Self {
        Self {
            order: order.0,
            clear_color,
            ..default()
        }
    }
}

/// Spawns the "outer" camera, which renders whatever is on [`Layer::MAIN`] to the screen.
fn setup_outer_camera(
    mut commands: Commands,
    mut egui_global_settings: ResMut<EguiGlobalSettings>,
) {
    // Disable the automatic creation of a primary context to set it up manually.
    egui_global_settings.auto_create_primary_context = false;
    commands.spawn((
        Camera::new(RenderOrder(1), ClearColorConfig::Default),
        Camera2d,
        OuterCamera,
        Projection::from(OrthographicProjection::default_2d()),
        RenderLayers::layer(Layer::MAIN),
        PrimaryEguiContext,
        IsDefaultUiCamera,
    ));
}

fn setup_camera(
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
    big_space: Single<Entity, With<BigSpace>>,
    vessel_query: Query<(Entity, &Vessel)>,
) {
    let canvas_size = Extent3d {
        width: RES_WIDTH,
        height: RES_HEIGHT,
        ..default()
    };

    // this Image serves as a canvas representing the low-resolution game screen
    let mut canvas = Image {
        texture_descriptor: TextureDescriptor {
            label: None,
            size: canvas_size,
            dimension: TextureDimension::D2,
            format: TextureFormat::Bgra8UnormSrgb,
            mip_level_count: 1,
            sample_count: 1,
            usage: TextureUsages::TEXTURE_BINDING
                | TextureUsages::COPY_DST
                | TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        },
        ..default()
    };

    // fill image.data with zeroes
    canvas.resize(canvas_size);

    let image_handle = images.add(canvas);

    // A sprite in the high-res layer which is effectively a canvas/billboard to which the
    // in-game camera renders.
    commands.spawn((
        Sprite::from_image(image_handle.clone()),
        Canvas,
        RenderLayers::layer(Layer::MAIN),
    ));

    // This camera renders whatever is on `PIXEL_PERFECT_LAYERS` to the canvas.
    commands.spawn((
        Camera::new(RenderOrder(0), ClearColorConfig::None),
        Camera2d,
        RenderTarget::from(image_handle.clone()),
        RenderLayers::layer(Layer::FOREGROUND),
        Msaa::Off,
        Projection::from(OrthographicProjection::default_2d()),
        InGameCamera::default(),
        ProjectionScaleZoom,
        FloatingOrigin,
        CellCoord::default(),
        Autofollow {
            target: vessel_query
                .iter()
                .filter(|(_, vessel)| vessel.controlled)
                .map(|(entity, _)| entity)
                .next(),
        },
        Bloom::OLD_SCHOOL,
        SpatialListener::new(100.0),
        // Put the in game camera inside the BigSpace.
        ChildOf(*big_space),
        children![
            (
                Name::new("OrbitGizmosCamera"),
                Camera::new(RenderOrder(-1), ClearColorConfig::None),
                Camera2d,
                RenderTarget::from(image_handle.clone()),
                RenderLayers::layer(Layer::ORBIT),
                Projection::from(OrthographicProjection::default_2d()),
                ProjectionScaleZoom,
                Bloom::OLD_SCHOOL,
                Msaa::Off,
            ),
            (
                Name::new("BackgroundCamera"),
                Camera::new(RenderOrder(-2), ClearColorConfig::Default),
                Camera2d,
                RenderTarget::from(image_handle.clone()),
                RenderLayers::layer(Layer::BACKGROUND),
                Projection::from(OrthographicProjection::default_2d()),
                ProjectionScaleZoom,
                Bloom::OLD_SCHOOL,
                Msaa::Off,
            )
        ],
    ));
}

/// Scales camera projection to fit the window (integer multiples only).
fn fit_canvas(
    mut resize_events: MessageReader<WindowResized>,
    mut projection: Single<&mut Projection, With<OuterCamera>>,
) {
    for event in resize_events.read() {
        let h_scale = event.width / RES_WIDTH as f32;
        let v_scale = event.height / RES_HEIGHT as f32;
        if let Projection::Orthographic(orthographic_projection) = &mut **projection {
            orthographic_projection.scale = 0.15;
            info!("{:?}", event);
            orthographic_projection.scale = 1. / h_scale.min(v_scale);
        }
    }
}

fn update_camera_position_for_autofollow(
    grid: Single<&Grid, With<BigSpace>>,
    camera: Single<(&mut Transform, &mut CellCoord, &Autofollow, &InGameCamera)>,
    position_query: Query<(&Transform, &CellCoord), Without<InGameCamera>>,
    rigidbody_query: Query<&RigidBody>,
    celestial_body_query: Query<&CelestialBody>,
    satellite_of_query: Query<&SatelliteOf>,
) {
    let (mut camera_transform, mut camera_cell, autofollow, in_game_camera) = camera.into_inner();
    let Some(target_entity) = autofollow.target else {
        camera_transform.rotation = Quat::default();
        return;
    };
    let (target_transform, target_cell) = position_query.get(target_entity).unwrap();
    let Ok(target_rigidbody) = rigidbody_query.get(target_entity) else {
        return;
    };
    *camera_cell = *target_cell;
    camera_transform.translation = target_transform.translation;
    camera_transform.rotation = match in_game_camera.view_mode {
        CameraViewMode::Orbital => Quat::default(),
        CameraViewMode::Free => {
            if let Ok(satellite_of) = satellite_of_query.get(target_entity)
                && let Ok((primary_transform, primary_cell)) =
                    position_query.get(satellite_of.primary())
            {
                let target_position = grid.grid_position_double(target_cell, target_transform);
                let primary_position = grid.grid_position_double(primary_cell, primary_transform);
                let direction = target_position - primary_position;
                Quat::from_rotation_z(-direction.xy().normalize().angle_to(DVec2::Y) as f32)
            } else {
                Quat::default()
            }
        }
        CameraViewMode::Locked => camera_transform
            .rotation
            .lerp(target_transform.rotation, 0.1),
        CameraViewMode::Chase => Quat::default(), // TODO
        CameraViewMode::Auto => {
            if let Ok(satellite_of) = satellite_of_query.get(target_entity)
                && let primary = satellite_of.primary()
                && let Ok((primary_transform, primary_cell)) =
                    position_query.get(satellite_of.primary())
                && let Ok(primary_rigidbody) = rigidbody_query.get(primary)
                && let Ok(primary_celestial_body) = celestial_body_query.get(primary)
            {
                let target_position = grid.grid_position_double(target_cell, target_transform);
                let primary_position = grid.grid_position_double(primary_cell, primary_transform);
                let orbit = Orbit::new(
                    (target_position - primary_position).xy(),
                    (target_rigidbody.velocity - primary_rigidbody.velocity)
                        .xy()
                        .as_dvec2(),
                    primary_rigidbody.mass,
                    target_rigidbody.mass,
                );
                if (orbit.periapsis - primary_celestial_body.radius as f64) < 0.0
                    && (orbit.apoapsis - primary_celestial_body.radius as f64) > 0.0
                {
                    // Free mode.
                    let direction = target_position - primary_position;
                    Quat::from_rotation_z(-direction.xy().normalize().angle_to(DVec2::Y) as f32)
                } else {
                    // Orbital mode.
                    Quat::default()
                }
            } else {
                Quat::default()
            }
        }
    };
}

fn camera_control(
    action_state: Res<ActionState<CameraAction>>,
    camera: Single<(&mut Transform, &mut Projection, &mut InGameCamera)>,
    time: Res<Time<Real>>,
) {
    let (mut transform, mut projection, mut in_game_camera) = camera.into_inner();
    let Projection::Orthographic(ref mut orthographic_projection) = *projection else {
        todo!("Handle non-orthographic camera projection");
    };
    if action_state.axis_pair(&CameraAction::Pan) != Vec2::ZERO {
        let delta = action_state.clamped_axis_pair(&CameraAction::Pan)
            * orthographic_projection.scale
            * time.delta_secs()
            * 200.0;
        transform.translation += Vec3::new(delta.x, delta.y, 0.0);
    }

    if action_state.just_pressed(&CameraAction::NextViewMode) {
        in_game_camera.view_mode = in_game_camera.view_mode.next();
        info!("Camera mode: {:?}", in_game_camera.view_mode);
    }
}

fn zoom(
    action_state: Res<ActionState<CameraAction>>,
    time: Res<Time<Real>>,
    projections: Query<&mut Projection, With<ProjectionScaleZoom>>,
) {
    for mut projection in projections {
        if let Projection::Orthographic(ref mut orthographic_projection) = *projection {
            if action_state.pressed(&CameraAction::ZoomIn) {
                orthographic_projection.scale *=
                    1.0 - CAMERA_ZOOM_RATE_MAX * 10.0 * time.delta_secs();
            }
            if action_state.pressed(&CameraAction::ZoomOut) {
                orthographic_projection.scale *=
                    1.0 + CAMERA_ZOOM_RATE_MAX * 10.0 * time.delta_secs();
            }
            if action_state.clamped_value(&CameraAction::Zoom) != 0.0 {
                orthographic_projection.scale *= 1.0
                    + CAMERA_ZOOM_RATE_MAX
                        * -action_state.clamped_value(&CameraAction::Zoom)
                        * time.delta_secs();
            }
        }
    }
}

/// Scale entities up if they end up becoming smaller than the minimum size in the current projection scale.
fn scale_entities(
    mut query: Query<(&mut Transform, &Aabb, &Autoscale)>,
    projection: Single<&Projection, With<InGameCamera>>,
) {
    if let Projection::Orthographic(orthographic_projection) = *projection {
        for (mut transform, aabb, autoscale) in query.iter_mut() {
            transform.scale = Vec3::new(
                f32::max(
                    orthographic_projection.scale / aabb.half_extents.x * autoscale.minimum_size,
                    1.0,
                ),
                f32::max(
                    orthographic_projection.scale / aabb.half_extents.y * autoscale.minimum_size,
                    1.0,
                ),
                1.0,
            );
        }
    }
}

#[derive(Component, Reflect)]
pub struct Autofollow {
    pub target: Option<Entity>,
}

fn change_focus(
    action_state: Res<ActionState<CameraAction>>,
    mut autofollow: Single<&mut Autofollow, With<InGameCamera>>,
    focus_targets_query: Query<(Entity, &Name), With<Focusable>>,
    vessels: Query<(Entity, &Vessel), With<Vessel>>,
) {
    if action_state.just_pressed(&CameraAction::FocusNone) {
        autofollow.target = None;
    }
    if action_state.just_pressed(&CameraAction::FocusControlledVessel) {
        for (vessel_id, vessel) in vessels {
            if vessel.controlled {
                autofollow.target = Some(vessel_id);
                break;
            }
        }
    }
    if action_state.just_pressed(&CameraAction::FocusNext)
        || action_state.just_pressed(&CameraAction::FocusPrev)
    {
        let iter = if action_state.just_pressed(&CameraAction::FocusNext) {
            Either::Left(focus_targets_query.iter().sort::<Entity>().rev())
        } else {
            Either::Right(focus_targets_query.iter().sort::<Entity>())
        };
        let mut peekable = iter.peekable();
        let Some(&(first_target, first_name)) = peekable.peek() else {
            return;
        };
        if let Some(current_target) = autofollow.target {
            while let Some((target, _name)) = peekable.next() {
                if current_target == target {
                    if let Some((next_target, next_name)) = peekable.next() {
                        autofollow.target = Some(next_target);
                        info!("focusing {}", next_name);
                    } else {
                        autofollow.target = Some(first_target);
                        info!("focusing {}", first_name);
                    }
                    break;
                }
            }
        } else {
            autofollow.target = Some(first_target);
            info!("focusing {}", first_name);
        }
    }
}

fn setup_pointer(mut commands: Commands) {
    commands.spawn((PIXEL_CAM_POINTER_ID, InGamePointer));
}

/// Relay PointerInput messages from window-based mouse inputs into the pixel-perfect canvas.
fn relay_pointer_input_messages(
    mut message_reader: Local<MessageCursor<PointerInput>>,
    mut messages: ResMut<Messages<PointerInput>>,
    render_target: Single<&RenderTarget, With<InGameCamera>>,
    window: Single<&Window, With<PrimaryWindow>>,
) {
    let messages_to_resend: Vec<PointerInput> = message_reader
        .read(&messages)
        .filter(|m| m.pointer_id == PointerId::Mouse)
        .map(|input| {
            PointerInput {
                pointer_id: PIXEL_CAM_POINTER_ID,
                location: Location {
                    target: bevy::camera::NormalizedRenderTarget::Image(ImageRenderTarget {
                        handle: render_target.as_image().unwrap().clone(),
                        scale_factor: 1.0,
                    }),
                    // TODO: This isn't quite correct; need to account for cases where the canvas
                    // does not fill the whole window.
                    position: Vec2::new(
                        (input.location.position.x / window.width()) * RES_WIDTH as f32,
                        (input.location.position.y / window.height()) * RES_HEIGHT as f32,
                    ),
                },
                action: input.action,
            }
        })
        .collect();
    for message in messages_to_resend {
        messages.write(message);
    }
}

/// Focus an entity when clicked.
fn change_focus_on_click(
    mut reader: MessageReader<PointerInput>,
    interactions: Query<&PointerInteraction, With<InGamePointer>>,
    mut autofollow: Single<&mut Autofollow, With<InGameCamera>>,
    focusable_query: Query<Has<Focusable>>,
) {
    for input in reader
        .read()
        .filter(|m| m.pointer_id == PIXEL_CAM_POINTER_ID)
    {
        if input.button_just_pressed(PointerButton::Primary) {
            for interaction in interactions.iter() {
                // TODO: Bubble up to next nearest hit when nearest is not focusable.
                if let Some((entity, _hit)) = interaction.get_nearest_hit()
                    && let Ok(focusable) = focusable_query.get(*entity)
                    && focusable
                {
                    autofollow.target = Some(*entity);
                }
            }
        }
    }
}
