use bevy::camera::visibility::{Layer, RenderLayers};
use bevy::color::palettes::css::LIGHT_GRAY;
use bevy::ecs::query::QueryData;
use bevy::ecs::system::lifetimeless::Read;
use bevy::input::common_conditions::input_toggle_active;
use bevy::math::DVec2;
use bevy::prelude::*;
use big_space::floating_origins::BigSpace;
use big_space::grid::Grid;
use big_space::grid::cell::CellCoord;

use crate::camera::InGameCamera;
use crate::physics::{NoGravity, Orbit, OrbitShape, RigidBody, SatelliteOf};
use crate::vessel::Vessel;

pub(super) struct OrbitGizmoPlugin;

impl Plugin for OrbitGizmoPlugin {
    fn build(&self, app: &mut App) {
        use crate::rendering::LayerExt;
        use bevy_gizmos_ext::GizmoConfigExt;
        use bevy_gizmos_ext::GizmoLineConfigExt;
        app.insert_gizmo_config(
            OrbitGizmoConfigGroup,
            GizmoConfig::default()
                .with_render_layers(RenderLayers::layer(Layer::ORBIT))
                .with_line(GizmoLineConfig::default().with_width(1.0)),
        );
        app.add_systems(
            PostUpdate,
            draw_orbits
                .after(TransformSystems::Propagate)
                .run_if(input_toggle_active(true, KeyCode::F8)),
        );
    }
}

#[derive(GizmoConfigGroup, Default, Reflect)]
struct OrbitGizmoConfigGroup;

trait RenderOrbit {
    fn render(
        &self,
        gizmos: &mut Gizmos<OrbitGizmoConfigGroup>,
        translation: &Vec2,
        color: Color,
        fade: bool,
    );
    fn render_ellipse(
        &self,
        gizmos: &mut Gizmos<OrbitGizmoConfigGroup>,
        translation: &Vec2,
        color: Color,
        fade: bool,
    );
    fn render_hyperbola(
        &self,
        gizmos: &mut Gizmos<OrbitGizmoConfigGroup>,
        translation: &Vec2,
        color: Color,
        fade: bool,
    );
}

impl RenderOrbit for Orbit {
    fn render(
        &self,
        gizmos: &mut Gizmos<OrbitGizmoConfigGroup>,
        translation: &Vec2,
        color: Color,
        fade: bool,
    ) {
        match self.shape() {
            OrbitShape::Circle => self.render_ellipse(gizmos, translation, color, fade),
            OrbitShape::Ellipse => self.render_ellipse(gizmos, translation, color, fade),
            OrbitShape::Parabola => self.render_hyperbola(gizmos, translation, color, fade),
            OrbitShape::Hyperbola => self.render_hyperbola(gizmos, translation, color, fade),
        }
    }

    fn render_ellipse(
        &self,
        gizmos: &mut Gizmos<OrbitGizmoConfigGroup>,
        translation: &Vec2,
        color: Color,
        fade: bool,
    ) {
        let angle = DVec2::X.angle_to(-self.eccentricity_vector) as f32;
        if fade {
            use bevy_gizmos_ext::GizmosExt;
            gizmos
                .ellipse_gradient_2d(
                    Isometry2d::new(translation + self.center().as_vec2(), Rot2::radians(angle)),
                    DVec2::new(self.semi_major_axis, self.semi_minor_axis).as_vec2(),
                    self.eccentric_anomaly().as_radians_f64() as f32,
                    color.with_alpha(0.01),
                    color.with_alpha(0.3),
                )
                .resolution(2000);
        } else {
            gizmos
                .ellipse_2d(
                    Isometry2d::new(translation + self.center().as_vec2(), Rot2::radians(angle)),
                    DVec2::new(self.semi_major_axis, self.semi_minor_axis).as_vec2(),
                    color.with_alpha(0.3),
                )
                .resolution(2000);
        }
    }

    fn render_hyperbola(
        &self,
        gizmos: &mut Gizmos<OrbitGizmoConfigGroup>,
        translation: &Vec2,
        color: Color,
        _fade: bool,
    ) {
        use bevy_gizmos_ext::GizmosExt;
        let angle = DVec2::X.angle_to(-self.eccentricity_vector) as f32;
        gizmos
            .hyperbola_2d(
                Isometry2d::new(translation - self.center().as_vec2(), Rot2::radians(angle)),
                DVec2::new(self.semi_major_axis, self.semi_minor_axis).as_vec2(),
                color.with_alpha(0.2),
            )
            .resolution(2000);
    }
}

#[derive(QueryData)]
struct OrbitSatelliteQueryData {
    cell: Read<CellCoord>,
    transform: Read<Transform>,
    global_transform: Read<GlobalTransform>,
    rigidbody: Read<RigidBody>,
    satellite_of: Read<SatelliteOf>,
    mesh_material: Read<MeshMaterial2d<ColorMaterial>>,
    vessel: Option<Read<Vessel>>,
}

#[derive(QueryData)]
struct OrbitPrimaryQueryData {
    cell: Read<CellCoord>,
    transform: Read<Transform>,
    global_transform: Read<GlobalTransform>,
    rigidbody: Read<RigidBody>,
}

fn draw_orbits(
    satellite_query: Query<OrbitSatelliteQueryData, Without<NoGravity>>,
    primary_query: Query<OrbitPrimaryQueryData>,
    grid: Single<&Grid, With<BigSpace>>,
    projection: Single<&Projection, With<InGameCamera>>,
    mut gizmos: Gizmos<OrbitGizmoConfigGroup>,
    materials: Res<Assets<ColorMaterial>>,
) {
    let apsis_radius = if let Projection::Orthographic(orthographic) = projection.into_inner() {
        orthographic.scale
    } else {
        1.0
    };
    for secondary in &satellite_query {
        let primary = primary_query.get(secondary.satellite_of.primary()).unwrap();
        let primary_position = grid.grid_position_double(primary.cell, primary.transform);
        let secondary_position = grid.grid_position_double(secondary.cell, secondary.transform);
        let orbit = Orbit::new(
            (secondary_position - primary_position).xy(),
            (secondary.rigidbody.velocity - primary.rigidbody.velocity)
                .xy()
                .as_dvec2(),
            primary.rigidbody.mass,
            secondary.rigidbody.mass,
        );
        let translation = primary.global_transform.translation().xy();
        let color = match materials.get(secondary.mesh_material) {
            Some(material) => material.color,
            None => Color::from(LIGHT_GRAY),
        };
        orbit.render(&mut gizmos, &translation, color, secondary.vessel.is_none());
        // AP and PE markers for controlled vessel.
        let perifocal_unit_vec = orbit.eccentricity_vector / orbit.eccentricity;
        let ap_vec = orbit.apoapsis * -perifocal_unit_vec;
        let pe_vec = orbit.periapsis * perifocal_unit_vec;
        if let Some(vessel) = secondary.vessel
            && vessel.controlled
        {
            gizmos.circle_2d(
                Isometry2d::from_translation(translation + ap_vec.as_vec2()),
                apsis_radius,
                color.with_alpha(1.0),
            );
            gizmos.circle_2d(
                Isometry2d::from_translation(translation + pe_vec.as_vec2()),
                apsis_radius,
                color.with_alpha(1.0),
            );
        }
    }
}
