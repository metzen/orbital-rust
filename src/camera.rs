use bevy::{
    camera::{
        RenderTarget,
        primitives::Aabb,
        visibility::{Layer, RenderLayers},
    },
    ecs::query::QueryData,
    math::DVec2,
    post_process::bloom::Bloom,
    prelude::*,
    render::render_resource::{
        Extent3d, TextureDescriptor, TextureDimension, TextureFormat, TextureUsages,
    },
    window::WindowResized,
};
use big_space::{
    floating_origins::{BigSpace, FloatingOrigin},
    grid::Grid,
    grid::cell::CellCoord,
};
use either::Either;
use leafwing_input_manager::prelude::*;

use crate::{physics::RigidBody, vessel::Vessel};

/// In-game resolution width.
const RES_WIDTH: u32 = 16 * 20;

/// In-game resolution height.
const RES_HEIGHT: u32 = 10 * 20;

// High-res rendering layer.
pub const HIGH_RES_LAYER: Layer = 1;

#[derive(Default, PartialEq, Copy, Clone, Debug)]
enum CameraViewMode {
    // The camera is aligned with the body (planet, moon, or sun) you are in orbit of, keeping it "below" you in the view.
    Free,
    // The camera rotates with the craft's attitude.
    Locked,
    // The camera follows the surface-based prograde direction.
    Chase,
    // The camera is aligned fixed cardinal orientation in space (like a map), rather than the planet.
    #[default]
    Orbital,
    // The camera switches between free and orbital when vessel is in a stable or hyperbolic orbit.
    Auto,
}

impl CameraViewMode {
    const VALUES: [Self; 4] = [Self::Free, Self::Locked, Self::Chase, Self::Orbital];
}

/// Camera that renders the pixel-perfect world to the [`Canvas`].
#[derive(Component, Default)]
pub struct InGameCamera {
    view_mode: CameraViewMode,
}

/// Camera that renders the [`Canvas`] (and other graphics on [`HIGH_RES_LAYER`]) to the screen.
#[derive(Component)]
struct OuterCamera;

/// Entities with this component will scale up to acheive a minimum rendered size
/// as specified when the scaling of the viewport is changed.
#[derive(Component, Reflect)]
pub struct Autoscale {
    pub minimum_size: f32,
}

impl Autoscale {
    /// Create an Autoscale component with the given minimum scaling size.
    pub fn new(minimum_size: f32) -> Self {
        Self { minimum_size}
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
            .with_dual_axis(Self::Pan, GamepadStick::RIGHT.with_deadzone_symmetric(0.2))
            .with_dual_axis(Self::Pan, VirtualDPad::arrow_keys())
            .with(Self::ZoomIn, KeyCode::Equal)
            .with(Self::ZoomIn, KeyCode::NumpadAdd)
            // .with(Self::ZoomIn, MouseScrollDirection::UP)
            .with(Self::ZoomIn, GamepadButton::RightThumb)
            .with(Self::ZoomOut, KeyCode::Minus)
            .with(Self::ZoomOut, KeyCode::NumpadSubtract)
            // .with(Self::ZoomOut, MouseScrollDirection::DOWN)
            .with(Self::ZoomOut, GamepadButton::LeftThumb)
            .with(Self::NextViewMode, KeyCode::KeyV)
            .with(Self::FocusNext, KeyCode::Tab)
            .with(
                Self::FocusPrev,
                ButtonlikeChord::new([KeyCode::ShiftLeft, KeyCode::Tab]),
            )
            .with(Self::FocusControlledVessel, KeyCode::Backquote)
            .with(Self::FocusNone, KeyCode::F12)
    }
}

pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(InputManagerPlugin::<CameraAction>::default());
        app.init_resource::<ActionState<CameraAction>>();
        app.insert_resource(CameraAction::default_input_map());
        app.register_type::<Autoscale>();
        app.add_systems(PostStartup, setup_camera);
        app.add_systems(Update, (fit_canvas, change_focus));
        app.add_systems(
            PostUpdate,
            (
                update_camera_position_for_autofollow.before(TransformSystems::Propagate),
                camera_control.before(TransformSystems::Propagate),
                scale_entities.before(TransformSystems::Propagate),
            ),
        );
    }
}

fn setup_camera(
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
    big_space: Single<Entity, With<BigSpace>>,
    vessel_query: Query<Entity, With<Vessel>>,
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

    // The "outer" camera, which renders whatever is on `HIGH_RES_LAYER` to the screen.
    //
    // By default, Egui will render to the first Camera created by an application, so this
    // one should be spawned first.
    commands.spawn((
        Camera2d,
        OuterCamera,
        Projection::from(OrthographicProjection::default_2d()),
        RenderLayers::layer(HIGH_RES_LAYER),
    ));
    // A sprite in the high-res layer which is effectively a canvas/billboard to which the
    // in-game camera renders.
    commands.spawn((
        Sprite::from_image(image_handle.clone()),
        Canvas,
        RenderLayers::layer(HIGH_RES_LAYER),
    ));

    // This camera renders whatever is on `PIXEL_PERFECT_LAYERS` to the canvas.
    commands.spawn((
        Camera {
            // render before the "main pass" camera
            order: -1,
            target: RenderTarget::from(image_handle.clone()),
            // hdr: true,
            ..default()
        },
        Camera2d,
        Msaa::Off,
        Projection::from(OrthographicProjection::default_2d()),
        InGameCamera::default(),
        FloatingOrigin,
        HighPrecisionScale(1.0),
        CellCoord::default(),
        Autofollow {
            target: vessel_query.iter().sort::<Entity>().next(),
        },
        Bloom::OLD_SCHOOL,
        SpatialListener::new(100.0),
        // Put the in game camera inside the BigSpace.
        ChildOf(*big_space),
    ));
}

/// Scales camera projection to fit the window (integer multiples only).
fn fit_canvas(
    mut resize_events: EventReader<WindowResized>,
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
    mut camera: Query<(&mut Transform, &mut CellCoord, &Autofollow, &InGameCamera)>,
    player: Query<(&Transform, &CellCoord, &RigidBody), Without<InGameCamera>>,
    grid: Single<&Grid, With<BigSpace>>,
) {
    let Ok((mut camera_transform, mut camera_grid_cell, autofollow, in_game_camera)) =
        camera.single_mut()
    else {
        return;
    };
    let Some(target_entity) = autofollow.target else {
        camera_transform.rotation = Quat::default();
        return;
    };
    let target = player.get(target_entity);
    let Ok((target_transform, target_grid_cell, rigidbody)) = target else {
        return;
    };
    camera_transform.translation = target_transform.translation;
    camera_transform.rotation = match in_game_camera.view_mode {
        CameraViewMode::Orbital => Quat::default(),
        CameraViewMode::Free => {
            if let Some(primary) = rigidbody.primary
                && let Ok((primary_transform, primary_gridcell, _primary_rigidbody)) =
                    player.get(primary)
            {
                let target_position = grid.grid_position_double(target_grid_cell, target_transform);
                let primary_position =
                    grid.grid_position_double(primary_gridcell, primary_transform);
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
        CameraViewMode::Auto => Quat::default(),  // TODO
    };
    *camera_grid_cell = *target_grid_cell;

    // let Vec3 { x, y, .. } = player.translation;
    // let direction = Vec3::new(x, y, camera.translation.z);

    // // Applies a smooth effect to camera movement using interpolation between
    // // the camera position and the player position on the x and y axes.
    // // Here we use the in-game time, to get the elapsed time (in seconds)
    // // since the previous update. This avoids jittery movement when tracking
    // // the player.
    // camera.translation = camera
    //     .translation
    //     .lerp(direction, time.delta_seconds() * CAM_LERP_FACTOR);
}

#[derive(Component)]
pub struct HighPrecisionScale(pub f64);

#[derive(QueryData)]
#[query_data(mutable)]
struct CameraQueryData {
    entity: Entity,
    transform: &'static mut Transform,
    projection: &'static mut Projection,
    grid_cell: &'static mut CellCoord,
    scale: &'static mut HighPrecisionScale,
    in_game_camera: &'static mut InGameCamera,
}

fn camera_control(
    action_state: Res<ActionState<CameraAction>>,
    mut query: Query<CameraQueryData, With<InGameCamera>>,
    time: Res<Time<Real>>,
    // frames: ReferenceFrames<i32>,
) {
    for mut camera in query.iter_mut() {
        let Projection::Orthographic(orthographic_projection) = &mut *camera.projection else {
            continue;
        };
        if action_state.axis_pair(&CameraAction::Pan) != Vec2::ZERO {
            let delta = action_state.clamped_axis_pair(&CameraAction::Pan)
                * orthographic_projection.scale
                * time.delta_secs()
                * 200.0;
            camera.transform.translation += Vec3::new(delta.x, delta.y, 0.0);
        }

        // let Some(reference_frame) = frames.parent_frame(camera.entity) else {
        //     continue;
        // };
        // if keyboard_input.pressed(KeyCode::ArrowRight) {
        //     // Example from https://github.com/aevyrie/big_space/blob/main/src/camera.rs
        //     // Calculates a high precision translation using a f64 movement, and then
        //     // converts it into a grid cell and low precision translation.
        //     //
        //     // let translation_next = DVec3 {
        //     //     x: 2.0 * scale.0,
        //     //     y: 0.0,
        //     //     z: 0.0,
        //     // };
        //     // let (cell_offset, new_translation) =
        //     //     reference_frame.translation_to_grid(translation_next);
        //     // info!(
        //     //     "Grid cell: {:?}, cell_offset: {:?}, next: {}, new_translation: {}",
        //     //     grid_cell, cell_offset, translation_next, new_translation
        //     // );
        //     // *grid_cell += cell_offset;
        //     // transform.translation += new_translation;
        //     // info!("transform: {:?}", transform);
        //     camera.transform.translation.x += camera.projection.scale * time.delta_secs() * 200.0;
        // }

        let scale_factor: f64 = 5.0;
        if action_state.pressed(&CameraAction::ZoomIn) {
            orthographic_projection.scale *= (1.0 - scale_factor * time.delta_secs_f64()) as f32;
            camera.scale.0 *= 1.0 - scale_factor * time.delta_secs_f64();
        }
        if action_state.pressed(&CameraAction::ZoomOut) {
            orthographic_projection.scale *= (1.0 + scale_factor * time.delta_secs_f64()) as f32;
            camera.scale.0 *= 1.0 + scale_factor * time.delta_secs_f64();
        }

        if action_state.just_pressed(&CameraAction::NextViewMode) {
            let index = CameraViewMode::VALUES
                .iter()
                .position(|m| m == &camera.in_game_camera.view_mode)
                .unwrap();
            camera.in_game_camera.view_mode =
                CameraViewMode::VALUES[(index + 1) % CameraViewMode::VALUES.len()];
            info!("Camera mode: {:?}", camera.in_game_camera.view_mode);
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
                (orthographic_projection.scale / aabb.half_extents.x).max(1.0),
                (orthographic_projection.scale / aabb.half_extents.y).max(1.0),
                1.0,
            ) * autoscale.minimum_size;
        }
    }
}

#[derive(Component)]
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
            Either::Left(focus_targets_query.iter().sort::<Entity>())
        } else {
            Either::Right(focus_targets_query.iter().sort::<Entity>().rev())
        };
        let mut peekable = iter.peekable();
        let Some(&(first_target, first_name)) = peekable.peek() else {
            return;
        };
        while let Some((target, name)) = peekable.next() {
            if autofollow.target.is_some() {
                if autofollow.target.unwrap() == target {
                    if let Some((next_target, next_name)) = peekable.next() {
                        autofollow.target = Some(next_target);
                        info!("focusing {}", next_name);
                    } else {
                        autofollow.target = Some(first_target);
                        info!("focusing {}", first_name);
                    }
                    break;
                }
            } else {
                autofollow.target = Some(target);
                info!("focusing {}", name);
                break;
            }
        }
    }
}
