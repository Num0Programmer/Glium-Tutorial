#[macro_use]
extern crate glium;
extern crate image;

#[allow(unused_variables)]
fn main()
{
    // trait imports
    use std::io::Cursor;
    use glium::{glutin, Surface};

    // representation of a point in space
    #[derive(Copy, Clone)]
    struct Vertex
    {
        position: [f32; 2],
        tex_coords: [f32; 2]
    }
    implement_vertex!(Vertex, position, tex_coords);

    // construct Glium Display
    let mut event_loop = glutin::event_loop::EventLoop::new();
    let wb = glutin::window::WindowBuilder::new();
    let cb = glutin::ContextBuilder::new();
    let display = glium::Display::new(wb, cb, &event_loop).unwrap();

    let indices = glium::index::NoIndices(
        glium::index::PrimitiveType::TrianglesList
    );

    // setup for image loading
    let image = image::load(
        Cursor::new(&include_bytes!("../glium_tutorial_creeper.png")),
        image::ImageFormat::Png
    ).unwrap().to_rgba8();
    let image_dimensions = image.dimensions();
    let image = glium::texture::RawImage2d::from_raw_rgba_reversed(
        &image.into_raw(), image_dimensions
    );
    let texture = glium::texture::SrgbTexture2d::new(&display, image).unwrap();

    // GLSL code for a vertex shader
    let vertex_shader_src = r#"
        #version 140

        in vec2 position;
        out vec2 my_attr; // our new attribute

        uniform mat4 matrix;

        void main()
        {
            my_attr = position; // we need to set the value of each `out` variable
            gl_Position = matrix * vec4(position, 0.0, 1.0);
        }
    "#;

    // GLSL code for a fragment shader
    let fragment_shader_src = r#"
        #version 140

        in vec2 my_attr;
        out vec4 color;

        void main()
        {
            color = vec4(my_attr, 0.0, 1.0); // we build a vec4 from a vec2 and two floats
        }
    "#;

    // send shaders to Glium
    let program = glium::Program::from_source(
        &display, vertex_shader_src,
        fragment_shader_src, None
    ).unwrap();

    let vertex1 = Vertex { position: [-0.5, -0.5], tex_coords: [0.0, 0.0] };
    let vertex2 = Vertex { position: [ 0.0,  0.5], tex_coords: [0.0, 1.0] };
    let vertex3 = Vertex { position: [ 0.5, -0.25], tex_coords: [1.0, 0.0] };
    let shape = vec![vertex1, vertex2, vertex3];

    let vertex_buffer = glium::VertexBuffer::new(&display, &shape).unwrap();
    let mut t: f32 = -0.5;
    event_loop.run(move |event, _, control_flow| {
        match event
        {
            glutin::event::Event::WindowEvent { event, .. } => match event
            {
                glutin::event::WindowEvent::CloseRequested =>
                {
                    *control_flow = glutin::event_loop::ControlFlow::Exit;
                    return
                },
                _ => return
            },
            glutin::event::Event::NewEvents(cause) => match cause
            {
                glutin::event::StartCause::ResumeTimeReached { .. } => (),
                glutin::event::StartCause::Init => (),
                _ => return
            },
            _ => return
        }

        let next_frame_time = std::time::Instant::now()
            + std::time::Duration::from_nanos(16_666_667);
        *control_flow = glutin::event_loop::ControlFlow::WaitUntil(
            next_frame_time
        );

        // we update `t`
        t += 0.0002;
        if t > 0.5
        {
            t = -0.5;
        }

        let uniforms = uniform! {
            matrix: [
                [ t.cos(), t.sin(), 0.0, 0.0],
                [-t.sin(), t.cos(), 0.0, 0.0],
                [0.0, 0.0, 1.0, 0.0],
                [0.0, 0.0, 0.0, 1.0f32]
            ]
        };
        let mut target = display.draw();
        target.clear_color(0.0, 0.0, 1.0, 1.0);
        target.draw(
            &vertex_buffer, &indices, &program,
            &uniforms, &Default::default()
        ).unwrap();
        target.finish().unwrap();
    });
}

