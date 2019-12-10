use std::rc::*;
use std::cell::*;

use glfw::{Action, Key, Context, WindowEvent::Size, SwapInterval};
use imgui::{Condition, Context as ImContext, Window as ImWindow, im_str};
use imgui_glfw_rs::glfw;
use imgui_glfw_rs::imgui;
use imgui_glfw_rs::ImguiGLFW;
use serde_json::*;


use rendering::traits::*;
use rendering::camera::*;
use rendering::{Viewport};

mod rendering;
mod data;
mod scene;
mod gui;

use crate::data::*;
use crate::scene::*;

fn main() {
	let mut glfw = glfw::init(glfw::FAIL_ON_ERRORS).unwrap();
    glfw.window_hint(glfw::WindowHint::ContextVersion(3, 3)); 

    let (mut window, events) = glfw
        .create_window(
            1420,
            768,
            "lsystems-gui",
            glfw::WindowMode::Windowed,
        )
        .expect("Failed to create window");

    window.make_current();
    window.set_all_polling(true);

    gl::load_with(|symbol| window.get_proc_address(symbol) as *const _);
    unsafe {
        gl::Enable(gl::BLEND);
        gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);
        gl::Enable(gl::DEPTH_TEST);
        gl::DepthFunc(gl::LESS);
        gl::ClearColor(0.1, 0.1, 0.1, 1.0);
    }

    // Limit to 60 fps
    glfw.set_swap_interval(SwapInterval::Sync(1));

    let mut viewport;
    let mut camera;
    {
        let (w, h) = window.get_size();

        viewport = Viewport::for_window(w as _, h as _);

        camera = Rc::new(RefCell::new(Camera::new(
            w as _, h as _,
            ProjectionType::Perspective(75.0)
        )));
    }

    let mut imgui = ImContext::create();

    let mut imgui_glfw = ImguiGLFW::new(&mut imgui, &mut window);

    let mut show_menu = true;

    // ======== Scene setup =================
    let mut scene = LSystemScene::new(&LSystemParameters::from_string(data::presets::PENROSE), &ApplicationSettings::default_settings(), camera.clone());
    // ======================================

    viewport.enable();

    while !window.should_close() {
        unsafe {
            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
        }

        let mut params = camera.borrow().to_render_parameters();

        scene.render(&mut params);

        let ui = imgui_glfw.frame(&mut window, &mut imgui);

        // DRAW GUI ===
        //ui.show_demo_window(&mut true);
        gui::do_debug_gui(&ui);
        gui::do_lsystem_params_gui(&ui, &mut scene);
        // ============

        imgui_glfw.draw(ui, &mut window);

        window.swap_buffers();

        glfw.poll_events();
        for (_, event) in glfw::flush_messages(&events) {
            imgui_glfw.handle_event(&mut imgui, &event);

            // Only pass events to camera is imgui does not capture them
            if !imgui.io().want_capture_mouse && !imgui.io().want_capture_keyboard {
                camera.borrow_mut().handle_event(&window, &event);
            }

            match event {
                glfw::WindowEvent::Key(glfw::Key::M, _, Action::Press, _) => {
                    show_menu = !show_menu;
                },
                glfw::WindowEvent::Size(w, h) => {
                    viewport.update(w as _, h as _);
                    viewport.enable();

                    camera.borrow_mut().update(w as _, h as _);
                },
                _ => {},
            }
        }
    }
}
