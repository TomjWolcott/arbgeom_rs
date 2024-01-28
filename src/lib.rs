use std::time::Instant;

use winit::{
    dpi::PhysicalPosition,
    event::*,
    event_loop::EventLoop, window::WindowBuilder,
};

use game_loop::game_loop;
use winit::dpi::PhysicalSize;
use winit::window::{Fullscreen, Window};

mod state;
use state::*;

pub mod binding_structs;
use binding_structs::*;
use manifold::shapes4D::*;

pub mod manifold;

const MOVEMENT_BINDINGS: &'static [VirtualKeyCode] = &[
    VirtualKeyCode::D,      // +X
    VirtualKeyCode::A,      // -X
    VirtualKeyCode::Space,  // +Y
    VirtualKeyCode::LShift, // -Y
    VirtualKeyCode::W,      // +Z
    VirtualKeyCode::S,      // -Z
];

const IDEAL_FPS: f32 = 20.0;

#[cfg(target_arch="wasm32")]
use wasm_bindgen::prelude::*;

use crate::manifold::*;
use crate::manifold::shapes3D::{ExtrudedShape, Sphere, Torus};

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
    let window = WindowBuilder::new()
        .with_fullscreen(Some(Fullscreen::Borderless(None)))
        // .with_inner_size(PhysicalSize { width: 1200.0, height: 2000.0 })
        .build(&event_loop)
        .unwrap();
    // window.set_cursor_grab(winit::window::CursorGrabMode::Confined);

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

    let mut game = Game::new(
        &window,
        // Hyperplane,
        // Hypersphere::new(10.0),
        // Hypersphube::new(10.0, 2.0),
        Ditorus::new(10.0, 8.0, 3.0),
        // ExtrudedShape(Sphere::new(10.0)),
        // ExtrudedShape(Torus::new(7.0, 5.0)),
        info,
        Vec::new()
    ).await;

    game_loop(
        event_loop,
        window,
        game,
        10,
        0.1,
        |game_loop| {
            // game_loop.game.info.print_position();

            if game_loop.number_of_updates() % 1 == 0 {
                game_loop.game.info.reorient();
            }

            for active_keycode in game_loop.game.active_keycodes.iter() {
                game_loop.game.info.movement(*active_keycode, &game_loop.game.manifold);
            }

            game_loop.game.state.update_buffers(
                Some(&game_loop.game.info),
                Some(&game_loop.game.manifold)
            );

            match game_loop.game.state.render() {
                Ok(_) => {}
                // Reconfigure the surface if lost
                Err(wgpu::SurfaceError::Lost) => game_loop.game.state.resize(game_loop.game.state.size),
                // The system is out of memory, we should probably quit
                Err(wgpu::SurfaceError::OutOfMemory) => game_loop.exit(),
                // All other errors (Outdated, Timeout) should be resolved by the next frame
                Err(e) => eprintln!("{:?}", e),
            }

            // println!("FPS: {:.1}", 1.0 / game_loop.accumulated_time());
        }, |game_loop| {

        }, |game_loop, event| match event {
            Event::WindowEvent {
                ref event,
                window_id,
            }  if *window_id == game_loop.window.id() => match event {
                WindowEvent::KeyboardInput {
                    input: KeyboardInput {
                        state,
                        virtual_keycode: Some(keycode),
                        ..
                    },
                    ..
                } if MOVEMENT_BINDINGS.contains(keycode) => {
                    if *state == ElementState::Pressed && !game_loop.game.active_keycodes.contains(keycode) {
                        game_loop.game.active_keycodes.push(*keycode);
                    } else if *state == ElementState::Released {
                        let index = game_loop.game.active_keycodes
                            .iter()
                            .position(|active_keycode| active_keycode == keycode)
                            .unwrap();

                        game_loop.game.active_keycodes.remove(index);
                    }
                },
                WindowEvent::CloseRequested | WindowEvent::KeyboardInput {
                    input:
                    KeyboardInput {
                        state: ElementState::Pressed,
                        virtual_keycode: Some(VirtualKeyCode::Escape),
                        ..
                    },
                    ..
                } => game_loop.exit(),
                WindowEvent::KeyboardInput {
                    input: KeyboardInput {
                        state: ElementState::Pressed,
                        virtual_keycode: Some(key_code),
                        ..
                    },
                    ..
                } => {
                    game_loop.game.manifold.change_on_keybinds(key_code);
                    game_loop.game.info = Info::default();
                },
                WindowEvent::Resized(physical_size) => {
                    game_loop.game.state.resize(*physical_size);
                },
                WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                    // new_inner_size is &&mut so we have to dereference it twice
                    game_loop.game.state.resize(**new_inner_size);
                },
                WindowEvent::CursorMoved {
                    position,
                    ..
                } => {
                    game_loop.game.mouse_state.position = Some(position.cast());

                    if let Some(delta) = game_loop.game.mouse_state.get_mouse_drag_delta() {
                        game_loop.game.info.rotate_around_y(delta.x / 500.0, -delta.y / 500.0);
                    }

                    game_loop.game.mouse_state.prev_position = game_loop.game.mouse_state.position;
                },
                WindowEvent::MouseInput {
                    button,
                    state,
                    ..
                } => {
                    let is_down = *state == ElementState::Pressed;

                    match *button {
                        MouseButton::Left => game_loop.game.mouse_state.left_down = is_down,
                        MouseButton::Right => game_loop.game.mouse_state.right_down = is_down,
                        _ => {}
                    };
                },
                _ => {}
            },
            _ => {}
        }
    );
    /*
    event_loop.run(move |event, _, control_flow| match event {
        Event::WindowEvent {
            ref event,
            window_id,
        }  if window_id == window.id() => {
            match event {
                WindowEvent::KeyboardInput {
                    input: KeyboardInput {
                        state,
                        virtual_keycode: Some(keycode),
                        ..
                    },
                    ..
                } if MOVEMENT_BINDINGS.contains(keycode) => {
                    if *state == ElementState::Pressed && !game.active_keycodes.contains(keycode) {
                        game.active_keycodes.push(*keycode);
                    } else if *state == ElementState::Released {
                        let index = game.active_keycodes
                            .iter()
                            .position(|active_keycode| active_keycode == keycode)
                            .unwrap();

                        game.active_keycodes.remove(index);
                    }
                },
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
                    game.state.resize(*physical_size);
                },
                WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                    // new_inner_size is &&mut so we have to dereference it twice
                    game.state.resize(**new_inner_size);
                },
                WindowEvent::CursorMoved {
                    position,
                    ..
                } => {
                    game.mouse_state.position = Some(position.cast());

                    if let Some(delta) = game.mouse_state.get_mouse_drag_delta() {
                        game.info.rotate_around_y(delta.x / 500.0, -delta.y / 500.0);
                    }

                    game.mouse_state.prev_position = game.mouse_state.position;
                },
                WindowEvent::MouseInput {
                    button,
                    state,
                    ..
                } => {
                    let is_down = *state == ElementState::Pressed;

                    match *button {
                        MouseButton::Left => game.mouse_state.left_down = is_down,
                        MouseButton::Right => game.mouse_state.right_down = is_down,
                        _ => {}
                    };
                },
                _ => {}
            }
        },
        Event::MainEventsCleared => {
            for active_keycode in game.active_keycodes.iter() {
                game.info.movement(*active_keycode, &game.manifold);
            }

            game.state.update_buffers(Some(&game.info), Some(&game.manifold));
            match game.state.render() {
                Ok(_) => {}
                // Reconfigure the surface if lost
                Err(wgpu::SurfaceError::Lost) => game.state.resize(game.state.size),
                // The system is out of memory, we should probably quit
                Err(wgpu::SurfaceError::OutOfMemory) => *control_flow = ControlFlow::Exit,
                // All other errors (Outdated, Timeout) should be resolved by the next frame
                Err(e) => eprintln!("{:?}", e),
            }

            control_flow.set_wait_until(
                Instant::now() + Duration::from_micros((1e6 / IDEAL_FPS) as u64)
            );
        },
        _ => {}
    });*/
}

struct Game<MANIFOLD: Manifold> {
    pub manifold: MANIFOLD,
    pub info: Info,
    pub geometry: Vec<Geometry>,
    pub state: State,
    pub mouse_state: MouseState,
    pub active_keycodes: Vec<VirtualKeyCode>,
    pub frame_start_time: Instant
}

impl<MANIFOLD: Manifold> Game<MANIFOLD> {
    async fn new(
        window: &Window,
        manifold: MANIFOLD,
        info: Info,
        geometry: Vec<Geometry>
    ) -> Self {
        let state = State::new(window, &info, &manifold).await;

        Self {
            manifold,
            info,
            geometry,
            state,
            mouse_state: MouseState::default(),
            active_keycodes: Vec::new(),
            frame_start_time: Instant::now()
        }
    }
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