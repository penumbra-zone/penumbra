#![allow(non_snake_case)]
/// The acceptable amount of difference between a value and its approximation.
const APPROXIMATION_TOLERANCE: f64 = 1e-8;

pub mod xyk;
pub mod balancer {}
pub mod volatility {}

pub mod math_utils;

pub(crate) mod debug;
