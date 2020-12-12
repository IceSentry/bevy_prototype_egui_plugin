use crate::backend::EguiBevyBackend;
use bevy::{
    math::vec2,
    prelude::*,
    render::{
        mesh::VertexAttribute,
        pipeline::{PipelineDescriptor, PrimitiveTopology},
        render_graph::{base, AssetRenderResourcesNode, RenderGraph},
        shader::{ShaderStage, ShaderStages},
    },
    render::{renderer::RenderResources, shader::ShaderDefs},
    window::WindowCloseRequested,
};
use egui::{app::RunMode, paint::Triangles};
use std::{sync::Arc, time::Instant};

struct EguiPluginState {
    start_time: Instant,
    frame_start: Instant,
    ctx: Arc<egui::Context>,
    raw_input: Option<egui::RawInput>,
    runner: EguiBevyBackend,
}

pub struct EguiContext {
    pub ui: Option<egui::Ui>,
}

#[derive(Default)]
pub struct WindowCloseRequestedReader {
    event_reader: EventReader<WindowCloseRequested>,
}

#[allow(dead_code)]
pub fn save_on_window_close_system(
    mut state: Local<WindowCloseRequestedReader>,
    window_close_requested_events: Res<Events<WindowCloseRequested>>,
) {
    if state
        .event_reader
        .iter(&window_close_requested_events)
        .next()
        .is_some()
    {
        // TODO
        // Save app state to a file

        // egui::app::set_value(
        //     &mut storage,
        //     WINDOW_KEY,
        //     &WindowSettings::from_display(&display),
        // );
        // egui::app::set_value(&mut storage, EGUI_MEMORY_KEY, &*ctx.memory());
        // app.on_exit(&mut storage);
        // storage.save();
    }
}

pub fn local_time_of_day() -> f64 {
    use chrono::Timelike;
    let time = chrono::Local::now().time();
    time.num_seconds_from_midnight() as f64 + 1e-9 * (time.nanosecond() as f64)
}

pub fn make_raw_input(window: &Window) -> egui::RawInput {
    egui::RawInput {
        screen_size: { egui::vec2(window.width as f32, window.height as f32) },
        ..Default::default()
    }
}

fn egui_check_windows(mut state: ResMut<EguiPluginState>, windows: Res<Windows>) {
    if state.raw_input.is_none() {
        state.raw_input = Some({
            let window = windows.get_primary().unwrap();
            make_raw_input(window)
        });
    }
}

fn egui_pre_update_system(mut state: ResMut<EguiPluginState>, mut ctx: ResMut<EguiContext>) {
    state.frame_start = Instant::now();
    if let Some(raw_input) = state.raw_input.clone().as_mut() {
        let time = state.start_time.elapsed().as_nanos() as f64 * 1e-9;
        raw_input.time = time;
        raw_input.seconds_since_midnight = Some(local_time_of_day());

        let ui = state.ctx.begin_frame(raw_input.clone());
        ctx.ui = Some(ui);
    }
}

fn convert_triangles_to_mesh(triangles: Triangles) -> Mesh {
    let mut positions = Vec::new();
    let mut uvs = Vec::new();
    // let mut colors = Vec::new();
    for vert in triangles.vertices.iter() {
        positions.push([vert.pos.x, vert.pos.y, 0.0]);
        uvs.push([vert.uv.0 as f32, vert.uv.1 as f32]);
        // colors.push(vert.color);
    }
    Mesh {
        primitive_topology: PrimitiveTopology::TriangleList,
        attributes: vec![
            VertexAttribute::position(positions),
            VertexAttribute::uv(uvs),
        ],
        indices: Some(triangles.indices),
    }
}

#[derive(Bundle)]
pub struct EguiMeshComponent {
    pub material: Handle<ColorMaterial>,
    pub render_pipeline: RenderPipelines,
    pub draw: Draw,
    pub mesh: Handle<Mesh>,
    pub transform: Transform,
    pub translation: Translation,
    pub rotation: Rotation,
    pub scale: Scale,
}

fn egui_post_update_system(
    mut commands: Commands,
    mut state: ResMut<EguiPluginState>,
    ctx: Res<EguiContext>,
) {
    let frame_time = (Instant::now() - state.frame_start).as_secs_f64() as f32;

    if let Some(raw_input) = state.raw_input.clone() {
        state.runner.frame_times.add(raw_input.time, frame_time);
    }

    if ctx.ui.is_none() {
        return;
    }

    let (_output, paint_jobs) = state.ctx.end_frame();

    // paint
    let _texture = state.ctx.texture();

    let mut meshes = Vec::new();
    for (_clip_rect, triangles) in paint_jobs {
        let mesh = convert_triangles_to_mesh(triangles);
        meshes.push(mesh);
    }

    commands.spawn(EguiMeshComponent {});

    // TODO
    // handle_output(output, &display, clipboard.as_mut());
}

#[derive(RenderResources, ShaderDefs, Default)]
struct EguiMaterial {
    pub a_pos: Vec2,
    pub a_color: Color,
    pub a_tc: Vec2,
}

const VERTEX_SHADER: &str = r#"
    #version 140
    uniform vec2 u_screen_size;
    uniform vec2 u_tex_size;
    in vec2 a_pos;
    in vec4 a_color;
    in vec2 a_tc;
    out vec4 v_color;
    out vec2 v_tc;
    void main() {
        gl_Position = vec4(
            2.0 * a_pos.x / u_screen_size.x - 1.0,
            1.0 - 2.0 * a_pos.y / u_screen_size.y,
            0.0,
            1.0);
        v_color = a_color / 255.0;
        v_tc = a_tc / u_tex_size;
    }
"#;

const FRAGMENT_SHADER: &str = r#"
    #version 140
    uniform sampler2D u_sampler;
    in vec4 v_color;
    in vec2 v_tc;
    out vec4 f_color;
    // glium expects linear output.
    vec3 linear_from_srgb(vec3 srgb) {
        bvec3 cutoff = lessThan(srgb, vec3(0.04045));
        vec3 higher = pow((srgb + vec3(0.055)) / vec3(1.055), vec3(2.4));
        vec3 lower = srgb / vec3(12.92);
        return mix(higher, lower, cutoff);
    }
    void main() {
        f_color = v_color;
        f_color.rgb = linear_from_srgb(f_color.rgb);
        f_color *= texture(u_sampler, v_tc).r;
    }
"#;

fn startup(
    mut _commands: Commands,
    mut pipelines: ResMut<Assets<PipelineDescriptor>>,
    mut shaders: ResMut<Assets<Shader>>,
    mut materials: ResMut<Assets<EguiMaterial>>,
    mut render_graph: ResMut<RenderGraph>,
) {
    // Create a new shader pipeline
    let _pipeline_handle = pipelines.add(PipelineDescriptor::default_config(ShaderStages {
        vertex: shaders.add(Shader::from_glsl(ShaderStage::Vertex, VERTEX_SHADER)),
        fragment: Some(shaders.add(Shader::from_glsl(ShaderStage::Fragment, FRAGMENT_SHADER))),
    }));

    // Add an AssetRenderResourcesNode to our Render Graph.
    // This will bind MyMaterial resources to our shader
    render_graph.add_system_node(
        "egui_material",
        AssetRenderResourcesNode::<EguiMaterial>::new(true),
    );

    // Add a Render Graph edge connecting our new "my_material" node to the main pass node.
    // This ensures "my_material" runs before the main pass
    render_graph
        .add_node_edge("egui_material", base::node::MAIN_PASS)
        .unwrap();

    let _egui_material = materials.add(EguiMaterial {
        a_pos: vec2(0.0, 0.0),
        a_color: Color::rgb(0.0, 0.0, 0.0),
        a_tc: vec2(0.0, 0.0),
    });
}

pub struct EguiPlugin;

impl Plugin for EguiPlugin {
    fn build(&self, app: &mut AppBuilder) {
        let start_time = Instant::now();

        let ctx = egui::Context::new();

        let state = EguiPluginState {
            start_time,
            frame_start: start_time,
            ctx,
            raw_input: None,
            runner: EguiBevyBackend::new(RunMode::Continuous), // TODO
        };
        let ui = EguiContext { ui: None };

        app.add_asset::<EguiMaterial>()
            .add_resource(ui)
            .add_resource(state)
            .add_startup_system(startup.system())
            .add_system_to_stage(stage::PRE_UPDATE, egui_pre_update_system.system())
            .add_system_to_stage(stage::POST_UPDATE, egui_post_update_system.system())
            .add_system(egui_check_windows.system());
    }
}
