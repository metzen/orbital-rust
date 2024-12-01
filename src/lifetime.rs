use bevy::prelude::*;

#[derive(Component)]
pub struct Ephemeral {
    pub ttl: i32,
}

/// Despawns ephemeral entities which have reached the end of their time-to-live.
pub fn reaper(
    mut commands: Commands,
    mut query: Query<(
        Entity,
        &mut Transform,
        &MeshMaterial2d<ColorMaterial>,
        &mut Ephemeral,
    )>,
    mut color_materials: ResMut<Assets<ColorMaterial>>,
) {
    for (entity, mut transform, color_material_handle, mut ephemeral) in query.iter_mut() {
        ephemeral.ttl -= 1;
        // TODO: Move scale and transperency to an animation system.
        transform.scale *= 1.01;
        let material = color_materials.get_mut(color_material_handle).unwrap();
        material.color.set_alpha(material.color.alpha() * 0.98);
        if ephemeral.ttl <= 0 {
            commands.entity(entity).remove_parent().despawn();
        }
    }
}
