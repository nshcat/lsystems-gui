use std::rc::*;
use std::cell::*;

use nalgebra_glm::Vec3;

use lsystems_core::*;
use lsystems_core::drawing::*;
use lsystems_core::drawing::primitives::*;
use lsystems_core::drawing::types::*;

use crate::data::*;
use crate::rendering::*;
use crate::rendering::camera::*;
use crate::rendering::meshes::*;
use crate::rendering::materials::*;
use crate::rendering::traits::*;
use crate::scene::bounding_box::*;

mod bounding_box;

/// A struct managing the currently displayed LSystem and providing methods
/// to update certain parts of it.
pub struct LSystemScene {
    /// The parameters describing the currently displayed LSystem.
    pub lsystem_params: LSystemParameters,
    /// The application settings
    pub app_settings: ApplicationSettings,
    /// The lsystem instance
    lsystem: LSystem,
    /// The mesh containing all lines of the lsystem
    lines_mesh: Mesh,
    /// The bounding box around the lsystem
    bounding_box: BoundingBox,
    /// The camera looking into the scene
    camera: Rc<RefCell<Camera>>
}

impl LSystemScene {
    /// Create LSystem manager instance with given initial lsystem
    pub fn new(params: &LSystemParameters, settings: &ApplicationSettings, camera: Rc<RefCell<Camera>>) -> LSystemScene {
        let mut lsystem = LSystem::new();

        Self::setup_lsystem(&mut lsystem, params);

        lsystem.iterate();
        lsystem.interpret();

        let mesh = Self::retrieve_line_mesh(&lsystem, params);
        let bb = Self::calculate_bounding_box(&settings.bounding_box_color, &lsystem);

        let mut scene = LSystemScene{
            lsystem_params: params.clone(),
            app_settings: settings.clone(),
            lines_mesh: mesh,
            lsystem,
            bounding_box: bb,
            camera
        };

        if settings.auto_center_camera {
            scene.center_camera();
        }

        scene
    }

    /// Calculate bounding box from given lsystem
    fn calculate_bounding_box(color: &Vec3, lsystem: &LSystem) -> BoundingBox {
        // Collect vertices
        let mut vertices = Vec::new();

        // Convert LSystem-Core vector to GLM vector
        fn convert_vector(vec: &Vector3f) -> Vec3 {
            Vec3::new(vec.x as _, vec.y as _, vec.z as _)
        }

        for line in &lsystem.line_segments {
            vertices.push(convert_vector(&line.begin));
            vertices.push(convert_vector(&line.end));
        }

        BoundingBox::new(color, &vertices)
    }

    /// Shortcut to auto refresh settings value
    fn auto_refresh(&self) -> bool {
        self.app_settings.auto_refresh
    }

    /// Center camera on lsystem with proper radius
    pub fn center_camera(&mut self) {
        // Determine the center
        let center = self.bounding_box.aabb.center().coords;
        self.camera.borrow_mut().recenter(&center);

        // Adjust zoom level if requested
        if self.app_settings.auto_adjust_radius {
            self.camera.borrow_mut().set_radius(self.bounding_box.radius());
        }
    }



    pub fn force_refresh_all(&mut self) {
        self.lsystem.set_drawing_parameters(&self.lsystem_params.drawing_parameters);
        self.lsystem.set_iteration_depth(self.lsystem_params.iteration_depth);
        self.apply_interpretations();
        self.apply_rules();

        self.iterate_lsystem();
        self.draw_lsystem();
    }

    /// Redraw the bounding box. Should be called when the lsystem was newly drawn.
    fn draw_bounding_box(&mut self) {
        self.bounding_box = Self::calculate_bounding_box(&self.app_settings.bounding_box_color, &self.lsystem);
    }

    pub fn refresh_bounding_box_color(&mut self) {
        self.bounding_box.set_color(&self.app_settings.bounding_box_color);
    }

    /// Notify scene that the  drawing parameters have changed
    pub fn refresh_drawing_parameters(&mut self) {
        if !self.auto_refresh() {
            return;
        }

        self.lsystem.set_drawing_parameters(&self.lsystem_params.drawing_parameters);
        self.draw_lsystem();
    }

    pub fn refresh_iteration_depth(&mut self) {
        if !self.auto_refresh() {
            return;
        }

        self.lsystem.set_iteration_depth(self.lsystem_params.iteration_depth);
        self.iterate_lsystem();
        self.draw_lsystem();
    }

    pub fn refresh_rules(&mut self) {
        if !self.auto_refresh() {
            return;
        }

        self.apply_rules();
        self.iterate_lsystem();
        self.draw_lsystem();
    }

    pub fn refresh_interpretations(&mut self) {
        if !self.auto_refresh() {
            return;
        }

        self.apply_interpretations();
        self.iterate_lsystem();
        self.draw_lsystem();
    }

    /// Apply interpretations in the lsystem parameters to the current lsystem instance
    fn apply_interpretations(&mut self) {
        self.lsystem.interpretations.clear();

        for interpretation in &self.lsystem_params.interpretations {
            if let Some(symbol) = interpretation.symbol {
                self.lsystem.interpretations.associate(symbol, interpretation.operation);
            }
        }
    }

    /// Apply axiom and rules stored in the lsystem parameters to the current lsystem instance
    fn apply_rules(&mut self) {
        self.lsystem.parse(&self.lsystem_params.axiom, &self.lsystem_params.rules.join("\n"));
    }

    /// Fully reiterate the lsystem. This is necessary if the iteration depth, the axiom or one or more 
    /// rules changed.
    fn iterate_lsystem(&mut self) {
        self.lsystem.iterate();
    }

    /// Draw the lsystem, which means interpreting it and retrieving all scene objects from it
    fn draw_lsystem(&mut self) {
        self.lsystem.interpret();
        self.lines_mesh = Self::retrieve_line_mesh(&self.lsystem, &self.lsystem_params);
        self.draw_bounding_box();

        // Since we redrew the lsystem, recenter camera if requested by the user
        if self.app_settings.auto_center_camera {
            self.center_camera();
        }
    }

    /// Setup new lsystem instance using given parameters. This will not start
    /// iteration!
    fn setup_lsystem(lsystem: &mut LSystem, params: &LSystemParameters) {
        lsystem.set_iteration_depth(params.iteration_depth);
        lsystem.set_drawing_parameters(&params.drawing_parameters);
        lsystem.parse(&params.axiom, &params.rules.join("\n"));
        lsystem.engine.set_seed(params.seed);

        for interp in &params.interpretations {
            if let Some(symbol) = interp.symbol {
                lsystem.interpretations.associate(symbol, interp.operation);
            }
        }
    }

    /// Create line mesh from interpreted lsystem
    fn retrieve_line_mesh(lsystem: &LSystem, params: &LSystemParameters) -> Mesh {
        // We are using a flat color material here.
        let mat = Box::new(SimpleMaterial::new());

        // Buffer for line vertices
        let mut vertices = Vec::new();

        // Convert LSystem-Core vector to GLM vector
        fn convert_vector(vec: &Vector3f) -> Vec3 {
            Vec3::new(vec.x as _, vec.y as _, vec.z as _)
        }

        for segment in &lsystem.line_segments {
            // Lookup color
            let color_index = if segment.color >= lsystem.parameters.color_palette_size as _ { 
                lsystem.parameters.color_palette_size - 1
            } else {
                segment.color as _
            };

            let color = if params.color_palette.len() == 0 {
                Vec3::repeat(1.0)
            } else {
                params.color_palette[color_index as usize]
            };

            let begin = Vertex::new(convert_vector(&segment.begin), color);
            let end = Vertex::new(convert_vector(&segment.end), color);
    
            vertices.push(begin);
            vertices.push(end);
        }

        Mesh::new(PrimitiveType::Lines, mat, &BasicGeometry::from_vertices(&vertices))
    }
}

impl Render for LSystemScene {
    /// Render lsystem to screen
    fn render(&self, params: &mut RenderParameters) {
        self.lines_mesh.render(params);

        if self.app_settings.draw_bounding_box {
            self.bounding_box.render(params);
        }
    }
}