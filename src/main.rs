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

use crate::data::*;
use crate::data::patches::*;
use crate::scene::*;
use crate::scene::lsystem::*;

fn main() {
    let test = BezierPatchParameters::empty();
    println!("{}", test.evaluate(0.0, 0.0));




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
    {
        let (w, h) = window.get_size();
        viewport = Viewport::for_window(w as _, h as _);
    }

    let mut imgui = ImContext::create();

    let mut imgui_glfw = ImguiGLFW::new(&mut imgui, &mut window);

    let mut show_menu = true;

    // ======== Scene setup =================
    let mut scene_manager = SceneManager::new();

    // Create initial scene
    {
        let (w, h) = window.get_size();

        scene_manager.push_scene(
            make_rc_cell(
                LSystemScene::new(
                    &LSystemParameters::from_string(data::presets::PENROSE),
                    &ApplicationSettings::default_settings(),
                    w as _,
                    h as _
                )
            )
        );
    }
    // ======================================

    viewport.enable();

    while !window.should_close() {
        unsafe {
            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
        }
        
        // The scene manager action emitted by the folling scene render.
        let action;
        {
            // Borrow mutable reference to the current scene for this frame
            let mut scene = scene_manager.current_scene().borrow_mut();

            // Perform logic
            scene.do_logic();

            // Render scene to screen
            scene.render();

            // Render the gui
            {
                let ui = imgui_glfw.frame(&mut window, &mut imgui);
                action = scene.do_gui(&ui);
                imgui_glfw.draw(ui, &mut window);
            }      
            
            // Present newly rendered frame to screen
            window.swap_buffers();

            // Handle input events
            glfw.poll_events();
            for (_, event) in glfw::flush_messages(&events) {
                imgui_glfw.handle_event(&mut imgui, &event);

                // Only pass events to scene if imgui does not capture them
                if !imgui.io().want_capture_mouse && !imgui.io().want_capture_keyboard {
                    scene.handle_event(&window, &event);
                }

                match event {
                    glfw::WindowEvent::Key(glfw::Key::M, _, Action::Press, _) => {
                        show_menu = !show_menu;
                    },
                    glfw::WindowEvent::Size(w, h) => {
                        viewport.update(w as _, h as _);
                        viewport.enable();

                        // Notify the scene that the screen size has changed. This is important
                        // to update internal state, such as cameras.
                        scene.handle_resize(w as _, h as _);
                    },
                    _ => {},
                }
            }

        }

        // Process action
        scene_manager.process_action(action);
    }
}
