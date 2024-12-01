use bevy::{
    core_pipeline::bloom::BloomSettings,
    prelude::*,
    render::{
        camera::RenderTarget, mesh::MeshAabb, render_resource::{
            Extent3d, TextureDescriptor, TextureDimension, TextureFormat, TextureUsages,
        }, view::RenderLayers
    },
    window::WindowResized,
};
use big_space::{
    reference_frame::local_origin::ReferenceFrames, BigSpace, FloatingOrigin, GridCell,
};

use crate::vessel::Vessel;

/// In-game resolution width.
pub const RES_WIDTH: u32 = 16 * 12;

/// In-game resolution height.
pub const RES_HEIGHT: u32 = 10 * 12;

/// Default render layers for pixel-perfect rendering.
/// You can skip adding this component, as this is the default.
pub const PIXEL_PERFECT_LAYERS: RenderLayers = RenderLayers::layer(0);

/// Render layers for high-resolution rendering.
pub const HIGH_RES_LAYERS: RenderLayers = RenderLayers::layer(1);

/// Camera that renders the pixel-perfect world to the [`Canvas`].
#[derive(Component)]
pub struct InGameCamera;

/// Camera that renders the [`Canvas`] (and other graphics on [`HIGH_RES_LAYERS`]) to the screen.
#[derive(Component)]
pub struct OuterCamera;

/// Entities with this component will scale up to prevent becoming smaller
/// than the size of one rendered pixel as the scaling of the viewport is
/// changed.
#[derive(Component, Default)]
pub struct Autoscale;

/// Low-resolution texture that contains the pixel-perfect world.
/// Canvas itself is rendered to the high-resolution world.
#[derive(Component)]
struct Canvas;

/// Entities with this component are able to hold camera focus.
#[derive(Component, Default)]
pub struct Focusable;

pub fn setup_camera(
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
    query: Query<Entity, With<BigSpace>>,
    vessel_query: Query<Entity, With<Vessel>>,
) {
    let big_space = query.single();
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

    // this camera renders whatever is on `PIXEL_PERFECT_LAYERS` to the canvas
    let in_game_camera = commands
        .spawn((
            // Camera2d::default(),
            Camera2dBundle {
                camera: Camera {
                    // render before the "main pass" camera
                    order: -1,
                    target: RenderTarget::Image(image_handle.clone()),
                    hdr: true,
                    ..default()
                },
                msaa: Msaa::Off,
                projection: OrthographicProjection::default_2d(),
                // projection: OrthographicProjection {
                //     scale: 2.0,
                //     // scale: 1e9,  // Solar system view.
                //     far: 1000.,
                //     near: -1000.,

                //     // ..default()
                // },
                ..default()
            },
            InGameCamera,
            FloatingOrigin,
            HighPrecisionScale(1.0),
            GridCell::<i32>::default(),
            Autofollow {
                target: vessel_query.iter().next(),
            },
            BloomSettings::OLD_SCHOOL,
            SpatialListener::new(100.0),
        ))
        .id();

    // Put the in game camera inside the BigSpace.
    commands.entity(in_game_camera).set_parent(big_space);

    commands.spawn((
        Sprite {
            image: image_handle,
            ..default()
        },
        Canvas,
        HIGH_RES_LAYERS,
    ));

    // the "outer" camera renders whatever is on `HIGH_RES_LAYERS` to the screen.
    // here, the canvas and one of the sample sprites will be rendered by this camera
    commands.spawn((Camera2dBundle::default(), OuterCamera, HIGH_RES_LAYERS));
}

/// Scales camera projection to fit the window (integer multiples only).
pub fn fit_canvas(
    mut resize_events: EventReader<WindowResized>,
    mut projections: Query<&mut OrthographicProjection, With<OuterCamera>>,
) {
    for event in resize_events.read() {
        let h_scale = event.width / RES_WIDTH as f32;
        let v_scale = event.height / RES_HEIGHT as f32;
        let mut projection = projections.single_mut();
        projection.scale = 0.2;
        // projection.scale = 1. / h_scale.min(v_scale);
    }
}

pub fn update_camera_position_for_autofollow(
    mut camera: Query<(&mut Transform, &mut GridCell<i32>, &Autofollow), With<InGameCamera>>,
    player: Query<(&Transform, &GridCell<i32>), Without<InGameCamera>>,
) {
    let Ok(camera) = camera.get_single_mut() else {
        return;
    };

    if camera.2.target.is_none() {
        return;
    }

    let (mut camera_transform, mut camera_grid_cell, autofollow) = camera;
    let target = player.get(autofollow.target.unwrap());
    let Ok((target_transform, target_grid_cell)) = target else {
        return;
    };
    camera_transform.translation = target_transform.translation;
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

pub fn camera_control(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut query: Query<
        (
            Entity,
            &mut Transform,
            &mut OrthographicProjection,
            &mut GridCell<i32>,
            &mut HighPrecisionScale,
        ),
        With<InGameCamera>,
    >,
    frames: ReferenceFrames<i32>,
    time: Res<Time>,
) {
    for (entity, mut transform, mut projection, mut grid_cell, mut scale) in query.iter_mut() {
        let Some(reference_frame) = frames.parent_frame(entity) else {
            continue;
        };
        if keyboard_input.pressed(KeyCode::ArrowLeft) {
            transform.translation.x -= projection.scale * time.delta_secs() * 200.0;
        }
        if keyboard_input.pressed(KeyCode::ArrowRight) {
            // Example from https://github.com/aevyrie/big_space/blob/main/src/camera.rs
            // Calculates a high precision translation using a f64 movement, and then
            // converts it into a grid cell and low precision translation.
            //
            // let translation_next = DVec3 {
            //     x: 2.0 * scale.0,
            //     y: 0.0,
            //     z: 0.0,
            // };
            // let (cell_offset, new_translation) =
            //     reference_frame.translation_to_grid(translation_next);
            // info!(
            //     "Grid cell: {:?}, cell_offset: {:?}, next: {}, new_translation: {}",
            //     grid_cell, cell_offset, translation_next, new_translation
            // );
            // *grid_cell += cell_offset;
            // transform.translation += new_translation;
            // info!("transform: {:?}", transform);
            transform.translation.x += projection.scale * time.delta_secs() * 200.0;
        }
        if keyboard_input.pressed(KeyCode::ArrowDown) {
            transform.translation.y -= projection.scale * time.delta_secs() * 200.0;
        }
        if keyboard_input.pressed(KeyCode::ArrowUp) {
            transform.translation.y += projection.scale * time.delta_secs() * 200.0;
        }

        let scale_factor: f64 = 5.0;
        if keyboard_input.pressed(KeyCode::Equal) {
            projection.scale *= (1.0 - scale_factor * time.delta_secs_f64()) as f32;
            scale.0 *= 1.0 - scale_factor * time.delta_secs_f64();
        }
        if keyboard_input.pressed(KeyCode::Minus) {
            projection.scale *= (1.0 + scale_factor * time.delta_secs_f64()) as f32;
            scale.0 *= 1.0 + scale_factor * time.delta_secs_f64();
        }
    }
}

/// Scale entities up if they end up becoming smaller than one pixel in the current projection scale.
pub fn scale_entities(
    mut query: Query<(&mut Transform, &Mesh2d), With<Autoscale>>,
    projections: Query<&OrthographicProjection, With<InGameCamera>>,
    meshes: ResMut<Assets<Mesh>>,
) {
    let projection = projections.single();
    for (mut transform, mesh) in query.iter_mut() {
        // TODO: This needs some fixing.
        let Some(m) = meshes.get(&mesh.0) else {
            todo!()
        };
        let Some(aabb) = m.compute_aabb() else {
            todo!()
        };

        let size = f32::min(aabb.half_extents.x, aabb.half_extents.y);
        if size / projection.scale < 1.0 {
            transform.scale = Vec3::splat(projection.scale / size);
        } else {
            transform.scale = Vec3::ONE;
        }
    }
}

#[derive(Component)]
pub struct Autofollow {
    pub target: Option<Entity>,
}

pub fn change_focus(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut autofollow_query: Query<&mut Autofollow, With<InGameCamera>>,
    focus_targets_query: Query<(Entity, &Name), With<Focusable>>,
) {
    if keyboard_input.just_pressed(KeyCode::Escape) {
        autofollow_query.single_mut().target = Option::None;
    }
    if keyboard_input.just_pressed(KeyCode::Tab) {
        info!("Focus change");
        let mut autofollow = autofollow_query.single_mut();
        let mut found = false;
        for (target, name) in focus_targets_query.iter() {
            info!("checking {}", name);
            if autofollow.target.is_some() {
                if found {
                    autofollow.target = Some(target);
                    info!("focusing {}", name);
                    break;
                }
                if autofollow.target.unwrap() == target {
                    found = true;
                    continue;
                }
            } else {
                autofollow.target = Some(target);
                info!("focusing {}", name);
                break;
            }
        }
    }
}
