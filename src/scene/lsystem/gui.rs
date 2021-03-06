use imgui::{MenuItem, EditableColor, ColorEdit, ImStr, StyleColor, ImString, ImColor, Slider, Condition, Context as ImContext, Window as ImWindow, im_str, Ui};
use nalgebra_glm::Vec3;
use crate::scene::lsystem::*;
use crate::scene::*;
use crate::scene::bezier::*;
use crate::data;
use crate::data::bezier::*;
use crate::data::*;
use crate::gui_utils::*;
use lsystems_core::drawing::types::*;
use lsystems_core::drawing::TurtleCommand;
use nfd::*;
use std::fs::*;


fn do_color_palette_entry(ui: &Ui, value: &mut Vec3, idx: usize) -> bool {
    let outer_id = ui.push_id(idx as i32);
    
    let mut color: [f32; 3] = [value.x, value.y, value.z];

    let mut changed = false;
    if ColorEdit::new(im_str!(""), &mut color).build(ui) {
        let new_color = Vec3::new(color[0], color[1], color[2]);
        *value = new_color;
        changed = true;
    }

    outer_id.pop(ui);
    return changed;
}

pub fn do_lsystem_params_gui(ui: &Ui, lsystem: &mut LSystemScene) -> SceneAction {
    let mut action = SceneAction::Nothing;

    ImWindow::new(&ImString::new(&lsystem.lsystem_params.name))
            .size([450.0, 550.0], Condition::FirstUseEver)
            .position([0.0, 60.0], Condition::FirstUseEver)
            .build(&ui, || {
                if ui.collapsing_header(im_str!("Drawing Parameters"))
                    .default_open(true)
                    .build() {
                    ui.indent();
                    do_drawing_parameters(ui, lsystem);
                    ui.unindent();
                }

                if ui.collapsing_header(im_str!("Rules"))
                    .default_open(true)
                    .build() {
                    ui.indent();
                    do_rules(ui, lsystem);
                    ui.unindent();
                }

                if ui.collapsing_header(im_str!("Interpretation Map"))
                    .default_open(true)
                    .build() {
                    ui.indent();
                    do_interpretations(ui, lsystem);
                    ui.unindent();
                }

                if ui.collapsing_header(im_str!("Color Palette"))
                    .default_open(false)
                    .build() {
                    ui.indent();
                    do_colors(ui, lsystem);
                    ui.unindent();
                }

                if ui.collapsing_header(im_str!("Bezier Patch Models"))
                    .default_open(false)
                    .build() {
                    ui.indent();
                    do_bezier_models(ui, lsystem, &mut action);
                    ui.unindent();
                }

                if ui.collapsing_header(im_str!("Application Settings"))
                    .default_open(true)
                    .build() {
                    ui.indent();
                    do_app_settings(ui, lsystem);
                    ui.unindent();
                }

                if ui.collapsing_header(im_str!("Debug Options"))
                    .default_open(false)
                    .build() {
                    ui.indent();
                    do_debug_options(ui, lsystem);
                    ui.unindent();
                }
            });

    action
}

fn draw_operations() -> Vec<&'static ImStr> {
    vec![
        im_str!("Forward"),
        im_str!("Forward (no draw)"),
        im_str!("Turn Right"),
        im_str!("Turn Left"),
        im_str!("Save State"),
        im_str!("Load State"),
        im_str!("Ignore"),
        im_str!("Forward (contracting)"),
        im_str!("Pitch Down"),
        im_str!("Pitch Up"),
        im_str!("Roll Left"),
        im_str!("Roll Right"),
        im_str!("Turn Around"),
        im_str!("Begin Polygon"),
        im_str!("End Polygon"),
        im_str!("Submit Vertex"),
        im_str!("Increment Color"),
        im_str!("Decrement Color"),
        im_str!("Increment Line Width"),
        im_str!("Decrement Line Width"),
    ]
}

fn do_bezier_models(ui: &Ui, system: &mut LSystemScene, action: &mut SceneAction) {

    //let mut to_rename: Option<(usize, char, char)> = None;
    let mut to_delete: Option<usize> = None;
    let mut to_edit: Option<usize> = None;

    // We need to push an outer ID here since we are using buttons with the same identifiers as the ones
    // used to remove and add rules.
    let outer_id = ui.push_id(4);

    for (i, model) in system.lsystem_params.bezier_models.iter_mut().enumerate() {
        let id = ui.push_id(i as i32);

        let mut symbol_str = ImString::with_capacity(16);

        if let Some(symbol) = model.symbol {
            symbol_str.push_str(&symbol.to_string());
        }

        let token = ui.push_item_width(20.0);

        // Save the old symbol for later renaming
        let old_symbol = model.symbol;

        if ui.input_text(im_str!("##sym"), &mut symbol_str).build() {
            let trimmed = symbol_str.to_str().trim();
            if trimmed.is_empty() {
                model.symbol = None;
            } else {
                model.symbol = Some(trimmed.chars().next().unwrap());
            }

            if let Some(old_symbol) = old_symbol {
                // Case 1: Simple rename
                if let Some(new_symbol) = model.symbol {
                    system.bezier_manager.rename_meshes(old_symbol, new_symbol);
                } else if let None = model.symbol {
                    // Case 2: Removed symbol
                    system.bezier_manager.remove_meshes(old_symbol);
                }
            } else {
                // Case 3: Has gotten a name when it didnt have one before
                if let Some(new_symbol) = model.symbol {
                    system.bezier_manager.update_meshes(model);
                }
            }
        }

        token.pop(ui);

        ui.same_line(0.0);

        if ui.button(im_str!("edit"), [0.0, 0.0]) {
            to_edit = Some(i);
        }

        let colors = ui.push_style_colors(&[
            (StyleColor::Button, [0.6, 0.239, 0.239, 1.0]),
            (StyleColor::ButtonHovered, [0.7, 0.2117, 0.2117, 1.0]),
            (StyleColor::ButtonActive, [0.8, 0.1607, 0.1607, 1.0])
        ]);

        ui.same_line(0.0);

        if ui.button(im_str!("-"), [0.0, 0.0]) {
            to_delete = Some(i);
        }
        
        colors.pop(ui);
        /*  */

        id.pop(ui);
    }

    match to_edit {
        Some(i) => {
            *action = SceneAction::PushScene(
                make_rc_cell(
                    BezierEditorScene::new(system.edit_bezier_model(i), system.width, system.height)
                )
            );
        },
        _ => {}
    }

    match to_delete {
        Some(i) => {
            // Remove mesh from manager, if the symbol was not empty.
            let id = system.lsystem_params.bezier_models[i].symbol;

            // Remove model parameters from lsystem parameters.
            system.lsystem_params.bezier_models.remove(i);

            if let Some(id) = id {
                system.bezier_manager.remove_meshes(id);

                // Search if there is another bezier model with the same identifier. 
                // This improves UX by automatically loading the meshes for it. Otherwise the user
                // would need to refresh the identifier for the other model.
                for model in &system.lsystem_params.bezier_models {
                    if let Some(other_id) = model.symbol {
                        if other_id == id {
                            system.bezier_manager.update_meshes(model);
                            break;
                        }
                    }
                }

                // Refresh view
                system.refresh_bezier_models();
            }
        }
        _ => {}
    };


    let colors = ui.push_style_colors(&[
        (StyleColor::Button, [0.349, 0.6, 0.239, 1.0]),
        (StyleColor::ButtonHovered, [0.3568, 0.7019, 0.2117, 1.0]),
        (StyleColor::ButtonActive, [0.3529, 0.8, 0.1607, 1.0])
    ]);

    if ui.button(im_str!("+"), [0.0, 0.0]) {
        // We do not have to refresh anything here, since per default, the models have an
        // empty identifier. This means their mesh is not generated.
        system.lsystem_params.bezier_models.push(
            BezierModelParameters::default()
        );
    }

    colors.pop(ui);
    outer_id.pop(ui);
}

fn index_to_operation(index: usize) -> TurtleCommand {
    match index {
        0 => TurtleCommand::Forward,
        1 => TurtleCommand::ForwardNoDraw,
        2 => TurtleCommand::TurnRight,
        3 => TurtleCommand::TurnLeft,
        4 => TurtleCommand::SaveState,
        5 => TurtleCommand::LoadState,
        6 => TurtleCommand::Ignore,
        7 => TurtleCommand::ForwardContracting,
        8 => TurtleCommand::PitchDown,
        9 => TurtleCommand::PitchUp,
        10 => TurtleCommand::RollLeft,
        11 => TurtleCommand::RollRight,
        12 => TurtleCommand::TurnAround,
        13 => TurtleCommand::BeginPolygon,
        14 => TurtleCommand::EndPolygon,
        15 => TurtleCommand::SubmitVertex,
        16 => TurtleCommand::IncrementColor,
        17 => TurtleCommand::DecrementColor,
        18 => TurtleCommand::IncrementLineWidth,
        19 => TurtleCommand::DecrementLineWidth,
        _ => panic!("Unknown draw operation value")
    }
}

fn save_text_file(path: &str, contents: &str) {
    write(path, contents).expect("Unable to write file");
}

fn load_text_file(path: &str) -> String {
    read_to_string(path).expect("Unable to read file")
}

fn do_colors(ui: &Ui, lsystem: &mut LSystemScene) {
    let mut was_changed = false;
    for (i, color) in &mut lsystem.lsystem_params.color_palette.iter_mut().enumerate() {
        if do_color_palette_entry(ui, color, i) {
            was_changed = true;
        }
    }

    if was_changed {
        lsystem.refresh_color_palette();
    }
}

pub fn do_main_menu_bar(ui: &Ui, lsystem: &mut LSystemScene) {
    if let Some(token) = ui.begin_main_menu_bar() {
        do_file_menu(ui, lsystem);
        do_presets(ui, lsystem);
        token.end(ui);
    }
}

fn do_presets(ui: &Ui, lsystem: &mut LSystemScene) {
    if let Some(token) = ui.begin_menu(im_str!("Examples"), true) {
        MenuItem::new(im_str!("2D"))
            .enabled(false)
            .build(ui);

        if MenuItem::new(im_str!("Koch Snowflake")).build(ui) {
            lsystem.load(data::presets::KOCH_SNOWFLAKE);
        }

        if MenuItem::new(im_str!("Penrose")).build(ui) {
            lsystem.load(data::presets::PENROSE);
        }


        ui.separator();

        MenuItem::new(im_str!("3D"))
            .enabled(false)
            .build(ui);

        token.end(ui);
    }
}

fn do_file_menu(ui: &Ui, lsystem: &mut LSystemScene) {
    if let Some(token) = ui.begin_menu(im_str!("File"), true) {
        if MenuItem::new(im_str!("New"))
            .shortcut(im_str!("      Ctrl+N"))
            .build(ui) {
                lsystem.load(data::presets::EMPTY);
        }

        if MenuItem::new(im_str!("Open"))
            .shortcut(im_str!("      Ctrl+O"))
            .build(ui) {
                let result = nfd::open_file_dialog(Some("json"), None).unwrap_or_else(|e| {
                    panic!(e);
                });

                match result {
                    Response::Okay(path) => {
                        let json = load_text_file(&path);
                        lsystem.load(&json);
                    },
                    Response::OkayMultiple(paths) => {
                        let json = load_text_file(&paths.iter().next().unwrap());
                        lsystem.load(&json);
                    },
                    // User canceled
                    _ => {}
                }
        }

        if MenuItem::new(im_str!("Save"))
            .shortcut(im_str!("      Ctrl+S"))
            .build(ui) {
                let result = nfd::open_save_dialog(Some("json"), None).unwrap_or_else(|e| {
                    panic!(e);
                });

                match result {
                    Response::Okay(path) => {
                        let json = lsystem.save();
                        save_text_file(&path, &json);
                    },
                    // User canceled, and multiple cant ever happen here
                    _ => {}
                }
        }

        token.end(ui);
    }
}

fn do_debug_options(ui: &Ui, lsystem: &mut LSystemScene) {
    if ui.checkbox(im_str!("Show normal vectors"), &mut lsystem.app_settings.show_normals) {
        lsystem.refresh_meshes();
    }

    if ui.checkbox(im_str!("Draw polygons as wireframe"), &mut lsystem.app_settings.draw_wireframe) {
        lsystem.refresh_wireframe_flag();
    }
}

fn do_interpretations(ui: &Ui, lsystem: &mut LSystemScene) {
    let mut modified = false;

    let params = &mut lsystem.lsystem_params;

    let mut to_delete: Option<usize> = None;

    // We need to push an outer ID here since we are using buttons with the same identifiers as the ones
    // used to remove and add rules.
    let outer_id = ui.push_id(2);

    for (i, interp) in params.interpretations.iter_mut().enumerate() {
        let id = ui.push_id(i as i32);

        let mut symbol_str = ImString::with_capacity(16);

        if let Some(symbol) = interp.symbol {
            symbol_str.push_str(&symbol.to_string());
        }

        let token = ui.push_item_width(20.0);

        if ui.input_text(im_str!("##sym"), &mut symbol_str).build() {
            let trimmed = symbol_str.to_str().trim();
            if trimmed.is_empty() {
                interp.symbol = None;
            } else {
                interp.symbol = Some(trimmed.chars().next().unwrap());
            }

            modified = true;
        }

        token.pop(ui);

        ui.same_line(0.0);
        ui.text(im_str!("->"));
        ui.same_line(0.0);
        let mut current_item: i32 = interp.operation as _;
        let items = draw_operations();

        if ui.combo(im_str!("##op"), &mut current_item, &items, 8) {
            let new_operation = index_to_operation(current_item as _);
            interp.operation = new_operation;
            modified = true;
        }

        let colors = ui.push_style_colors(&[
            (StyleColor::Button, [0.6, 0.239, 0.239, 1.0]),
            (StyleColor::ButtonHovered, [0.7, 0.2117, 0.2117, 1.0]),
            (StyleColor::ButtonActive, [0.8, 0.1607, 0.1607, 1.0])
        ]);

        ui.same_line(0.0);

        if ui.button(im_str!("-"), [0.0, 0.0]) {
            println!("To delete: {}", i);
            modified = true;
            to_delete = Some(i);
        }
        
        colors.pop(ui);

        id.pop(ui);     
    }  

    match to_delete {
        Some(i) => {
            params.interpretations.remove(i);
        }
        _ => {}
    };


    let colors = ui.push_style_colors(&[
        (StyleColor::Button, [0.349, 0.6, 0.239, 1.0]),
        (StyleColor::ButtonHovered, [0.3568, 0.7019, 0.2117, 1.0]),
        (StyleColor::ButtonActive, [0.3529, 0.8, 0.1607, 1.0])
    ]);

    if ui.button(im_str!("+"), [0.0, 0.0]) {
        params.interpretations.push(
            Interpretation{
                symbol: None,
                operation: TurtleCommand::Forward
            }
        );

        println!("Added new interp, size is now {}", params.interpretations.len());
        modified = true;
    }

    colors.pop(ui);
    outer_id.pop(ui);

    if modified {
        lsystem.refresh_interpretations();
    }
}

fn do_rules(ui: &Ui, lsystem: &mut LSystemScene) {
    let mut modified = false;
    let params = &mut lsystem.lsystem_params;

    let mut axiom = ImString::with_capacity(256);
    axiom.push_str(&params.axiom);
    if ui.input_text(im_str!("Axiom"), &mut axiom).build() {
        params.axiom = axiom.to_str().to_string();
        modified = true;
    }

    ui.text(im_str!("Production rules:"));
    ui.indent();

    // The rule to delete. It can only ever be one per frame, so this is enough.
    let mut to_delete = None;

    for (i, rule) in params.rules.iter_mut().enumerate() {
        let mut rule_str = ImString::with_capacity(256);
        rule_str.push_str(rule);

        let id = ui.push_id(i as i32);

        if ui.input_text(im_str!("##rule"), &mut rule_str).build() {
            *rule = rule_str.to_str().to_string();
            modified = true;
        }

        let colors = ui.push_style_colors(&[
            (StyleColor::Button, [0.6, 0.239, 0.239, 1.0]),
            (StyleColor::ButtonHovered, [0.7, 0.2117, 0.2117, 1.0]),
            (StyleColor::ButtonActive, [0.8, 0.1607, 0.1607, 1.0])
        ]);

        ui.same_line(0.0);

        if ui.button(im_str!("-"), [0.0, 0.0]) {
            modified = true;
            to_delete = Some(i);
        }

        colors.pop(ui);
        id.pop(ui);
    }

    let colors = ui.push_style_colors(&[
        (StyleColor::Button, [0.349, 0.6, 0.239, 1.0]),
        (StyleColor::ButtonHovered, [0.3568, 0.7019, 0.2117, 1.0]),
        (StyleColor::ButtonActive, [0.3529, 0.8, 0.1607, 1.0])
    ]);

    if ui.button(im_str!("+"), [0.0, 0.0]) {
        params.rules.push(String::new());
        modified = true;
    }

    colors.pop(ui);



    ui.unindent();

    // Handle deletion request
    match to_delete {
        Some(i) => {
            params.rules.remove(i);
        },
        _ => {}
    };

    if modified {
        lsystem.refresh_rules();
    }
}

fn do_drawing_parameters(ui: &Ui, lsystem: &mut LSystemScene) {  
    {
        let mut modified = false;
        let params = &mut lsystem.lsystem_params.drawing_parameters;

        let mut start_pos: [f32; 2] = [params.start_position.x as _, params.start_position.y as _,];
        if ui.drag_float2(im_str!("Starting position"), &mut start_pos)
            .min(-500.0)
            .max(500.0)
            .display_format(im_str!("%.2lf"))
            .speed(0.06)
            .build() {
                params.start_position = Vector2f::new(start_pos[0] as _, start_pos[1] as _);
                modified = true;
        }

        let mut start_angle: f32 = params.start_angle.to_degrees() as _;
        if ui.drag_float(im_str!("Starting angle"), &mut start_angle)
            .min(0.0)
            .max(360.0)
            .display_format(im_str!("%.lf"))
            .speed(1.0)
            .build() {
                params.start_angle = (start_angle as f64).to_radians();
                modified = true;
        }

        let mut delta_angle: f32 = params.angle_delta.to_degrees() as _;
        if ui.drag_float(im_str!("Angle delta"), &mut delta_angle)
            .min(0.0)
            .max(360.0)
            .display_format(im_str!("%.lf"))
            .speed(1.0)
            .build() {
                params.angle_delta = (delta_angle as f64).to_radians();
                modified = true;
        }

        let mut step: f32 = params.step as _;
        if ui.drag_float(im_str!("Step"), &mut step)
            .min(0.0)
            .max(360.0)
            .display_format(im_str!("%.2lf"))
            .speed(0.01)
            .build() {
                params.step = step as _;
                modified = true;
        }

        let mut line_width: f32 = params.initial_line_width as _;
        if ui.drag_float(im_str!("Line Width"), &mut line_width)
            .min(0.0)
            .max(360.0)
            .display_format(im_str!("%.2lf"))
            .speed(0.01)
            .build() {
                params.initial_line_width = line_width as _;
                modified = true;
        }

        let mut line_delta: f32 = params.line_width_delta as _;
        if ui.drag_float(im_str!("Line Width Delta"), &mut line_delta)
            .min(0.0)
            .max(360.0)
            .display_format(im_str!("%.2lf"))
            .speed(0.01)
            .build() {
                params.line_width_delta = line_delta as _;
                modified = true;
        }

        if modified {
            lsystem.refresh_drawing_parameters();
        }
    }

    // Technically, the iteration depth is not path of the drawing parameters, but it is displayed in the same section. 
    {
        if Slider::<u32>::new(im_str!("Iterations"), 0..=13).build(ui, &mut lsystem.lsystem_params.iteration_depth) {
            lsystem.refresh_iteration_depth();
        }

        let mut current_item: i32 = lsystem.lsystem_params.line_draw_mode as _;
        let items = vec![im_str!("Legacy Lines"), im_str!("2D Lines"), im_str!("3D Lines")];

        if ui.combo(im_str!("Line Mode"), &mut current_item, &items, 3) {
            let new_mode = match current_item {
                0 => LineDrawMode::Basic,
                1 => LineDrawMode::Advanced2D,
                _ => LineDrawMode::Advanced3D
            };

            lsystem.lsystem_params.line_draw_mode = new_mode;
            lsystem.draw_lsystem();
        }

        ui.same_line(0.0);
        help_marker(ui, im_str!("Three approaches to rendering lines are supported:\n\
                                 \tLegacy: Renders lines using built-in OpenGL functionality. Does not support custom widths.\n\
                                 \t2D: Uses a custom geometry shader to render lines as triangle strips. Supports arbitrary widths.\n\
                                 \t3D: Renders lines as 3D tubes. Useful for more realistic looking models, like plants."));
    }
}

pub fn do_debug_gui(ui: &Ui) {
    ImWindow::new(im_str!("Debug"))
            .size([85.0, 55.0], Condition::Always)
            .position([0.0, 0.0], Condition::Always)
            .build(&ui, || {
                let fps = ui.io().framerate;
                ui.text(format!(
                    "FPS: {:.1}",
                    fps
                ));
                
            });
}

fn do_app_settings(ui: &Ui, lsystem: &mut LSystemScene) {
    ui.checkbox(im_str!("Auto refresh"), &mut lsystem.app_settings.auto_refresh);

    if !lsystem.app_settings.auto_refresh {
        ui.same_line_with_spacing(0.0, 30.0);
        if ui.button(im_str!("Reload"), [0.0, 0.0]) {
            lsystem.force_refresh_all();
        }
    } else {
        ui.same_line(0.0);
        help_marker(ui, im_str!("Auto refresh can make editing large L-Systems rather slow. Consider disabling the option when dealing with big systems."))
    }

    ui.spacing();

    ui.checkbox(im_str!("Center camera on reload"), &mut lsystem.app_settings.auto_center_camera);
    ui.same_line(0.0);
    help_marker(ui, im_str!("Causes the camera to be focused on the center of the L-System's bounding box on reload, which makes rotation more enjoyable."));
    ui.same_line_with_spacing(0.0, 30.0);
    if ui.button(im_str!("Center"), [0.0, 0.0]) {
        lsystem.center_camera();
    }

    if lsystem.app_settings.auto_center_camera {
        ui.indent();
        ui.checkbox(im_str!("Also adjust camera zoom"), &mut lsystem.app_settings.auto_adjust_radius);ui.same_line(0.0);
        help_marker(ui, im_str!("This will adjust the zoom level to always have the whole L-System in view."));    
        ui.unindent();
    }

    ui.spacing();

    ui.checkbox(im_str!("Draw bounding box"), &mut lsystem.app_settings.draw_bounding_box);

    if lsystem.app_settings.draw_bounding_box {
        let bbcolor = &mut lsystem.app_settings.bounding_box_color;
        let mut color: [f32; 3] = [bbcolor.x, bbcolor.y, bbcolor.z];

        ui.indent();
        if ColorEdit::new(im_str!("Box color"), &mut color).build(ui) {
            let new_color = Vec3::new(color[0], color[1], color[2]);
            lsystem.app_settings.bounding_box_color = new_color;
            lsystem.refresh_bounding_box_color();
        }
        ui.unindent();
    }
}