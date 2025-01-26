use bevy::prelude::*;

#[derive(Component)]
pub struct Ephemeral {
    pub ttl: Timer,
}

/// Despawns ephemeral entities which have reached the end of their time-to-live.
pub fn reaper(
    mut commands: Commands,
    mut query: Query<(Entity, &MeshMaterial2d<ColorMaterial>, &mut Ephemeral)>,
    mut color_materials: ResMut<Assets<ColorMaterial>>,
    time: Res<Time>,
) {
    for (entity, color_material_handle, mut ephemeral) in query.iter_mut() {
        ephemeral.ttl.tick(time.delta());
        // TODO: Move scale and transperency to an animation system.

        let material = color_materials.get_mut(color_material_handle).unwrap();
        material.color.set_alpha(material.color.alpha() * 0.99);
        // Psychedelic!
        // material.color = material.color.rotate_hue(10.0);
        if ephemeral.ttl.finished() {
            commands.entity(entity).remove_parent().despawn();
        }
    }
}
