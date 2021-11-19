use winit::window::WindowBuilder;
use winit::event_loop::{ControlFlow, EventLoop};
use winit::event::WindowEvent;
use winit::event::VirtualKeyCode;
use winit::event::KeyboardInput;
use winit::event::Event;
use winit::event::ElementState;

mod engine;
mod camera;

fn main() {
    env_logger::init();


    let event_loop = EventLoop::new();
    let window = WindowBuilder::new().build(&event_loop).unwrap();
    
    let mut engine = pollster::block_on(engine::Engine::new(&window));
    let mut last_render_time = std::time::Instant::now();
    event_loop.run(move |event, _, control_flow| {

        *control_flow = ControlFlow::Poll;
        match event {

            Event::DeviceEvent {
                ref event,
                ..
            } => {
                engine.input(event);
            }
            // window-specific event
            Event::WindowEvent {
                ref event,
                window_id
            } if window_id == window.id() => {
                match event {

                    WindowEvent::CloseRequested | WindowEvent::KeyboardInput {
                        input: KeyboardInput {
                            state: ElementState::Pressed,
                            virtual_keycode: Some(VirtualKeyCode::Escape),
                            ..
                        },
                        ..
                    } => *control_flow = ControlFlow::Exit,
                    WindowEvent::Resized(physical_size) => {
                        engine.resize(*physical_size);
                    },
                    WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                        engine.resize(**new_inner_size)
                    },
                    _ => {}
                }
            },
            Event::RedrawRequested(_) => {
                let now = std::time::Instant::now();
                let dt = now - last_render_time;
                last_render_time = now;
                engine.update(dt);
                match engine.render() {
                    Ok(_) => {},
                    Err(wgpu::SurfaceError::Lost) => engine.resize(engine.get_size()),
                    Err(wgpu::SurfaceError::OutOfMemory) => *control_flow = ControlFlow::Exit,
                    Err(e) => eprintln!("{:?}", e)
                }
            },
            Event::MainEventsCleared => {
                window.request_redraw();
            }
            _ => {}
        }
    })
}
