use std::os::raw::c_int;
use std::os::raw::c_void;

use nuklear_sys::*;
struct Vertex {
    position: [f32; 2],
    uv: [f32; 2],
    color: u32,
}

const MAX_INDEX_BUFFER_SIZE: usize = 128 * 1024;
const MAX_VERTEX_BUFFER_SIZE: usize = 512 * 1024;

macro_rules! size_of {
    ($T:ty) => {
        std::mem::size_of::<$T>()
    };
}
macro_rules! offset_of {
    ($T:ty,$field:tt) => {{
        let elem: $T = std::mem::zeroed();
        &elem.$field as *const _ as usize - &elem as *const _ as usize
    }};
}
macro_rules! align_of {
    ($T:ty) => {
        std::mem::align_of::<$T>()
    };
}

pub enum Key {
    None,
    Shift,
    Ctrl,
    Del,
    Enter,
    Tab,
    Backspace,
    Copy,
    Cut,
    Paste,
    Up,
    Down,
    Left,
    Right,
}

pub enum Button {
    Left,
    Middle,
    Right,
    Double,
}

pub enum State {
    Press,
    Release,
}

fn convert_virtual_key(key: Key) -> i32 {
    match key {
        Key::None=>nk_keys_NK_KEY_MAX,
        Key::Shift => nk_keys_NK_KEY_SHIFT,
        Key::Ctrl => nk_keys_NK_KEY_CTRL,
        Key::Del => nk_keys_NK_KEY_DEL,
        Key::Enter => nk_keys_NK_KEY_ENTER,
        Key::Tab => nk_keys_NK_KEY_TAB,
        Key::Backspace => nk_keys_NK_KEY_BACKSPACE,
        Key::Copy => nk_keys_NK_KEY_COPY,
        Key::Cut => nk_keys_NK_KEY_CUT,
        Key::Paste => nk_keys_NK_KEY_PASTE,
        Key::Up => nk_keys_NK_KEY_UP,
        Key::Down => nk_keys_NK_KEY_DOWN,
        Key::Left => nk_keys_NK_KEY_LEFT,
        Key::Right => nk_keys_NK_KEY_RIGHT,
    }
}

fn convert_button(button: Button) -> i32 {
    match button {
        Button::Left => nk_buttons_NK_BUTTON_LEFT,
        Button::Right => nk_buttons_NK_BUTTON_MIDDLE,
        Button::Middle => nk_buttons_NK_BUTTON_RIGHT,
        Button::Double => nk_buttons_NK_BUTTON_DOUBLE,
        _ => nk_buttons_NK_BUTTON_MAX,
    }
}
pub struct Context {
    pub context: nk_context,
    buffer: nk_buffer,
    atlas: nk_font_atlas,

    bind_group_layout: wgpu::BindGroupLayout,
    pipeline_layout: wgpu::PipelineLayout,
    vs_module: wgpu::ShaderModule,
    fs_module: wgpu::ShaderModule,
    pipeline: wgpu::RenderPipeline,

    null_texture: nk_draw_null_texture,
    texture: wgpu::Texture,
    texture_view: wgpu::TextureView,
    sampler: wgpu::Sampler,
    index_buffer: wgpu::Buffer,
    vertex_buffer: wgpu::Buffer,
    uniform_buffer: wgpu::Buffer,
    bind_group: wgpu::BindGroup,
    indices: Vec<u8>,
    vertices: Vec<u8>,
    cursor_x: i32,
    cursor_y: i32,
}
impl Context {
    pub unsafe fn input_begin(&mut self) {
        nk_input_begin(&mut self.context);
    }
    pub unsafe fn input_end(&mut self) {
        nk_input_end(&mut self.context);
    }
    pub unsafe fn input_char(&mut self, c: char) {
        nk_input_char(&mut self.context, c as i8);
    }
    pub unsafe fn input_key(&mut self, key: Key, state: State) {
        match key {
            Key::None => {}
            _ => {
                let s = match state {
                    State::Press => 1,
                    State::Release => 0,
                };
                nk_input_key(&mut self.context, convert_virtual_key(key), s);
            }
        };
    }
    pub unsafe fn input_motion(&mut self, cursor_x: i32, cursor_y: i32) {
        self.cursor_x = cursor_x;
        self.cursor_y = cursor_y;
        nk_input_motion(&mut self.context, cursor_x, cursor_y);
    }
    pub unsafe fn input_button(&mut self, button: Button, state: State) {
        let s = match state {
            State::Press => 1,
            State::Release => 0,
        };
        nk_input_button(
            &mut self.context,
            convert_button(button),
            self.cursor_x,
            self.cursor_y,
            s,
        );
    }
    pub unsafe fn input_scroll(&mut self, scroll_x: f32, scroll_y: f32) {
        nk_input_scroll(&mut self.context, nk_vec2(scroll_x, scroll_y));
    }
    pub unsafe fn update(&mut self, queue: &wgpu::Queue, screen_width: f32, screen_height: f32) {
        let mut ibuf: nk_buffer = std::mem::zeroed();
        let mut vbuf: nk_buffer = std::mem::zeroed();
        nk_buffer_init_fixed(
            &mut ibuf,
            self.indices.as_mut_ptr() as *mut c_void,
            MAX_INDEX_BUFFER_SIZE as usize,
        );
        nk_buffer_init_fixed(
            &mut vbuf,
            self.vertices.as_mut_ptr() as *mut c_void,
            MAX_VERTEX_BUFFER_SIZE as usize,
        );

        let mut config: nk_convert_config = std::mem::zeroed();
        let vertex_layout = [
            nk_draw_vertex_layout_element {
                attribute: nk_draw_vertex_layout_attribute_NK_VERTEX_POSITION,
                format: nk_draw_vertex_layout_format_NK_FORMAT_FLOAT,
                offset: offset_of!(Vertex, position),
            },
            nk_draw_vertex_layout_element {
                attribute: nk_draw_vertex_layout_attribute_NK_VERTEX_TEXCOORD,
                format: nk_draw_vertex_layout_format_NK_FORMAT_FLOAT,
                offset: offset_of!(Vertex, uv),
            },
            nk_draw_vertex_layout_element {
                attribute: nk_draw_vertex_layout_attribute_NK_VERTEX_COLOR,
                format: nk_draw_vertex_layout_format_NK_FORMAT_R8G8B8A8,
                offset: offset_of!(Vertex, color),
            },
            nk_draw_vertex_layout_element {
                attribute: nk_draw_vertex_layout_attribute_NK_VERTEX_ATTRIBUTE_COUNT,
                format: nk_draw_vertex_layout_format_NK_FORMAT_COUNT,
                offset: 0,
            },
        ];
        config.vertex_layout = vertex_layout.as_ptr();
        config.vertex_size = size_of!(Vertex);
        config.vertex_alignment = align_of!(Vertex);
        config.global_alpha = 1.0;
        config.shape_AA = nk_anti_aliasing_NK_ANTI_ALIASING_OFF;
        config.line_AA = nk_anti_aliasing_NK_ANTI_ALIASING_OFF;
        config.circle_segment_count = 22;
        config.curve_segment_count = 22;
        config.arc_segment_count = 22;
        config.null = self.null_texture;

        nk_convert(
            &mut self.context,
            &mut self.buffer,
            &mut vbuf,
            &mut ibuf,
            &mut config,
        );
        queue.write_buffer(&self.index_buffer, 0, self.indices.as_slice());
        queue.write_buffer(&self.vertex_buffer, 0, self.vertices.as_slice());

        let mut projection: [f32; 16] = [
            2.0, 0.0, 0.0, 0.0, 0.0, -2.0, 0.0, 0.0, 0.0, 0.0, -1.0, 0.0, -1.0, 1.0, 0.0, 1.0,
        ];
        projection[0] = projection[0] / screen_width;
        projection[5] = projection[5] / screen_height;
        queue.write_buffer(&self.uniform_buffer, 0, bytemuck::cast_slice(&projection));
    }
    pub unsafe fn new(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        swap_chain_texture_format: wgpu::TextureFormat,
    ) -> Self {
        let mut context: nk_context = std::mem::zeroed();
        let mut buffer: nk_buffer = std::mem::zeroed();
        let mut atlas: nk_font_atlas = std::mem::zeroed();
        nk_init_default(&mut context, std::ptr::null_mut());
        nk_buffer_init_default(&mut buffer);
        nk_font_atlas_init_default(&mut atlas);
        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: None,
            bindings: &[
                wgpu::BindGroupLayoutEntry::new(
                    0,
                    wgpu::ShaderStage::VERTEX,
                    wgpu::BindingType::UniformBuffer {
                        dynamic: false,
                        min_binding_size: wgpu::BufferSize::new(4 * 16),
                    },
                ),
                wgpu::BindGroupLayoutEntry::new(
                    1,
                    wgpu::ShaderStage::FRAGMENT,
                    wgpu::BindingType::SampledTexture {
                        multisampled: false,
                        component_type: wgpu::TextureComponentType::Float,
                        dimension: wgpu::TextureViewDimension::D2,
                    },
                ),
                wgpu::BindGroupLayoutEntry::new(
                    2,
                    wgpu::ShaderStage::FRAGMENT,
                    wgpu::BindingType::Sampler { comparison: false },
                ),
            ],
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            bind_group_layouts: &[&bind_group_layout],
        });

        let vs_module = device.create_shader_module(wgpu::include_spirv!("nuklear.vert.spv"));
        let fs_module = device.create_shader_module(wgpu::include_spirv!("nuklear.frag.spv"));

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            layout: &pipeline_layout,
            vertex_stage: wgpu::ProgrammableStageDescriptor {
                module: &vs_module,
                entry_point: "main",
            },
            fragment_stage: Some(wgpu::ProgrammableStageDescriptor {
                module: &fs_module,
                entry_point: "main",
            }),
            rasterization_state: Some(wgpu::RasterizationStateDescriptor {
                front_face: wgpu::FrontFace::Cw,
                cull_mode: wgpu::CullMode::Back,
                depth_bias: 0,
                depth_bias_slope_scale: 0.0,
                depth_bias_clamp: 0.0,
            }),
            primitive_topology: wgpu::PrimitiveTopology::TriangleList,
            color_states: &[wgpu::ColorStateDescriptor {
                format: swap_chain_texture_format,
                color_blend: wgpu::BlendDescriptor {
                    src_factor: wgpu::BlendFactor::SrcAlpha,
                    dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                    operation: wgpu::BlendOperation::Add,
                },
                alpha_blend: wgpu::BlendDescriptor {
                    src_factor: wgpu::BlendFactor::One,
                    dst_factor: wgpu::BlendFactor::Zero,
                    operation: wgpu::BlendOperation::Add,
                },
                write_mask: wgpu::ColorWrite::ALL,
            }],
            depth_stencil_state: None,
            vertex_state: wgpu::VertexStateDescriptor {
                index_format: wgpu::IndexFormat::Uint16,
                vertex_buffers: &[wgpu::VertexBufferDescriptor {
                    stride: size_of!(Vertex) as wgpu::BufferAddress,
                    step_mode: wgpu::InputStepMode::Vertex,
                    attributes: &[
                        wgpu::VertexAttributeDescriptor {
                            format: wgpu::VertexFormat::Float2,
                            offset: offset_of!(Vertex, position) as u64,
                            shader_location: 0,
                        },
                        wgpu::VertexAttributeDescriptor {
                            format: wgpu::VertexFormat::Float2,
                            offset: offset_of!(Vertex, uv) as u64,
                            shader_location: 1,
                        },
                        wgpu::VertexAttributeDescriptor {
                            format: wgpu::VertexFormat::Uint,
                            offset: offset_of!(Vertex, color) as u64,
                            shader_location: 2,
                        },
                    ],
                }],
            },
            sample_count: 1,
            sample_mask: !0,
            alpha_to_coverage_enabled: false,
        });
        let mut null_texture: nk_draw_null_texture = std::mem::zeroed();
        nk_font_atlas_begin(&mut atlas);
        let mut width: c_int = 0;
        let mut height: c_int = 0;
        let image = nk_font_atlas_bake(
            &mut atlas,
            &mut width,
            &mut height,
            nk_font_atlas_format_NK_FONT_ATLAS_RGBA32,
        ) as *const u8;
        let image_data = std::slice::from_raw_parts(image, (width * height * 4) as usize);
        let texture_extent = wgpu::Extent3d {
            width: width as u32,
            height: height as u32,
            depth: 1,
        };
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: None,
            size: texture_extent,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8Unorm,
            usage: wgpu::TextureUsage::SAMPLED | wgpu::TextureUsage::COPY_DST,
        });
        let texture_view = texture.create_default_view();
        queue.write_texture(
            wgpu::TextureCopyView {
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
            },
            image_data,
            wgpu::TextureDataLayout {
                offset: 0,
                bytes_per_row: (width * 4) as u32,
                rows_per_image: 0,
            },
            texture_extent,
        );
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::Repeat,
            address_mode_v: wgpu::AddressMode::Repeat,
            address_mode_w: wgpu::AddressMode::Repeat,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Linear,
            ..Default::default()
        });
        nk_font_atlas_end(&mut atlas, nk_handle_id(0), &mut null_texture);
        if !atlas.default_font.is_null() {
            nk_style_set_font(&mut context, &mut (*atlas.default_font).handle)
        }
        let index_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: None,
            size: MAX_VERTEX_BUFFER_SIZE as u64,
            usage: wgpu::BufferUsage::INDEX | wgpu::BufferUsage::COPY_DST,
            mapped_at_creation: false,
        });
        let vertex_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: None,
            size: MAX_VERTEX_BUFFER_SIZE as u64,
            usage: wgpu::BufferUsage::VERTEX | wgpu::BufferUsage::COPY_DST,
            mapped_at_creation: false,
        });
        let uniform_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: None,
            size: 4 * 16,
            usage: wgpu::BufferUsage::UNIFORM | wgpu::BufferUsage::COPY_DST,
            mapped_at_creation: false,
        });
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &bind_group_layout,
            bindings: &[
                wgpu::Binding {
                    binding: 0,
                    resource: wgpu::BindingResource::Buffer(uniform_buffer.slice(..)),
                },
                wgpu::Binding {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(&texture_view),
                },
                wgpu::Binding {
                    binding: 2,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
            ],
            label: None,
        });

        let mut indices: Vec<u8> = Vec::new();
        indices.resize(MAX_INDEX_BUFFER_SIZE, 0);
        let mut vertices: Vec<u8> = Vec::new();
        vertices.resize(MAX_VERTEX_BUFFER_SIZE, 0);
        Self {
            context,
            buffer,
            atlas,
            bind_group_layout,
            pipeline_layout,
            vs_module,
            fs_module,
            pipeline,
            null_texture,
            texture,
            texture_view,
            sampler,
            index_buffer,
            vertex_buffer,
            uniform_buffer,
            bind_group,
            indices,
            vertices,
            cursor_x: 0,
            cursor_y: 0,
        }
    }
}
impl Drop for Context {
    fn drop(&mut self) {
        unsafe {
            nk_font_atlas_clear(&mut self.atlas);
            nk_buffer_free(&mut self.buffer);
            nk_free(&mut self.context);
        }
    }
}

pub trait Renderer<'a> {
    fn draw_gui(&mut self, context: &'a mut Context, screen_width: f32, screen_height: f32);
}

impl<'a> Renderer<'a> for wgpu::RenderPass<'a> {
    fn draw_gui(&mut self, context: &'a mut Context, screen_width: f32, screen_height: f32) {
        self.set_pipeline(&context.pipeline);
        self.set_bind_group(0, &context.bind_group, &[]);
        self.set_index_buffer(context.index_buffer.slice(..));
        self.set_vertex_buffer(0, context.vertex_buffer.slice(..));
        self.set_viewport(0.0, 0.0, screen_width, screen_height, 0.0, 1.0);
        unsafe {
            let mut draw_command = nk__draw_begin(&mut context.context, &mut context.buffer);
            let mut index_offset = 0;
            loop {
                if draw_command.is_null() {
                    break;
                }
                if (*draw_command).elem_count != 0 {
                    let mut origin_x = (*draw_command).clip_rect.x;
                    if origin_x < 0.0 {
                        origin_x = 0.0;
                    }
                    let mut origin_y = (*draw_command).clip_rect.y;
                    if origin_y < 0.0 {
                        origin_y = 0.0;
                    }
                    self.set_scissor_rect(
                        origin_x as u32 * screen_width as u32,
                        origin_y as u32 * screen_height as u32,
                        (*draw_command).clip_rect.w as u32 * screen_width as u32,
                        (*draw_command).clip_rect.h as u32 * screen_height as u32,
                    );
                    self.draw_indexed(
                        index_offset..index_offset + (*draw_command).elem_count,
                        0,
                        0..1,
                    );
                    index_offset += (*draw_command).elem_count as u32;
                }
                draw_command =
                    nk__draw_next(draw_command, &mut context.buffer, &mut context.context);
            }
            nk_clear(&mut context.context);
        }
    }
}
