use std::time::Instant;

use winit::{
    event::*,
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder, dpi::PhysicalPosition,
};

mod state;
use state::*;

pub mod binding_structs;
use binding_structs::*;

#[cfg(target_arch="wasm32")]
use wasm_bindgen::prelude::*;

#[cfg_attr(target_arch="wasm32", wasm_bindgen(start))]
pub async fn run() {
    cfg_if::cfg_if! {
        if #[cfg(target_arch = "wasm32")] {
            std::panic::set_hook(Box::new(console_error_panic_hook::hook));
            console_log::init_with_level(log::Level::Warn).expect("Couldn't initialize logger");
        } else {
            env_logger::init();
        }
    }

    println!("hi1");

    let event_loop = EventLoop::new();
    let window = WindowBuilder::new().build(&event_loop).unwrap();

    #[cfg(target_arch = "wasm32")]
    {
        // Winit prevents sizing with CSS, so we have to set
        // the size manually when on web.
        use winit::dpi::PhysicalSize;
        window.set_inner_size(PhysicalSize::new(450, 400));
        
        use winit::platform::web::WindowExtWebSys;
        web_sys::window()
            .and_then(|win| win.document())
            .and_then(|doc| {
                let dst = doc.get_element_by_id("wasm-example")?;
                let canvas = web_sys::Element::from(window.canvas());
                dst.append_child(&canvas).ok()?;
                Some(())
            })
            .expect("Couldn't append canvas to document body.");
    }

    let mut info = Info::default();
    info.rotate_around_y(0.0, 0.0);

    let mut state = State::new(window, info).await;
    let mut mouse_state = MouseState::default();
    let instant = Instant::now();

    event_loop.run(move |event, _, control_flow| match event {
        Event::WindowEvent {
            ref event,
            window_id,
        }  if window_id == state.window().id() => if !state.input(event) { 
            match event {
                WindowEvent::CloseRequested | WindowEvent::KeyboardInput {
                    input:
                        KeyboardInput {
                            state: ElementState::Pressed,
                            virtual_keycode: Some(VirtualKeyCode::Escape),
                            ..
                        },
                    ..
                } => *control_flow = ControlFlow::Exit,
                WindowEvent::Resized(physical_size) => {
                    state.resize(*physical_size);
                },
                WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                    // new_inner_size is &&mut so we have to dereference it twice
                    state.resize(**new_inner_size);
                },
                WindowEvent::CursorMoved {
                    position, 
                    .. 
                } => {
                    mouse_state.position = Some(position.cast());

                    if let Some(delta) = mouse_state.get_mouse_drag_delta() {
                        state.info.rotate_around_y(delta.x / 500.0, -delta.y / 500.0);
                    }

                    mouse_state.prev_position = mouse_state.position;
                },
                WindowEvent::MouseInput {
                    button,
                    state,
                    ..
                } => {
                    let is_down = *state == ElementState::Pressed;

                    match *button {
                        MouseButton::Left => mouse_state.left_down = is_down,
                        MouseButton::Right => mouse_state.right_down = is_down,
                        _ => {}
                    };
                },
                _ => {}
            }
        },
        Event::RedrawRequested(window_id) if window_id == state.window().id() => {
            state.info.time = instant.elapsed().as_millis() as f32;
            state.update();
            match state.render() {
                Ok(_) => {}
                // Reconfigure the surface if lost
                Err(wgpu::SurfaceError::Lost) => state.resize(state.size),
                // The system is out of memory, we should probably quit
                Err(wgpu::SurfaceError::OutOfMemory) => *control_flow = ControlFlow::Exit,
                // All other errors (Outdated, Timeout) should be resolved by the next frame
                Err(e) => eprintln!("{:?}", e),
            }
        },
        Event::MainEventsCleared => {
            // RedrawRequested will only trigger once, unless we manually
            // request it.
            state.window().request_redraw();
        },
        _ => {}
    });
}

struct MouseState {
    pub left_down: bool,
    pub right_down: bool,
    pub position: Option<PhysicalPosition<f32>>,
    pub prev_position: Option<PhysicalPosition<f32>>
}

impl MouseState {
    fn get_mouse_drag_delta(&self) -> Option<PhysicalPosition<f32>> {
        if self.left_down {
            Some(PhysicalPosition { 
                x: self.position?.x - self.prev_position?.x, 
                y: self.position?.y - self.prev_position?.y, 
            })
        } else {
            None
        }
    }
}

impl Default for MouseState {
    fn default() -> Self {
        Self {
            left_down: false,
            right_down: false,
            position: None,
            prev_position: None
        }
    }
}