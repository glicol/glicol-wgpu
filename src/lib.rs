use std::{
    cell::RefCell,
    rc::Rc,
    time::{Duration, Instant},
};
use winit::{
    event::*,
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

// mod vertex;
// use vertex::*;

mod renderer;
use renderer::*;
// mod font;
// pub use font::*;

// mod behaviour;
// use behaviour::*;
// #[cfg(not(target_arch = "wasm32"))]
// mod audio;
// #[cfg(not(target_arch = "wasm32"))]
// pub use audio::*;

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
    let window_clone = window_ref.clone();

    #[cfg(target_arch = "wasm32")]
    {
        // resize_window(&window);
        resize_window(&window_clone);
        use winit::platform::web::WindowExtWebSys;
        web_sys::window()
            .and_then(|win| win.document())
            .and_then(|doc| {
                let dst = doc.get_element_by_id("canvas-div")?;
                // if let Some(canvas) = window.canvas() {
                let canvas = window_clone.borrow().canvas();
                let canvas_element = web_sys::Element::from(canvas);
                dst.append_child(&canvas_element).ok()?;

                let w = web_sys::window().expect("should have a Window");
                // let canvas = window.canvas();
                // resize_canvas(&canvas);

                let closure = Closure::wrap(Box::new(move || {
                    // log::warn!("resize_canvas");
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

    event_loop.run(move |event, _, control_flow| match event {
        Event::WindowEvent {
            ref event,
            window_id,
        } if window_id == renderer.window().borrow().id() => {
            if !renderer.input(event) {
                //modifiers
                match event {
                    WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
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
            renderer.update();
            renderer.render().unwrap();

            // #[cfg(target_arch = "wasm32")]
            // {
            //     resize_window(renderer.window());
            // }
            // let new_size = renderer.window().inner_size();
            // renderer.resize(new_size);
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
// fn resize_canvas(canvas: &web_sys::HtmlCanvasElement) {
//     let window = web_sys::window().expect("should have a Window");
//     let width = window.inner_width().unwrap().as_f64().unwrap() as u32;
//     let height = window.inner_height().unwrap().as_f64().unwrap() as u32;

//     // 调整 canvas 的属性宽高
//     canvas.set_width(width);
//     canvas.set_height(height);

//     // 调整 canvas 的样式宽高
//     let style = canvas.style();
//     style
//         .set_property("width", &format!("{}px", width))
//         .unwrap();
//     style
//         .set_property("height", &format!("{}px", height))
//         .unwrap();
// }
