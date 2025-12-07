use bevy::input::InputSystems;
use bevy::prelude::*;
use bevy_egui::input::EguiWantsInput;
use leafwing_input_manager::plugin::InputManagerSystem;
use leafwing_input_manager::prelude::updating::EnabledInput;
use leafwing_input_manager::prelude::{ClashStrategy, MouseMove, MouseScroll};

fn configure_gamepads(mut gamepads: Query<&mut GamepadSettings, Added<GamepadSettings>>) {
    for mut settings in &mut gamepads {
        // TODO: Maybe set this specifically for LeftTrigger2|RightTrigger2 .
        settings.default_button_settings.set_release_threshold(0.01);
        settings.default_button_settings.set_press_threshold(0.05);
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

pub struct InputPlugin;

impl Plugin for InputPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(ClashStrategy::PrioritizeLongest);
        app.add_systems(
            PreUpdate,
            disable_leafwing_input_when_egui_wants_input
                .after(InputSystems)
                .before(InputManagerSystem::Unify),
        );
        app.add_systems(Update, configure_gamepads);
    }
}
