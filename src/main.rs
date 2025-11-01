#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use bevy::{
    audio::{AddAudioSource, AudioPlugin, SpatialScale},
    input::InputSystems,
    prelude::*,
    window::WindowResolution,
};

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
use bevy::winit::WinitWindows;
use bevy_egui::{EguiPlugin, input::EguiWantsInput};
use bevy_framepace::{FramepacePlugin, FramepaceSettings, Limiter};
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use big_space::plugin::BigSpaceDefaultPlugins;
use camera::CameraPlugin;
use clap::Parser;
use diagnostics::DiagnosticsPlugin;
use hud::HudPlugin;
use leafwing_input_manager::{
    plugin::InputManagerSystem,
    user_input::{MouseMove, MouseScroll, updating::EnabledInput},
};
use physics::PhysicsPlugin;
use scene::setup_scene;
use timewarp::TimeWarpPlugin;
use trails::TrailsPlugin;
use vessel::VesselPlugin;
use winit::window::Icon;

use crate::{lifetime::LifetimePlugin, scene::add_name_to_big_space};

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

fn app_control(keyboard_input: Res<ButtonInput<KeyCode>>, mut exit: MessageWriter<AppExit>) {
    if keyboard_input.pressed(KeyCode::KeyQ) {
        exit.write(AppExit::Success);
    }
}

fn disable_leafwing_input_when_egui_wants_input(
    egui_wants_input: Res<EguiWantsInput>,
    mut key_code: ResMut<EnabledInput<KeyCode>>,
    mut mouse_button: ResMut<EnabledInput<MouseButton>>,
    mut mouse_move: ResMut<EnabledInput<MouseMove>>,
    mut mouse_scroll: ResMut<EnabledInput<MouseScroll>>,
) {
    key_code.is_enabled = !egui_wants_input.wants_any_keyboard_input();
    mouse_button.is_enabled = !egui_wants_input.wants_any_pointer_input();
    mouse_move.is_enabled = !egui_wants_input.wants_any_pointer_input();
    mouse_scroll.is_enabled = !egui_wants_input.wants_any_pointer_input();
}

fn set_window_icon(windows: Option<NonSend<WinitWindows>>) {
    let image = image::open("icon.ico")
        .expect("Failed to open icon path")
        .into_rgba8();
    let (width, height) = image.dimensions();
    let icon = Icon::from_rgba(image.into_raw(), width, height).unwrap();
    let Some(winit) = windows else { return };
    for window in winit.windows.values() {
        window.set_window_icon(Some(icon.clone()));
    }
}

fn main() {
    let args = Args::parse();
    App::new()
        .insert_resource(ClearColor(Color::BLACK))
        .add_plugins((
            DefaultPlugins
                .build()
                // BigSpacePropagationPlugin will handle syncing Transforms to GlobalTransform.
                .disable::<TransformPlugin>()
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: String::from("Orbital"),
                        present_mode: bevy::window::PresentMode::AutoNoVsync,
                        resolution: WindowResolution::new(1152, 720)
                            .with_scale_factor_override(1.0),
                        ..default()
                    }),
                    ..default()
                })
                .set(ImagePlugin::default_nearest())
                .set(AudioPlugin {
                    global_volume: GlobalVolume::new(bevy::audio::Volume::Linear(1.0)),
                    default_spatial_scale: SpatialScale::new_2d(AUDIO_SCALE),
                }),
            BigSpaceDefaultPlugins,
            CameraPlugin,
            FramepacePlugin,
            DiagnosticsPlugin,
            MeshPickingPlugin,
            LifetimePlugin,
            TimeWarpPlugin,
            PhysicsPlugin,
            VesselPlugin,
            TrailsPlugin,
            HudPlugin,
            EguiPlugin::default(),
            WorldInspectorPlugin::new(),
            // LogDiagnosticsPlugin::default(),
        ))
        .insert_resource(FramepaceSettings {
            limiter: match args.framerate_limit {
                None => Limiter::Auto,
                Some(framerate) => Limiter::from_framerate(framerate),
            },
        })
        .add_audio_source::<SineAudio>()
        .insert_resource(Time::<Fixed>::from_hz(args.fixed_update_frequency))
        .add_systems(
            PreStartup,
            (set_window_icon, setup_scene, add_name_to_big_space).chain(),
        )
        .add_systems(
            PreUpdate,
            disable_leafwing_input_when_egui_wants_input
                .after(InputSystems)
                .before(InputManagerSystem::Unify),
        )
        .add_systems(Update, app_control)
        .run();
}
