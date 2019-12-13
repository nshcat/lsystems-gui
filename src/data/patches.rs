#[macro_use]
use serde_derive::*;
use nalgebra_glm::Vec3;

/// Data of a single bezier curve in a bezier patch
#[derive(Serialize, Deserialize)]
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
}

/// A structure containing all data and settings to construct a 3D bicubic bezier
/// patch surface.
#[derive(Serialize, Deserialize)]
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
}

/// A collection of multiple bezier patch definitions which make up a whole
/// model which can be used as part of an L-System.
#[derive(Serialize, Deserialize)]
pub struct BezierModelParameters {
    /// The name this model can be referenced by in the L-System
    pub identifier: char,
    /// The parameters of the patches this model is made out of
    pub patches: Vec<BezierPatchParameters>
}


