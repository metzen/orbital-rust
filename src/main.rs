use bevy::{
    audio::{AddAudioSource, AudioPlugin, SpatialScale},
    prelude::*,
};
use bevy::{transform::TransformSystem, window::WindowResolution};

mod audio;
mod camera;
mod diagnostics;
mod hud;
mod lifetime;
mod physics;
mod scene;
mod timewarp;
mod trails;
mod vessel;

use audio::SineAudio;
use bevy_framepace::FramepacePlugin;
use big_space::BigSpacePlugin;
use camera::{
    camera_control, change_focus, scale_entities, setup_camera,
    update_camera_position_for_autofollow,
};
use diagnostics::DiagnosticsPlugin;
use hud::HudPlugin;
use lifetime::reaper;
use physics::PhysicsPlugin;
use scene::setup_scene;
use timewarp::TimeWarpPlugin;
use trails::TrailsPlugin;
use vessel::VesselPlugin;

// // The initial scene file will be loaded below and not change when the scene is saved
// const SCENE_FILE_PATH: &str = "scenes/solar_system.scn.ron";

// fn load_scene_system(mut commands: Commands, asset_server: Res<AssetServer>) {
//     // "Spawning" a scene bundle creates a new entity and spawns new instances
//     // of the given scene's entities as children of that entity.
//     commands.spawn(DynamicSceneBundle {
//         // Scenes are loaded just like any other asset.
//         scene: asset_server.load(SCENE_FILE_PATH),
//         ..default()
//     });
// }

/// Spatial audio uses the distance to attenuate the sound volume. In 2D with the default camera,
/// 1 pixel is 1 unit of distance, so we use a scale so that 100 pixels is 1 unit of distance for
/// audio.
const AUDIO_SCALE: f32 = 1.0 / 100.0;

fn main() {
    App::new()
        .insert_resource(ClearColor(Color::BLACK))
        .add_plugins((
            DefaultPlugins
                .build()
                // BigSpacePlugin will handle syncing Transforms to GlobalTransform.
                .disable::<TransformPlugin>()
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: String::from("Orbital"),
                        present_mode: bevy::window::PresentMode::AutoNoVsync,
                        resolution: WindowResolution::new(1024.0, 768.0),
                        ..default()
                    }),
                    ..default()
                })
                .set(ImagePlugin::default_nearest())
                .set(AudioPlugin {
                    global_volume: GlobalVolume::new(1.0),
                    default_spatial_scale: SpatialScale::new_2d(AUDIO_SCALE),
                }),
            BigSpacePlugin::<i32>::default(),
            FramepacePlugin,
            DiagnosticsPlugin,
            TimeWarpPlugin,
            PhysicsPlugin,
            VesselPlugin,
            TrailsPlugin,
            HudPlugin,
            // LogDiagnosticsPlugin::default(),
        ))
        .add_audio_source::<SineAudio>()
        .insert_resource(Time::<Fixed>::from_hz(64.0))
        .add_systems(PreStartup, setup_scene)
        .add_systems(PostStartup, setup_camera)
        .add_systems(
            Update,
            (
                //fit_canvas,
                app_control,
                change_focus,
            ),
        )
        .add_systems(
            PostUpdate,
            (
                update_camera_position_for_autofollow.before(TransformSystem::TransformPropagate),
                camera_control.before(TransformSystem::TransformPropagate),
                scale_entities.before(TransformSystem::TransformPropagate),
                reaper,
            ),
        )
        .run();
}

fn app_control(keyboard_input: Res<ButtonInput<KeyCode>>, mut exit: EventWriter<AppExit>) {
    if keyboard_input.pressed(KeyCode::KeyQ) {
        exit.send(AppExit::Success);
    }
}
