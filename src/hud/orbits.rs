use bevy::camera::visibility::{Layer, RenderLayers};
use bevy::color::palettes::css::LIGHT_GRAY;
use bevy::ecs::query::QueryData;
use bevy::ecs::system::lifetimeless::Read;
use bevy::gizmos::gizmos::GizmoBuffer;
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

// Allow for Orbit to be used as a primitive for Gizmo rendering.
impl Primitive2d for Orbit {}

/// Builder for configuring the drawing options of [`Orbit`].
pub struct Orbit2dBuilder<'a, Config, Clear>
where
    Config: GizmoConfigGroup,
    Clear: 'static + Send + Sync,
{
    gizmos: &'a mut GizmoBuffer<Config, Clear>,
    isometry: Isometry2d,
    shape: OrbitShape,
    half_size: Vec2,
    start_angle: f32,
    color: Color,
    resolution: u32,
    fade: bool,
    apsides: bool,
    apsis_radius: f32,
}

impl<Config, Clear> Orbit2dBuilder<'_, Config, Clear>
where
    Config: GizmoConfigGroup,
    Clear: 'static + Send + Sync,
{
    /// Set the number of line-segments for the orbit.
    pub fn resolution(mut self, resolution: u32) -> Self {
        self.resolution = resolution;
        self
    }

    /// Set the fade behavior of the orbit.
    pub fn fade(mut self, fade: bool) -> Self {
        self.fade = fade;
        self
    }

    /// Set whether to draw the apsides of the orbit.
    pub fn apsides(mut self, apsides: bool) -> Self {
        self.apsides = apsides;
        self
    }

    /// Set the radius of the apsis circles.
    pub fn apsis_radius(mut self, apsis_radius: f32) -> Self {
        self.apsis_radius = apsis_radius;
        self
    }

    fn draw_orbit(&mut self) {
        match self.shape {
            OrbitShape::Circle | OrbitShape::Ellipse => self.draw_elliptical_orbit(),
            OrbitShape::Parabola | OrbitShape::Hyperbola => self.draw_hyperbolic_orbit(),
        }
    }

    fn draw_elliptical_orbit(&mut self) {
        use bevy_gizmos_ext::GizmoBufferExt;
        if self.fade {
            self.gizmos
                .ellipse_gradient_2d(
                    self.isometry,
                    self.half_size,
                    self.start_angle,
                    self.color.with_alpha(0.01),
                    self.color.with_alpha(0.3),
                )
                .resolution(self.resolution);
        } else {
            self.gizmos
                .ellipse_2d(self.isometry, self.half_size, self.color.with_alpha(0.3))
                .resolution(self.resolution);
        }
    }

    fn draw_hyperbolic_orbit(&mut self) {
        use bevy_gizmos_ext::GizmoBufferExt;
        self.gizmos
            .hyperbola_2d(self.isometry, self.half_size, self.color.with_alpha(0.3))
            .resolution(self.resolution);
    }

    fn draw_apsides(&mut self) {
        self.gizmos.circle_2d(
            self.isometry * vec2(self.half_size.x, 0.0),
            self.apsis_radius,
            self.color,
        );
        self.gizmos.circle_2d(
            self.isometry * vec2(-self.half_size.x, 0.0),
            self.apsis_radius,
            self.color,
        );
    }
}

impl<Config, Clear> Drop for Orbit2dBuilder<'_, Config, Clear>
where
    Config: GizmoConfigGroup,
    Clear: 'static + Send + Sync,
{
    fn drop(&mut self) {
        self.draw_orbit();
        if self.apsides {
            self.draw_apsides();
        }
    }
}

impl<Config, Clear> GizmoPrimitive2d<Orbit> for GizmoBuffer<Config, Clear>
where
    Config: GizmoConfigGroup,
    Clear: 'static + Send + Sync,
{
    type Output<'a>
        = Orbit2dBuilder<'a, Config, Clear>
    where
        Self: 'a;

    fn primitive_2d(
        &mut self,
        orbit: &Orbit,
        isometry: impl Into<Isometry2d>,
        color: impl Into<Color>,
    ) -> Self::Output<'_> {
        const DEFAULT_ORBIT_RESOLUTION: u32 = 32;
        Orbit2dBuilder {
            gizmos: self,
            isometry: isometry.into(),
            shape: orbit.shape(),
            half_size: DVec2::new(orbit.semi_major_axis, orbit.semi_minor_axis).as_vec2(),
            start_angle: orbit.eccentric_anomaly().as_radians_f64() as f32,
            color: color.into(),
            resolution: DEFAULT_ORBIT_RESOLUTION,
            fade: false,
            apsides: false,
            apsis_radius: 1.0,
        }
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
        gizmos
            .primitive_2d(
                &orbit,
                Isometry2d::new(
                    primary.global_transform.translation().xy() + orbit.center().as_vec2(),
                    Rot2::radians(DVec2::X.angle_to(-orbit.eccentricity_vector) as f32),
                ),
                match materials.get(secondary.mesh_material) {
                    Some(material) => material.color,
                    None => Color::from(LIGHT_GRAY),
                },
            )
            .resolution(2000)
            .fade(secondary.vessel.is_none())
            .apsis_radius(apsis_radius)
            .apsides(secondary.vessel.is_some_and(|vessel| vessel.controlled));
    }
}
