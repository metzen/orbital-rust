use std::f64::consts::FRAC_PI_2;
use std::f64::consts::PI;

use super::*;

#[test]
fn test_angle_from_radians() {
    let result = Angle::from_radians(1.0);
    assert_eq!(result, Angle::Radians(1.0));
}

#[test]
fn test_angle_from_degrees() {
    let result = Angle::from_degrees(90.0);
    assert_eq!(result, Angle::Degrees(90.0));
}

#[test]
fn test_angle_from_degrees_as_radians_f64() {
    let result = Angle::from_degrees(90.0).as_radians_f64();
    assert_eq!(result, FRAC_PI_2);
}

#[test]
fn test_angle_from_radians_as_degrees_f64() {
    let result = Angle::from_radians(1.0).as_degrees_f64();
    assert_eq!(result, 180.0 / PI);
}
