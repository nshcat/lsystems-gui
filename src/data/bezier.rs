#[macro_use]
use serde_derive::*;
use nalgebra_glm::{Mat4, Vec3};
extern crate nalgebra;

/// Data of a single bezier curve in a bezier patch
#[derive(Serialize, Deserialize, Clone)]
pub struct BezierCurveParameters {
    /// The four control points that describe this curve
    pub control_points: [Vec3; 4]
}

impl BezierCurveParameters {
    pub fn empty() -> BezierCurveParameters {
        BezierCurveParameters {
            control_points: [Vec3::zeros(), Vec3::zeros(), Vec3::zeros(), Vec3::zeros()]
        }
    }

    pub fn from_points(pts: [Vec3; 4]) -> BezierCurveParameters {
        BezierCurveParameters {
            control_points: pts
        }
    }

    pub fn evaluate(&self, u: f32) -> Vec3 {
        let b0 = (1.0 - u).powf(3.0);
        let b1 = 3.0 * u * ((1.0 - u).powf(2.0));
        let b2 = 3.0 * (u.powf(2.0)) * (1.0 - u);
        let b3 = u.powf(3.0);

        (self.control_points[0] * b0)
        + (self.control_points[1] * b1)
        + (self.control_points[2] * b2)
        + (self.control_points[3] * b3)
    }
}

/// A structure containing all data and settings to construct a 3D bicubic bezier
/// patch surface.
#[derive(Serialize, Deserialize, Clone)]
pub struct BezierPatchParameters {
    /// The four bezier curves that make up the patch
    pub curves: [BezierCurveParameters; 4]
}

impl BezierPatchParameters {
    pub fn empty() -> BezierPatchParameters {
        BezierPatchParameters {
            curves: [
                BezierCurveParameters::empty(),
                BezierCurveParameters::empty(),
                BezierCurveParameters::empty(),
                BezierCurveParameters::empty()
            ]
        }
    }

    /// Evalute the patch at given UV position in [0,1]x[0,1].
    pub fn evaluate(&self, u: f32, v: f32) -> Vec3 {
        let curve0 = &self.curves[0];
        let curve1 = &self.curves[1];
        let curve2 = &self.curves[2];
        let curve3 = &self.curves[3];

        let pt0 = curve0.evaluate(u);
        let pt1 = curve1.evaluate(u);
        let pt2 = curve2.evaluate(u);
        let pt3 = curve3.evaluate(u);

        let temp_curve = BezierCurveParameters::from_points([pt0, pt1, pt2, pt3]);
        
        temp_curve.evaluate(v)
    }
}

/// A collection of multiple bezier patch definitions which make up a whole
/// model which can be used as part of an L-System.
#[derive(Serialize, Deserialize, Clone)]
pub struct BezierModelParameters {
    /// The name this model can be referenced by in the L-System
    pub symbol: Option<char>,
    /// The parameters of the patches this model is made out of
    pub patches: Vec<BezierPatchParameters>
}

impl BezierModelParameters {
    pub fn empty() -> BezierModelParameters {
        BezierModelParameters {
            symbol: None,
            patches: Vec::new()
        }
    }
}

