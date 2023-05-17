#[macro_use]
extern crate glium;

#[path = "../tuto-07-teapot.rs"]
mod teapot;

#[allow(unused_variables)]
fn main()
{
    #[allow(unused_imports)]
    use glium::{glutin, Surface};

    // construct Glium Display and event loop
    let mut event_loop = glutin::event_loop::EventLoop::new();
    let wb = glutin::window::WindowBuilder::new();
    // depth buffer helps with determining if a pixel should be repainted with
    // newest fragment value
    let cb = glutin::ContextBuilder::new().with_depth_buffer(24); // 24 bits
    let display = glium::Display::new(wb, cb, &event_loop).unwrap();

    // load teapot
    let positions = glium::VertexBuffer::new(&display, &teapot::VERTICES).unwrap();
    let normals = glium::VertexBuffer::new(&display, &teapot::NORMALS).unwrap();
    let indices = glium::IndexBuffer::new(
        &display, glium::index::PrimitiveType::TrianglesList, &teapot::INDICES
    ).unwrap();

    // GLSL code for a vertex shader
    let vertex_shader_src = r#"
        #version 140

        in vec3 position;
        in vec3 normal;

        out vec3 v_normal;

        uniform mat4 perspective;
        uniform mat4 view;
        uniform mat4 model;

        void main()
        {
            mat4 modelview = view * model;
            v_normal = transpose(inverse(mat3(modelview))) * normal;
            gl_Position = perspective * modelview * vec4(position, 1.0);
        }
    "#;

    // GLSL code for a fragment shader
    let fragment_shader_src = r#"
        #version 140

        in vec3 v_normal;
        out vec4 color;
        uniform vec3 u_light;

        void main()
        {
            float brightness = dot(normalize(v_normal), normalize(u_light));
            vec3 dark_color = vec3(0.6, 0.0, 0.0);
            vec3 regular_color = vec3(1.0, 0.0, 0.0);

            // mix() interpolates between light and dark to give us a gradiant
            // across objects
            color = vec4(mix(dark_color, regular_color, brightness), 1.0);
        }
    "#;

    // send shaders to Glium
    let program = glium::Program::from_source(
        &display, vertex_shader_src,
        fragment_shader_src, None
    ).unwrap();

    event_loop.run(move |event, _, control_flow|
    {
        let next_frame_time = std::time::Instant::now()
            + std::time::Duration::from_nanos(16_666_667);
        *control_flow = glutin::event_loop::ControlFlow::WaitUntil(next_frame_time);

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

        let mut target = display.draw();
        // reset display color and depth buffer to blue and 1, respectively
        target.clear_color_and_depth((0.0, 0.0, 1.0, 1.0), 1.0);

        let view = view_matrix(
            &[2.0, -1.0, 1.0],
            &[-2.0, 1.0, 1.0],
            &[0.0, 1.0, 0.0]
        );
        let perspective = {
            let (width, height) = target.get_dimensions();
            let aspect_ratio = height as f32 / width as f32;

            let fov: f32 = 3.141592 / 3.0;
            let zfar = 1024.0;
            let znear = 0.1;

            let f = 1.0 / (fov / 2.0).tan();
            [
                [f * aspect_ratio, 0.0,                       0.0             , 0.0],
                [       0.0      ,  f ,                       0.0             , 0.0],
                [       0.0      , 0.0,  (zfar + znear) / (zfar - znear)      , 1.0],
                [       0.0      , 0.0, -(2.0 * zfar * znear) / (zfar - znear), 0.0]
            ]
        };

        let light = [-1.0, 0.4, 0.9f32];
        let model = [
            [0.01, 0.0, 0.0, 0.0],
            [0.0, 0.01, 0.0, 0.0],
            [0.0, 0.0, 0.01, 0.0],
            [0.0, 0.0, 2.0, 1.0f32]
        ];

        // needed to tell backend what to do with a list of 'possible'
        // operations; needed because depth test and buffer handling happen on
        // hardware
        let params = glium::DrawParameters {
            depth: glium::Depth {
                // indicates pixels should only be kept if depth is less than
                // existing value in depth buffer
                test: glium::draw_parameters::DepthTest::IfLess,

                // indicates depth value of pixels which pass the test should
                // be written to depth buffer
                write: true,
                .. Default::default()
            },
            // turns on backface culling operation
            backface_culling: glium::draw_parameters::BackfaceCullingMode
                ::CullClockwise,
            .. Default::default()
        };

        target.draw(
            (&positions, &normals),
            &indices,
            &program,
            &uniform! {
                model: model, view: view,
                perspective: perspective, u_light: light
            },
            &params
        ).unwrap();
        target.finish().unwrap();
    });
}

// position: position of the camera in the scene
// direction: direction camera is facing in scene coords
// up: represents the direction in scene coords of the top of the screen
fn view_matrix(
    position: &[f32; 3], direction: &[f32; 3], up: &[f32; 3]
) -> [[f32; 4]; 4]
{
    let f = {
        let f = direction;
        let len = f[0] * f[0] + f[1] * f[1] + f[2] * f[2];
        let len = len.sqrt();
        [f[0] / len, f[1] / len, f[2] / len]
    };

    let s = [
        up[1] * f[2] - up[2] * f[1],
        up[2] * f[0] - up[0] * f[2],
        up[0] * f[1] - up[1] * f[0]
    ];

    let s_norm = {
        let len = s[0] * s[0] + s[1] * s[1] + s[2] * s[2];
        let len = len.sqrt();
        [s[0] / len, s[1] / len, s[2] / len]
    };

    let u = [
        f[1] * s_norm[2] - f[2] * s_norm[1],
        f[2] * s_norm[0] - f[0] * s_norm[2],
        f[0] * s_norm[1] - f[1] * s_norm[0]
    ];

    let p = [
        -position[0] * s_norm[0] - position[1] * s_norm[1] - position[2] * s_norm[2],
        -position[0] * u[0] - position[1] * u[1] - position[2] * u[2],
        -position[0] * f[0] - position[1] * f[1] - position[2] * f[2]
    ];
    [
        [s_norm[0], u[0], f[0], 0.0],
        [s_norm[1], u[1], f[1], 0.0],
        [s_norm[2], u[2], f[2], 0.0],
        [p[0], p[1], p[2], 1.0]
    ]
}

