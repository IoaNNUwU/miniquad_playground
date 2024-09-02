use macroquad::prelude::*;

pub use macroquad::miniquad::*;

#[macroquad::main("Raw miniquad")]
async fn main() {
    let atlas = load_texture("assets/atlas.png").await.unwrap();

    let raw_texture: TextureId = atlas.raw_miniquad_id();

    let (pipeline, bindings) = {
        let InternalGlContext { quad_context: ctx, .. } = unsafe { get_internal_gl() };

        make_pipelone_and_bindings(ctx)
    };

    let mut pos = vec3(1.0, 0.4, 5.0);

    loop {
        clear_background(LIGHTGRAY);

        if is_key_pressed(KeyCode::Y) {
            if !is_key_down(KeyCode::LeftShift) { pos.y += 1.0 } else { pos.y -= 1.0 }
        }
        if is_key_pressed(KeyCode::Z) {
            if !is_key_down(KeyCode::LeftShift) { pos.z += 1.0 } else { pos.z -= 1.0 }
        }
        if is_key_pressed(KeyCode::X) {
            if !is_key_down(KeyCode::LeftShift) { pos.x += 1.0 } else { pos.x -= 1.0 }
        }

        let camera = Camera3D {
            position: pos,
            target: vec3(0.0, 0.0, 0.0),
            up: Vec3::Y,
            ..Default::default()
        };

        set_camera(&camera);

        draw_cube_wires(vec3(0.0, 0.0, 0.0), vec3(1.0, 1.0, 1.0), macroquad::prelude::BLACK);

        draw_grid(20, 1.0, MAGENTA, DARKBLUE);

        {
            let mut gl = unsafe { get_internal_gl() };

            gl.flush();
            
            gl.quad_context.texture_set_filter(
                raw_texture,
                FilterMode::Nearest,
                MipmapFilterMode::None,
            );

            gl.quad_context.apply_pipeline(&pipeline);

            gl.quad_context.apply_bindings(&bindings);

            gl.quad_context.begin_default_pass(miniquad::PassAction::Nothing);

            let matrix = gl.quad_gl.get_projection_matrix();

            let color = vec3(1.0, 0.3, 0.5);

            gl.quad_context.apply_uniforms(UniformsSource::table(&Uniforms { mat: matrix, color }));

            gl.quad_context.draw(0, 3, 1);

            gl.quad_context.end_render_pass();
        }

        /* Back to screen space, render some text */ set_default_camera();

        next_frame().await
    }
}

#[repr(C)]
struct Uniforms {
    mat: Mat4,
    color: Vec3,
}

#[repr(C)]
struct MyVertex {
    pos: Vec3,
}

const WHITE: Vec3 = vec3(1.0, 1.0, 1.0);
const RED:   Vec3 = vec3(1.0, 0.0, 0.0);
const GREEN: Vec3 = vec3(0.0, 1.0, 0.0);
const BLUE:  Vec3 = vec3(0.0, 0.0, 1.0);
const BLACK: Vec3 = vec3(0.0, 0.0, 0.0);

fn myvertex(x: f32, y: f32, z: f32) -> MyVertex {
    MyVertex { pos: vec3(x, y, z) }
}

fn make_pipelone_and_bindings(ctx: &mut dyn RenderingBackend) -> (Pipeline, Bindings) {

    #[rustfmt::skip]
    let vertices = [
        myvertex(-0.5, -0.5, 0.0),
        myvertex( 0.5,  0.5, 0.0),
        myvertex( 0.5, -0.5, 0.0),
    ];

    let vertex_buffer = ctx.new_buffer(
        BufferType::VertexBuffer,
        BufferUsage::Immutable,
        BufferSource::slice(&vertices),
    );

    let indices = [
        0, 1, 2,
    ];

    let index_buffer = ctx.new_buffer(
        BufferType::IndexBuffer,
        BufferUsage::Immutable,
        BufferSource::slice(&indices[..]),
    );

    let bindings = Bindings {
        vertex_buffers: vec![vertex_buffer],
        index_buffer,
        images: vec![],
    };

    let shader = ctx
        .new_shader(
            miniquad::ShaderSource::Glsl {
                vertex: VERTEX,
                fragment: FRAGMENT,
            },
            ShaderMeta { 
                uniforms: UniformBlockLayout { 
                    uniforms: vec![
                        UniformDesc {
                            name: "camera_projection".to_string(),
                            uniform_type: UniformType::Mat4,
                            array_count: 1,
                        },
                        UniformDesc {
                            name: "triangle_color".to_string(),
                            uniform_type: UniformType::Float3,
                            array_count: 1,
                        },
                    ] 
                }, 
                images: vec![] 
            }
        )
        .unwrap();

    let pipeline = ctx.new_pipeline(
        &[BufferLayout {
            stride: (size_of::<f32>() * (3)) as i32,
            step_func: VertexStep::PerVertex,
            step_rate: 1,
        }],
        &[
            VertexAttribute::new("in_pos", VertexFormat::Float3),
        ],
        shader,
        PipelineParams::default(),
    );

    (pipeline, bindings)
}

pub const VERTEX: &str = r#"
#version 330 core

layout (location = 0) in vec3 in_pos;

uniform mat4 camera_projection;

void main()
{
    gl_Position = vec4(in_pos, 1.0) * camera_projection;
}
"#;

pub const FRAGMENT: &str = r#"
#version 330 core

uniform vec3 triangle_color;

out vec4 FragColor;

void main() 
{
    FragColor = vec4(triangle_color, 1.0f);
}
"#;