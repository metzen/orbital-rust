use bevy::picking::pointer::PointerInteraction;
use bevy::prelude::*;

use crate::camera::InGamePointer;

pub(super) struct HoveredPlugin;

impl Plugin for HoveredPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup);
        app.add_systems(FixedUpdate, update);
    }
}

#[derive(Component)]
pub struct HoverText;

fn setup(mut commands: Commands) {
    use crate::hud::TextFontExt;
    commands.spawn((
        Name::new("hover text"),
        Node {
            margin: UiRect {
                left: auto(),
                right: auto(),
                top: px(100.0),
                bottom: auto(),
            },
            ..default()
        },
        Text::default(),
        TextFont::ui_default(),
        HoverText,
    ));
}

fn update(
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
