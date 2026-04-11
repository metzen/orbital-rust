use crate::camera::HIGH_RES_LAYER;
use bevy::camera::visibility::RenderLayers;
use bevy::prelude::*;

use crate::hud::HudSubject;
use crate::physics::SatelliteOf;

pub(super) struct SubjectPlugin;

impl Plugin for SubjectPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup);
        app.add_systems(FixedUpdate, update);
    }
}

#[derive(Component)]
pub struct HubSubjectText;

fn setup(mut commands: Commands) {
    use crate::hud::TextFontExt;
    commands.spawn((
        Name::new("Subject text"),
        Node {
            top: px(20.0),
            left: px(20.0),
            padding: px(10.0).into(),
            ..default()
        },
        RenderLayers::layer(HIGH_RES_LAYER),
        BackgroundColor::from(Srgba::new(0.05, 0.11, 0.15, 1.0)),
        children![(
            Text::default(),
            TextLayout::new_with_justify(Justify::Center),
            HubSubjectText,
            TextFont::ui_default(),
        ),],
    ));
}

fn update(
    subject_vessel_query: Query<Entity, With<HudSubject>>,
    mut hud_subject_text: Single<&mut Text, With<HubSubjectText>>,
    name_query: Query<(&Name, Option<&SatelliteOf>)>,
) {
    if let Ok(entity) = subject_vessel_query.single() {
        let mut parent = Some(entity);
        let mut parts = Vec::new();
        while let Some(entity) = parent
            && let Ok((name, satellite_of)) = name_query.get(entity)
        {
            parts.push(format!("{}", name));
            parent = satellite_of.map(|x| x.primary());
        }
        parts.reverse();
        hud_subject_text.0 = parts.join(" / ");
    } else {
        hud_subject_text.0 = String::from("none");
    }
}
