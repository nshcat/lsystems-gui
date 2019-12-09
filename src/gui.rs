use imgui::{ImStr, StyleColor, ImString, ImColor, Slider, Condition, Context as ImContext, Window as ImWindow, im_str, Ui};
use crate::scene::*;
use lsystems_core::drawing::types::*;
use lsystems_core::drawing::DrawOperation;

pub fn do_lsystem_params_gui(ui: &Ui, lsystem: &mut LSystemManager) {
    ImWindow::new(im_str!("LSystem Parameters"))
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
            });
}

fn draw_operations() -> Vec<&'static ImStr> {
    vec![
        im_str!("Forward"),
        im_str!("ForwardNoDraw"),
        im_str!( "TurnRight"),
        im_str!("TurnLeft"),
        im_str!("SaveState"),
        im_str!("LoadState"),
        im_str!("Ignore"),
        im_str!("ForwardContracting"),
        im_str!("PitchDown"),
        im_str!("PitchUp"),
        im_str!("RollLeft"),
        im_str!("RollRight"),
        im_str!("TurnAround"),
        im_str!("BeginPolygon"),
        im_str!("EndPolygon"),
        im_str!("SubmitVertex"),
        im_str!("IncrementColor"),
        im_str!("DecrementColor"),
        im_str!("IncrementLineWidth"),
        im_str!("DecrementLineWidth"),
    ]
}

fn index_to_operation(index: usize) -> DrawOperation {
    match index {
        0 => DrawOperation::Forward,
        1 => DrawOperation::ForwardNoDraw,
        2 => DrawOperation::TurnRight,
        3 => DrawOperation::TurnLeft,
        4 => DrawOperation::SaveState,
        5 => DrawOperation::LoadState,
        6 => DrawOperation::Ignore,
        7 => DrawOperation::ForwardContracting,
        8 => DrawOperation::PitchDown,
        9 => DrawOperation::PitchUp,
        10 => DrawOperation::RollLeft,
        11 => DrawOperation::RollRight,
        12 => DrawOperation::TurnAround,
        13 => DrawOperation::BeginPolygon,
        14 => DrawOperation::EndPolygon,
        15 => DrawOperation::SubmitVertex,
        16 => DrawOperation::IncrementColor,
        17 => DrawOperation::DecrementColor,
        18 => DrawOperation::IncrementLineWidth,
        19 => DrawOperation::DecrementLineWidth,
        _ => panic!("Unknown draw operation value")
    }
}

fn do_interpretations(ui: &Ui, lsystem: &mut LSystemManager) {
    let mut modified = false;

    let params = &mut lsystem.lsystem_params;

    let mut to_change: Option<(char, Option<(char, DrawOperation)>)> = None;

    for (i, (symbol, op)) in params.interpretations.iter().enumerate() {
        let id = ui.push_id(i as i32);

        let mut symbol_str = ImString::with_capacity(16);
        symbol_str.push_str(&symbol.to_string());

        let token = ui.push_item_width(20.0);

        if ui.input_text(im_str!("##sym"), &mut symbol_str).build() {
            if symbol_str.to_str().len() > 0 {
                to_change = Some((*symbol, Some((symbol_str.to_str().chars().next().unwrap(), *op))));
            } else {
                to_change = Some((*symbol, None));
            }

            modified = true;
        }

        token.pop(ui);

        ui.same_line(0.0);
        ui.text(im_str!("->"));
        ui.same_line(0.0);
        let mut current_item: i32 = *op as _;
        let items = draw_operations();

        if ui.combo(im_str!("##op"), &mut current_item, &items, 5) {
            to_change = Some((*symbol, Some((*symbol, index_to_operation(current_item as _)))));
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
            to_change = Some((*symbol, None));
        }
        
        colors.pop(ui);

        id.pop(ui);
    }  

    match to_change {
        Some((oldsym, Some((newsym, op)))) => {
            if params.interpretations.contains_key(&oldsym) {
                params.interpretations.remove(&oldsym).unwrap();
            }

            params.interpretations.insert(newsym, op);
        },
        Some((oldsym, None)) => {
            params.interpretations.remove(&oldsym).unwrap();
        },
        _ => {}
    };


    let colors = ui.push_style_colors(&[
        (StyleColor::Button, [0.349, 0.6, 0.239, 1.0]),
        (StyleColor::ButtonHovered, [0.3568, 0.7019, 0.2117, 1.0]),
        (StyleColor::ButtonActive, [0.3529, 0.8, 0.1607, 1.0])
    ]);

    if ui.button(im_str!("+"), [0.0, 0.0]) {
        params.interpretations.insert(' ', DrawOperation::Forward);
        modified = true;
    }

    colors.pop(ui);

    if modified {
        lsystem.refresh_interpretations();
    }
}

fn do_rules(ui: &Ui, lsystem: &mut LSystemManager) {
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

fn do_drawing_parameters(ui: &Ui, lsystem: &mut LSystemManager) {  
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

        if modified {
            lsystem.refresh_drawing_parameters();
        }
    }

    // Technically, the iteration depth is not path of the drawing parameters, but it is displayed in the same section. 
    {
        if Slider::<u32>::new(im_str!("Iterations"), 0..=13).build(ui, &mut lsystem.lsystem_params.iteration_depth) {
            lsystem.refresh_iteration_depth();
        }
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