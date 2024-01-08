use std::{cell::RefCell, rc::Rc};
use winit::{
    event::*,
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

#[cfg(target_arch = "wasm32")]
use js_sys::{Float32Array, Function};

mod vertex;
pub use vertex::*;

mod utils;
pub use utils::*;

mod renderer;
pub use renderer::*;

mod input;
use input::*;

#[cfg(not(target_arch = "wasm32"))]
mod audio;
#[cfg(not(target_arch = "wasm32"))]
pub use audio::*;

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

#[cfg_attr(target_arch = "wasm32", wasm_bindgen(start))]
pub async fn run() {
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_title("glicol-wgpu")
        .build(&event_loop)
        .unwrap();
    let window_ref = Rc::new(RefCell::new(window));

    #[cfg(target_arch = "wasm32")]
    {
        let window_clone = window_ref.clone();
        resize_window(&window_clone);
        use winit::platform::web::WindowExtWebSys;
        web_sys::window()
            .and_then(|win| win.document())
            .and_then(|doc| {
                let dst = doc.get_element_by_id("canvas-div")?;
                let canvas = window_clone.borrow().canvas();
                let canvas_element = web_sys::Element::from(canvas);
                dst.append_child(&canvas_element).ok()?;

                let w = web_sys::window().expect("should have a Window");
                let closure = Closure::wrap(Box::new(move || {
                    resize_window(&window_clone);
                }) as Box<dyn FnMut()>);

                let c =
                    w.add_event_listener_with_callback("resize", closure.as_ref().unchecked_ref());
                closure.forget();

                Some(())
            })
            .expect("Couldn't append canvas to document body.");
    }

    let mut renderer = Renderer::new(window_ref).await;

    #[cfg(target_arch = "wasm32")]
    {
        //     use glicol::Engine;
        //     let mut engine = Engine::<128>::new();
        //     let mut engine_ref = Rc::new(RefCell::new(engine));
        //     let mut engine_ref2 = engine_ref.clone();
        //     renderer.add_audio_engine(engine_ref2);
        //     // engine_ref
        //     //     .borrow_mut()
        //     //     .update_with_code(r#"o: speed 16.0 >> seq 60 >> hh 0.03"#);
        //     let window = web_sys::window().expect("no global `window` exists");
        //     let this = JsValue::null();
        //     let start = window
        //         .get("audioStart")
        //         .unwrap()
        //         .dyn_into::<Function>()
        //         .unwrap();
        //     start.call0(&this).unwrap();
        //     let sab = window
        //         .get("dataSAB")
        //         .unwrap()
        //         .dyn_into::<js_sys::SharedArrayBuffer>()
        //         .unwrap();
        //     let buf = Float32Array::new(&sab);
        //     let write_ptr = JsValue::from(
        //         window
        //             .get("writePtr")
        //             .unwrap()
        //             .dyn_into::<js_sys::Uint32Array>()
        //             .unwrap(),
        //     );
        //     let read_ptr = JsValue::from(
        //         window
        //             .get("readPtr")
        //             .unwrap()
        //             .dyn_into::<js_sys::Uint32Array>()
        //             .unwrap(),
        //     );

        //     let f = Rc::new(RefCell::new(None));
        //     let g = f.clone();
        //     *g.borrow_mut() = Some(Closure::wrap(Box::new(move || {
        //         loop {
        //             let wr = js_sys::Atomics::load(&write_ptr, 0).unwrap();
        //             let rd = js_sys::Atomics::load(&read_ptr, 0).unwrap();
        //             let available_read = (wr + 2048 - rd) % 2048;
        //             let available_write = 2048 - available_read;
        //             if available_write <= 128 {
        //                 break;
        //             }
        //             let mut engine_borrow = engine_ref.borrow_mut();
        //             let (block, _console_info) = engine_borrow.next_block(vec![]);

        //             for i in 0..128 {
        //                 let wr = js_sys::Atomics::load(&write_ptr, 0).unwrap();
        //                 let val = block[0][i];
        //                 buf.set_index(wr as u32, val);
        //                 js_sys::Atomics::store(&write_ptr, 0, (wr + 1) % (2048)).unwrap();
        //             }
        //         }
        //         request_animation_frame(f.borrow().as_ref().unwrap());
        //     }) as Box<dyn FnMut()>));

        //     request_animation_frame(g.borrow().as_ref().unwrap());
    }
    // #[cfg(not(target_arch = "wasm32"))]
    // let mut modifiers = ModifiersState::default();
    event_loop.run(move |event, _, control_flow| match event {
        Event::WindowEvent {
            ref event,
            window_id,
        } if window_id == renderer.window().borrow().id() => {
            if !renderer.input(event) {
                //modifiers
                match event {
                    WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                    // not working on web, so we manually manage modifiers
                    // #[cfg(not(target_arch = "wasm32"))]
                    // WindowEvent::ModifiersChanged(new_modifiers) => {
                    //     modifiers = *new_modifiers;
                    // }
                    WindowEvent::Resized(physical_size) => {
                        renderer.resize(*physical_size);
                    }
                    WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                        // new_inner_size is &mut so w have to dereference it twice
                        renderer.resize(**new_inner_size);
                    }
                    _ => {}
                }
            }
        }
        Event::MainEventsCleared => {
            renderer.window().borrow().request_redraw();
        }
        Event::RedrawRequested(window_id) if window_id == renderer.window().borrow().id() => {
            // renderer.update();
            renderer.render().unwrap();
        }
        _ => {}
    });
}

#[cfg(target_arch = "wasm32")]
fn resize_window(window: &Rc<RefCell<winit::window::Window>>) {
    let w = web_sys::window().expect("should have a Window");
    let width =
        (w.inner_width().unwrap().as_f64().unwrap() * window.borrow().scale_factor()) as u32;
    let height =
        (w.inner_height().unwrap().as_f64().unwrap() * window.borrow().scale_factor()) as u32;

    // Resize the winit window
    use winit::dpi::PhysicalSize;
    window
        .borrow()
        .set_inner_size(PhysicalSize::new(width, height));
}

// #[cfg(target_arch = "wasm32")]
// fn request_animation_frame(f: &Closure<dyn FnMut()>) {
//     web_sys::window()
//         .expect("REASON")
//         .request_animation_frame(f.as_ref().unchecked_ref())
//         .expect("should register `requestAnimationFrame` OK");
// }
