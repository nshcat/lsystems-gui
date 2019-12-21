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
    pub curves: [BezierCurveParameters; 4],
    /// Color of this patch
    pub color: Vec3
}

impl BezierPatchParameters {
    pub fn empty() -> BezierPatchParameters {
        BezierPatchParameters {
            curves: [
                BezierCurveParameters::empty(),
                BezierCurveParameters::empty(),
                BezierCurveParameters::empty(),
                BezierCurveParameters::empty()
            ],
            color: Vec3::new(0.7, 0.7, 0.7)
        }
    }

    pub fn default() -> BezierPatchParameters {
        BezierPatchParameters {
            color: Vec3::new(0.7, 0.7, 0.7),
            curves: [
                BezierCurveParameters {
                    control_points: [ Vec3::new(0.0, 0.0, 0.0), Vec3::new(0.25, 0.0, 0.0), Vec3::new(0.75, 0.0, 0.0),  Vec3::new(1.0, 0.0, 0.0) ]
                },
                BezierCurveParameters {
                    control_points: [ Vec3::new(0.0, 0.25, 0.0), Vec3::new(0.25, 0.25, 0.0), Vec3::new(0.75, 0.25, 0.0),  Vec3::new(1.0, 0.25, 0.0) ]
                },
                BezierCurveParameters {
                    control_points: [ Vec3::new(0.0, 0.75, 0.0), Vec3::new(0.25, 0.75, 0.0), Vec3::new(0.75, 0.75, 0.0),  Vec3::new(1.0, 0.75, 0.0) ]
                },
                BezierCurveParameters {
                    control_points: [ Vec3::new(0.0, 1.0, 0.0), Vec3::new(0.25, 1.0, 0.0), Vec3::new(0.75, 1.0, 0.0),  Vec3::new(1.0, 1.0, 0.0) ]
                }
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

    /// Clone this bezier patch, and apply mirroring on given mirror plane to the clone.
    pub fn clone_mirrored(&self, mirror_plane: MirrorPlane) -> BezierPatchParameters {
        let mut cloned = self.clone();

        let factors = mirror_plane.factors();
        
        for curve in &mut cloned.curves {
            for point in &mut curve.control_points {
                point.component_mul_assign(&factors);
            }
        }
    
        cloned
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

    /// Create a bezier patch that is just a flat plane in xy.
    pub fn default() -> BezierModelParameters {
        BezierModelParameters {
            symbol: None,
            patches: vec![BezierPatchParameters::default()]
        }
    }
}

/// All possible planes that can be used to mirror a bezier model.
#[derive(Clone, Copy)]
pub enum MirrorPlane {
    XY,
    XZ,
    YZ,
    /// No mirroring will be performed. This is present to make GUI logic easier.
    None
}

impl MirrorPlane {
    /// Retrieve factor vector for given mirroring plane. This is used to
    /// transform the coordinates of the vertices of the cloned model.
    pub fn factors(&self) -> Vec3 {
        match self {
            Self::XY => Vec3::new(1.0, 1.0, -1.0),
            Self::XZ => Vec3::new(1.0, -1.0, 1.0),
            Self::YZ => Vec3::new(-1.0, 1.0, 1.0),
            Self::None => Vec3::new(1.0, 1.0, 1.0)
        }
    }
}