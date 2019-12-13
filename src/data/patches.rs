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

    /// Evalute the patch at given UV position in [0,1]x[0,1].
    pub fn evaluate(&self, u: f32, v: f32) -> Vec3 {
        let curve0 = &self.curves[0];
        let curve1 = &self.curves[1];
        let curve2 = &self.curves[2];
        let curve3 = &self.curves[3];

        type MatType = nalgebra::Matrix<f32, nalgebra::Dynamic, nalgebra::Dynamic, nalgebra::VecStorage<f32, nalgebra::Dynamic, nalgebra::Dynamic>>;

        let GBx = MatType::from_row_slice(4, 4,
            &[curve0.control_points[0].z, curve0.control_points[1].z, curve0.control_points[2].z, curve0.control_points[3].z, 
            curve1.control_points[0].z, curve1.control_points[1].z, curve1.control_points[2].z, curve1.control_points[3].z, 
            curve2.control_points[0].z, curve2.control_points[1].z, curve2.control_points[2].z, curve2.control_points[3].z, 
            curve3.control_points[0].z, curve3.control_points[1].z, curve3.control_points[2].z, curve3.control_points[3].z]
        );

        let GBy = MatType::from_row_slice(4, 4,
            &[curve0.control_points[0].y, curve0.control_points[1].y, curve0.control_points[2].y, curve0.control_points[3].y, 
            curve1.control_points[0].y, curve1.control_points[1].y, curve1.control_points[2].y, curve1.control_points[3].y, 
            curve2.control_points[0].y, curve2.control_points[1].y, curve2.control_points[2].y, curve2.control_points[3].y, 
            curve3.control_points[0].y, curve3.control_points[1].y, curve3.control_points[2].y, curve3.control_points[3].y] 
        );

        let GBz = MatType::from_row_slice(4, 4,
            &[curve0.control_points[0].z, curve0.control_points[1].z, curve0.control_points[2].z, curve0.control_points[3].z, 
            curve1.control_points[0].z, curve1.control_points[1].z, curve1.control_points[2].z, curve1.control_points[3].z, 
            curve2.control_points[0].z, curve2.control_points[1].z, curve2.control_points[2].z, curve2.control_points[3].z, 
            curve3.control_points[0].z, curve3.control_points[1].z, curve3.control_points[2].z, curve3.control_points[3].z], 
        );

        let U = MatType::from_row_slice(1, 4,
            &[u.powf(3.0), u.powf(2.0), u, 1.0]
        );

        let Vt = MatType::from_row_slice(4, 1,
            &[v.powf(3.0), v.powf(2.0), v, 1.0]
        );

        let Mb = MatType::from_row_slice(4, 4,
            &[  -1.0, 3.0, -3.0, 1.0,
                3.0, -6.0, 3.0, 0.0,
                -3.0, 3.0, 0.0, 0.0,
                1.0, 0.0, 0.0, 0.0]
        );

        let Mbt = Mb.transpose();

        let resultX = &U * &Mb * &GBx * &Mbt * &Vt;
        let resultY = &U * &Mb * &GBy * &Mbt * &Vt;
        let resultZ = &U * &Mb * &GBz * &Mbt * &Vt;

        Vec3::new(resultX[0], resultY[0], resultZ[0])
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

