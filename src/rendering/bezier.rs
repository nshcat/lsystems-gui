use crate::data::bezier::*;
use crate::rendering::meshes::*;

/// Geometry generated from a bicubic bezier patch.
pub struct BezierGeometry {
    /// The underlying plane geometry
    plane: PlaneGeometry
}

impl BezierGeometry {
    /// Create a new bezier patch geometry based on given patch parameters and resolution values.
    pub fn new(parameters: &BezierPatchParameters, rows: u32, cols: u32) -> BezierGeometry {
        let mut plane = PlaneGeometry::new(rows, cols, parameters.color);

        // u
        for x in 0..=cols {
            let u = (x as f32) / (cols) as f32;

            // Evaluate all control point curves
            let pt0 = parameters.curves[0].evaluate(u);
            let pt1 = parameters.curves[1].evaluate(u);
            let pt2 = parameters.curves[2].evaluate(u);
            let pt3 = parameters.curves[3].evaluate(u);

            // v
            for y in 0..=rows {
                let v = (y as f32) / (rows) as f32;
                let index_base = y * (cols + 1);
                let index = (index_base + x) as usize;

                // Blend points together for this particular v
                let vertex = BezierCurveParameters::from_points([pt0, pt1, pt2, pt3]).evaluate(v);

                // Set vertex in plane geometry
                plane.set_vertex(index, vertex);
            }
        }

        plane.regenerate_normals();

        BezierGeometry{
            plane: plane
        }
    }
}

impl IndexedGeometry for BezierGeometry {
    fn retrieve_indices(&self) -> &[u32] {
        self.plane.retrieve_indices()
    }
}

impl Geometry for BezierGeometry {
    fn retrieve_attributes(&self) -> Vec<&dyn AttributeArrayBase> {
        self.plane.retrieve_attributes()
    }
}