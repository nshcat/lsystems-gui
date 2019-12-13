use std::rc::*;
use std::cell::*;

use nalgebra_glm::Vec3;

use lsystems_core::*;
use lsystems_core::drawing::types::*;

use serde_json::*;

use crate::data::*;
use crate::data::patches::*;
use crate::rendering::*;
use crate::rendering::camera::*;
use crate::rendering::meshes::*;
use crate::rendering::materials::*;
use crate::rendering::traits::*;
use crate::scene::*;
use crate::scene::lsystem::bounding_box::*;
use crate::scene::lsystem::normal_test_material::*;
use crate::scene::lsystem::normal_color_test_material::*;

mod bounding_box;
mod normal_test_material;
mod normal_color_test_material;
mod gui;

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
    /// The triangle fan meshes generated by the LSystem
    polygon_meshes: Vec<Mesh>,
    /// The bounding box around the lsystem. It might not exist, for example if there arent enough points.
    bounding_box: Option<BoundingBox>,
    /// The camera looking into the scene
    camera: Camera,
    /// This option contains a reference shared with a BezierEditorScene instance that is running on top
    /// of this scene in the SceneManager. If at any point, this scene gets evaluated and this is not None,
    /// it contains the result of the editor scene and has to be handled.
    model_to_refresh: Option<(usize, RcCell<BezierModelParameters>)>,
    /// Screen width
    pub width: u32,
    /// Screen height
    pub height: u32
}

impl LSystemScene {
    /// Create LSystem manager instance with given initial lsystem
    pub fn new(params: &LSystemParameters, settings: &ApplicationSettings, w: u32, h: u32) -> LSystemScene {
        let mut lsystem = LSystem::new();

        Self::setup_lsystem(&mut lsystem, params);

        lsystem.iterate();
        lsystem.interpret();

        let poly_meshes = Self::retrieve_polygon_meshes(&lsystem, params, settings);
        let mesh = Self::retrieve_line_mesh(&lsystem, params);
        let bb = Self::calculate_bounding_box(&settings.bounding_box_color, &lsystem);

        let mut scene = LSystemScene{
            lsystem_params: params.clone(),
            app_settings: settings.clone(),
            lines_mesh: mesh,
            polygon_meshes: poly_meshes,
            lsystem,
            bounding_box: bb,
            camera: Camera::new(w, h, ProjectionType::Perspective(75.0)),
            model_to_refresh: None,
            width: w,
            height: h
        };

        if settings.auto_center_camera {
            scene.center_camera();
        } 

        scene
    }

    /// Mark the bezier model with given index as being "currently in edit mode".
    /// This means that a EditBezierScene is going to be the active scene and modify
    /// its contents.
    pub fn edit_bezier_model(&mut self, index: usize) -> RcCell<BezierModelParameters> {
        let model = self.lsystem_params.bezier_models[index].clone();

        let cell = make_rc_cell(model);

        self.model_to_refresh = Some((index, cell.clone()));

        cell
    }

    /// Calculate bounding box from given lsystem
    fn calculate_bounding_box(color: &Vec3, lsystem: &LSystem) -> Option<BoundingBox> {
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

        for poly in &lsystem.polygons {
            for vertex in &poly.vertices {
                vertices.push(convert_vector(vertex));
            }
        }

        // The bounding box can only be create if theres at least one vertex present
        match vertices.len() {
            0 => None,
            _ => Some(BoundingBox::new(color, &vertices))
        } 
    }

    /// Shortcut to auto refresh settings value
    fn auto_refresh(&self) -> bool {
        self.app_settings.auto_refresh
    }

    /// Center camera on lsystem with proper radius
    pub fn center_camera(&mut self) {
        // We can only center the camera if there exists a bounding box
        if let Some(bb) = &self.bounding_box {
            // Determine the center
            let center = bb.aabb.center().coords;
            self.camera.recenter(&center);

            // Adjust zoom level if requested
            if self.app_settings.auto_adjust_radius {
                self.camera.set_radius(bb.radius());
            }
        }
    }

    pub fn refresh_color_palette(&mut self) {
        self.lsystem_params.drawing_parameters.color_palette_size = self.lsystem_params.color_palette.len() as _;
        self.draw_lsystem();
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
        if let Some(bb) = &mut self.bounding_box {
            bb.set_color(&self.app_settings.bounding_box_color);
        }
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
        self.refresh_meshes();
        self.draw_bounding_box();

        // Since we redrew the lsystem, recenter camera if requested by the user
        if self.app_settings.auto_center_camera {
            self.center_camera();
        }
    }

    pub fn refresh_bezier_models(&mut self) {
        /* TODO */
    }

    /// Does not redraw lsystem, just recreates the meshes. Needed if mesh data changes, such as debug settings
    /// or the color palette entries.
    pub fn refresh_meshes(&mut self) {
        self.lines_mesh = Self::retrieve_line_mesh(&self.lsystem, &self.lsystem_params);
        self.polygon_meshes = Self::retrieve_polygon_meshes(&self.lsystem, &self.lsystem_params, &self.app_settings);
    }

    /// Notify scene that the wireframe setting has changed
    pub fn refresh_wireframe_flag(&mut self) {
        for mesh in &mut self.polygon_meshes {
            mesh.draw_wireframe = self.app_settings.draw_wireframe;
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

    /// Load lsystem parameters from JSON string.
    pub fn load(&mut self, json_str: &str) {
        let params = from_str::<LSystemParameters>(json_str);

        match params {
            Ok(params) => {
                self.lsystem_params = params;
                self.force_refresh_all();
            }
            Err(e) => {
                println!("Could not load given JSON string as LSystem parameters: {}", e);
            }
        };
    }

    /// Save lsystem parameters to JSON string.
    pub fn save(&mut self) -> String {
        to_string_pretty(&self.lsystem_params).unwrap()
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

    fn retrieve_polygon_meshes(lsystem: &LSystem, params: &LSystemParameters, settings: &ApplicationSettings) -> Vec<Mesh> {
        let mut meshes = Vec::new();

        for polygon in &lsystem.polygons {
            let color = if params.color_palette.len() > 0 {
                params.color_palette[polygon.color as usize]
            } else {
                Vec3::new(1.0, 1.0, 1.0)
            };

            let mut vertices = Vec::new();

            for vertex in &polygon.vertices {
                let position = Vec3::new(vertex.x as _, vertex.y as _, vertex.z as _);
                vertices.push(Vertex::new(position, color.clone()));
            }

            let mat = Box::new(SimpleMaterial::new());
            let geometry = BasicGeometry::with_auto_normals(PrimitiveType::TriangleFan, &vertices);
            let mut mesh = Mesh::new(PrimitiveType::TriangleFan, mat, &geometry);
            mesh.draw_wireframe = settings.draw_wireframe;
            meshes.push(mesh);

            if settings.show_normals {
                let mat = Box::new(NormalTestMaterial::new((params.drawing_parameters.step/2.0) as _, &Vec3::new(1.0, 1.0, 0.0)));
                let mut mesh = Mesh::new(PrimitiveType::TriangleStrip, mat, &geometry);
                mesh.draw_wireframe = settings.draw_wireframe;
                meshes.push(mesh);
            }
        }

        meshes
    }
}


impl Scene for LSystemScene {
    /// Render scene to screen. This also includes any GUI components.
    fn render(&self) {
        let mut params = self.camera.to_render_parameters();

        self.lines_mesh.render(&mut params);

        for mesh in &self.polygon_meshes {
            mesh.render(&mut params);
        }

        if let Some(bb) = &self.bounding_box {
            if self.app_settings.draw_bounding_box {
                bb.render(&mut params);
            }
        }
    }

    /// Perform logic. Currently, this means checking if a BezierEditorScene just ended, which would mean
    /// that the modified model has to be applied to the parameters of the current lsystem.
    fn do_logic(&mut self) {

        let mut should_clear = false;

        if let Some((i, r)) = &self.model_to_refresh {
            // Retrieve the new model parameters
            let parameters = r.borrow().clone();

            // Apply it
            self.lsystem_params.bezier_models[*i] = parameters;

            // We now need to refresh bezier models.
            self.refresh_bezier_models();

            should_clear = true;
        }

        if should_clear {
            // Clear it, so that we don't to the refreshing again next frame.
            self.model_to_refresh = None
        }
    }

    /// Show imgui GUI if needed.
    fn do_gui(&mut self, ui: &Ui) -> SceneAction {
        gui::do_main_menu_bar(ui, self);
        gui::do_lsystem_params_gui(ui, self)
    }

    /// Handle input event. This is only called if the UI does not want to grab input.
    fn handle_event(&mut self, window: &Window, event: &WindowEvent) {
        self.camera.handle_event(window, event);
    }

    /// Handle window resize event.
    fn handle_resize(&mut self, w: u32, h: u32) {
        self.camera.update(w, h);

        self.width = w;
        self.height = h;
    }
}