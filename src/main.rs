extern crate euclid;
extern crate glutin;
extern crate gl;
extern crate tracer;

use euclid::{Size2D, TypedPoint3D};
use tracer::WorldPosition;

fn main() {
    let window = glutin::Window::new().unwrap();

    unsafe { window.make_current() };

    unsafe {
        gl::load_with(|symbol| window.get_proc_address(symbol) as *const _);

        gl::ClearColor(0.0, 1.0, 0.0, 1.0);
    }

    let mut scene = tracer::Scene::new(Size2D::new(800, 800),
                                       WorldPosition::new(0.0, 0.0, -100.),
                                       WorldPosition::new(0.0, 0.0, -100.));
    let sphere = tracer::ObjectKind::Sphere { radius: 10. };
    let sphere = tracer::Object::new(sphere, WorldPosition::new(0., 0., 0.));
    scene.add_object(sphere);

    let sphere = tracer::ObjectKind::Sphere { radius: 12. };
    let sphere = tracer::Object::new(sphere, WorldPosition::new(0., 5., 0.));
    scene.add_object(sphere);

    for event in window.wait_events() {
        unsafe { gl::Clear(gl::COLOR_BUFFER_BIT) };
        window.swap_buffers();

        match event {
            glutin::Event::Closed => break,
            _ => ()
        }
    }
}
