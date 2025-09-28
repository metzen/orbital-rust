use bevy::{ecs::entity_disabling::Disabled, prelude::*};

/// What should happen when an ephemeral entity expires.
pub enum ExpirationAction {
    Despawn,
    Disable,
}

pub enum Clock {
    Real,
    Virtual,
}

/// An "ephemeral" entity.
#[derive(Component)]
pub struct Ephemeral {
    pub ttl: Timer,
    pub expiration_action: ExpirationAction,
    pub clock: Clock,
}

impl Ephemeral {
    pub fn new(ttl: Timer, expiration_action: ExpirationAction, clock: Clock) -> Self {
        Self {
            ttl,
            expiration_action,
            clock,
        }
    }
}

/// Despawns ephemeral entities which have reached the end of their time-to-live.
pub fn reaper(
    mut commands: Commands,
    mut query: Query<(Entity, &MeshMaterial2d<ColorMaterial>, &mut Ephemeral)>,
    mut color_materials: ResMut<Assets<ColorMaterial>>,
    real_time: Res<Time<Real>>,
    virtual_time: Res<Time<Virtual>>,
) {
    for (entity, color_material_handle, mut ephemeral) in query.iter_mut() {
        let delta = match ephemeral.clock {
            Clock::Real => real_time.delta(),
            Clock::Virtual => virtual_time.delta(),
        };
        ephemeral.ttl.tick(delta);
        // TODO: Move scale and transperency to an animation system.

        let material = color_materials.get_mut(color_material_handle).unwrap();
        material.color.set_alpha(ephemeral.ttl.fraction_remaining());
        // Psychedelic!
        // material.color = material.color.rotate_hue(10.0);
        if ephemeral.ttl.finished() {
            match ephemeral.expiration_action {
                ExpirationAction::Despawn => {
                    commands.entity(entity).despawn();
                }
                ExpirationAction::Disable => {
                    commands.entity(entity).remove::<ChildOf>();
                    commands.entity(entity).insert(Disabled);
                }
            };
        }
    }
}
