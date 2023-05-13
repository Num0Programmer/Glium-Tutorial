#[macro_use]
extern crate glium;

#[allow(unused_variables)]
fn main()
{
    // trait imports
    use glium::{glutin, Surface};

    // construct Glium Display
    let mut event_loop = glutin::event_loop::EventLoop::new();
    let wb = glutin::window::WindowBuilder::new();
    let cb = glutin::ContextBuilder::new();
    let display = glium::Display::new(wb, cb, &event_loop).unwrap();

    // run event loop
    event_loop.run(move |ev, _, control_flow|
    {
        let mut target = display.draw();
        target.clear_color(0.0, 0.0, 1.0, 1.0);
        target.finish().unwrap(); // destroy frame and copy surface to screen

        let next_frame_time = std::time::Instant::now()
            + std::time::Duration::from_nanos(16_666_67);
        *control_flow = glutin::event_loop::ControlFlow
            ::WaitUntil(next_frame_time);
        match ev
        {
            glutin::event::Event::WindowEvent { event, .. }
                => match event
                {
                    glutin::event::WindowEvent::CloseRequested
                        =>
                        {
                            *control_flow = glutin::event_loop
                                ::ControlFlow::Exit;
                            return
                        },
                        _ => return,
                },
                _ => ()
        }
    });
}

