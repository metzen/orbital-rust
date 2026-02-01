#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod audio;
mod camera;
mod diagnostics;
mod gizmos;
mod hud;
mod input;
mod lifetime;
mod math;
mod physics;
mod scene;
mod timewarp;
mod trails;
mod util;
mod vessel;

use bevy::audio::{AddAudioSource, AudioPlugin, SpatialScale, Volume};
use bevy::ecs::system::NonSendMarker;
use bevy::input::common_conditions::{input_just_pressed, input_pressed, input_toggle_active};
use bevy::prelude::*;
use bevy::window::{PresentMode, PrimaryWindow, WindowMode, WindowResolution};
use bevy::winit::WINIT_WINDOWS;
use bevy_framepace::{FramepaceSettings, Limiter};
use clap::Parser;
use diagnostics::DiagnosticsPlugin;
use winit::window::Icon;

use crate::audio::SineAudio;
use crate::camera::CameraPlugin;
use crate::hud::HudPlugin;
use crate::input::InputPlugin;
use crate::lifetime::LifetimePlugin;
use crate::physics::PhysicsPlugin;
use crate::scene::{add_name_to_big_space, setup_scene, spawn_atmosphere_layers};
use crate::timewarp::TimeWarpPlugin;
use crate::trails::TrailsPlugin;
use crate::vessel::VesselPlugin;

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

#[derive(Parser)]
struct Args {
    #[arg(long, default_value_t = 64.0)]
    fixed_update_frequency: f64,

    #[arg(long)]
    framerate_limit: Option<f64>,
}

#[cfg(any(windows, unix))]
fn set_window_icon(_non_send_marker: NonSendMarker) {
    WINIT_WINDOWS.with_borrow(|winit| {
        let image = image::open("icon.ico")
            .expect("Failed to open icon path")
            .into_rgba8();
        let (width, height) = image.dimensions();
        let icon = Icon::from_rgba(image.into_raw(), width, height).unwrap();
        for window in winit.windows.values() {
            window.set_window_icon(Some(icon.clone()));
        }
    })
}

fn toggle_fullscreen(mut window: Single<&mut Window, With<PrimaryWindow>>) {
    window.mode = match window.mode {
        WindowMode::BorderlessFullscreen(MonitorSelection::Primary) => WindowMode::Windowed,
        WindowMode::Windowed => WindowMode::BorderlessFullscreen(MonitorSelection::Primary),
        _ => panic!("Unexpected window mode"),
    }
}

fn exit(mut writer: MessageWriter<AppExit>) {
    writer.write(AppExit::Success);
}

fn main() {
    let args = Args::parse();
    App::new()
        .add_plugins((
            DefaultPlugins
                .build()
                // BigSpacePropagationPlugin will handle syncing Transforms to GlobalTransform.
                .disable::<TransformPlugin>()
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: String::from("Orbital"),
                        present_mode: PresentMode::AutoNoVsync,
                        resolution: WindowResolution::new(1280, 800)
                            .with_scale_factor_override(1.0),
                        fit_canvas_to_parent: true,
                        ..default()
                    }),
                    ..default()
                })
                .set(ImagePlugin::default_nearest())
                .set(AudioPlugin {
                    global_volume: GlobalVolume::new(Volume::Linear(1.0)),
                    default_spatial_scale: SpatialScale::new_2d(AUDIO_SCALE),
                }),
            MeshPickingPlugin,
            // LogDiagnosticsPlugin::default(),
        ))
        // Third-party plugins.
        .add_plugins((
            bevy_egui::EguiPlugin::default(),
            bevy_framepace::FramepacePlugin,
            bevy_framepace::debug::DiagnosticsPlugin,
            bevy_inspector_egui::quick::WorldInspectorPlugin::new()
                .run_if(input_toggle_active(false, KeyCode::F12)),
            big_space::plugin::BigSpaceDefaultPlugins,
        ))
        // Crate plugins.
        .add_plugins((
            CameraPlugin,
            DiagnosticsPlugin,
            HudPlugin,
            InputPlugin,
            LifetimePlugin,
            PhysicsPlugin,
            TimeWarpPlugin,
            TrailsPlugin,
            VesselPlugin,
        ))
        .insert_resource(ClearColor(Color::BLACK))
        .insert_resource(FramepaceSettings {
            limiter: match args.framerate_limit {
                None => Limiter::Auto,
                Some(framerate) => Limiter::from_framerate(framerate),
            },
        })
        .insert_resource(Time::<Fixed>::from_hz(args.fixed_update_frequency))
        .add_audio_source::<SineAudio>()
        .add_systems(
            PreStartup,
            (
                #[cfg(any(unix, windows))]
                set_window_icon,
                setup_scene,
                add_name_to_big_space,
                spawn_atmosphere_layers,
            )
                .chain(),
        )
        .add_systems(
            Update,
            exit.run_if(input_just_pressed(KeyCode::KeyQ))
                .run_if(input_pressed(KeyCode::ControlLeft)),
        )
        .add_systems(
            Update,
            toggle_fullscreen.run_if(input_just_pressed(KeyCode::F11)),
        )
        .run();
}
