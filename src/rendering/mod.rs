use super::data::*;
use gfx;
use gfx::Device;
use gfx_window_glutin;
use gfx::traits::FactoryExt;
use glutin;
use gfx::Factory;
use gfx_device_gl::{Resources, CommandBuffer, Device as GlDevice};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

const WINDOW_TITLE: &'static str = "Simple Life";

const QUAD_VERTICES: [Vertex; 4] = [Vertex { position: [-0.5, 0.5] },
                                    Vertex { position: [-0.5, -0.5] },
                                    Vertex { position: [0.5, -0.5] },
                                    Vertex { position: [0.5, 0.5] }];

const QUAD_INDICES: [u16; 6] = [0, 1, 2, 2, 3, 0];

const CLEARING_COLOR: [f32; 4] = [0.1, 0.2, 0.3, 1.0];

const WHITE: [f32; 4] = [1., 1., 1., 1.];
const COLOURED: [f32; 4] = [0.2, 0.4, 0.5, 1.];

const SCALE_TOTAL: f32 = 2.0;
const INSTANCE_PORTION: f32 = 1.8;

pub type ColorFormat = gfx::format::Rgba8;
pub type DepthFormat = gfx::format::DepthStencil;

gfx_defines!{
    vertex Vertex {
        position: [f32; 2] = "a_Position",
    }

    vertex Instance {
        translate: [f32; 2] = "a_Translate",
        colour: [f32; 4] = "a_Color",
    }

    constant Locals {
        scale: [[f32;2];2] = "u_Scale",
    }

    pipeline pipe {
        vertex: gfx::VertexBuffer<Vertex> = (),
        instance: gfx::InstanceBuffer<Instance> = (),
        scale: gfx::Global<[[f32;2];2]> = "u_Scale",
        locals: gfx::ConstantBuffer<Locals> = "Locals",
        out: gfx::RenderTarget<ColorFormat> = "Target0",
    }
}

// Fills the provided instance buffer, but also returns a vector of instances for later
// manipulation, when we want to update the instances and update the buffer again.
fn fill_instances(instances: &mut [Instance], grid: &Grid, size: [[f32; 2]; 2]) -> Vec<Instance> {
    let width = grid.width();
    let height = grid.height();
    let cells = grid.cells();

    let size_x = size[0][0];
    let size_y = size[1][1];
    let scale_remaining = SCALE_TOTAL - INSTANCE_PORTION;
    let gap_x = scale_remaining / (width + 1) as f32;
    let gap_y = scale_remaining / (height + 1) as f32;
    let begin_x = -1. + gap_x + (size_x / 2.);
    let begin_y = -1. + gap_y + (size_y / 2.);

    let mut translate = [begin_x, begin_y];

    let mut v = Vec::with_capacity(grid.area());
    let mut index = 0;
    for i in 0..height {
        for j in 0..width {
            let cell = cells[i][j];
            let colour = if cell.alive() { COLOURED } else { WHITE };
            let inst = Instance { translate, colour };
            v.push(inst.clone());
            instances[index] = inst;
            translate[0] += size_x + gap_x;
            index += 1;
        }
        translate[1] += size_y + gap_y;
        translate[0] = begin_x
    }
    v
}

pub struct App {
    grid: Arc<Mutex<Grid>>,
    updates_per_second: u16,
    window: glutin::Window,
    device: GlDevice,
    // main_depth: DepthStencilView<Resources, DepthFormat>,
    events_loop: glutin::EventsLoop,
    pso: gfx::PipelineState<Resources, pipe::Meta>,
    data: pipe::Data<Resources>,
    encoder: gfx::Encoder<Resources, CommandBuffer>,
    slice: gfx::Slice<Resources>,
    upload: gfx::handle::Buffer<Resources, Instance>,
    instances: Vec<Instance>,
    uploading: bool,
}

impl App {
    pub fn new(grid: Grid, window_width: u32, window_height: u32, updates_per_second: u16) -> Self {
        let events_loop = glutin::EventsLoop::new();
        let builder = glutin::WindowBuilder::new()
            .with_title(WINDOW_TITLE)
            .with_dimensions(window_width, window_height)
            .with_vsync();
        let (window, device, mut factory, main_color, _) =
            gfx_window_glutin::init::<ColorFormat, DepthFormat>(builder, &events_loop);
        let encoder = factory.create_command_buffer().into();

        let width: u32 = grid.width() as u32;
        let height: u32 = grid.height() as u32;
        let area = grid.area() as u32;

        let size = [[INSTANCE_PORTION / width as f32, 0.],
                    [0., INSTANCE_PORTION / height as f32]];

        let upload = factory.create_upload_buffer(area as usize).unwrap();
        let insts = {
            let mut writer = factory.write_mapping(&upload).unwrap();
            fill_instances(&mut writer, &grid, size)
        };

        let instances = factory
            .create_buffer(area as usize,
                           gfx::buffer::Role::Vertex,
                           gfx::memory::Usage::Dynamic,
                           gfx::TRANSFER_DST)
            .unwrap();


        let (quad_vertices, mut slice) =
            factory.create_vertex_buffer_with_slice(&QUAD_VERTICES, &QUAD_INDICES[..]);
        slice.instances = Some((area, 0));
        let locals = Locals { scale: size };

        App {
            grid: Arc::new(Mutex::new(grid)),
            updates_per_second: updates_per_second,
            window: window,
            device: device,
            events_loop: events_loop,
            // main_depth: main_depth,
            pso: factory
                .create_pipeline_simple(include_bytes!("shaders/instancing.glslv"),
                                        include_bytes!("shaders/instancing.glslf"),
                                        pipe::new())
                .unwrap(),
            encoder: encoder,
            data: pipe::Data {
                vertex: quad_vertices,
                instance: instances,
                scale: size,
                locals: factory
                    .create_buffer_immutable(&[locals],
                                             gfx::buffer::Role::Constant,
                                             gfx::Bind::empty())
                    .unwrap(),
                out: main_color,
            },
            instances: insts,
            slice: slice,
            upload: upload,
            uploading: true,
        }
    }

    fn render(&mut self) {
        let grid = self.grid.lock().unwrap();
        let encoder = &mut self.encoder;

        if self.uploading {
            encoder
                .copy_buffer(&self.upload, &self.data.instance, 0, 0, self.upload.len())
                .unwrap();
            self.uploading = false;
        } else {
            let cells = grid.cells();
            let mut buffer_idx = 0;
            for i in 0..grid.height() {
                for j in 0..grid.width() {
                    let cell = cells[i][j];
                    let colour = if cell.alive() { COLOURED } else { WHITE };
                    self.instances[buffer_idx].colour = colour;
                    buffer_idx += 1;
                }
            }
            encoder
                .update_buffer(&self.data.instance, &self.instances, 0)
                .unwrap();
        }

        encoder.clear(&self.data.out, CLEARING_COLOR);
        encoder.draw(&self.slice, &self.pso, &self.data);
    }

    pub fn run(&mut self) {
        // Do updates to the grid in another thread.
        {
            let grid = self.grid.clone();
            let updates_per_second = self.updates_per_second;
            thread::spawn(move || {
                let wait_duration = Duration::from_millis(1000 / updates_per_second as u64);
                let mut last_updated = Instant::now();
                loop {
                    if last_updated.elapsed() > wait_duration {
                        let mut grid = grid.lock().unwrap();
                        grid.advance();
                        last_updated = Instant::now()
                    }
                }
            });
        }

        let mut running = true;
        while running {
            // fetch events
            self.events_loop
                .poll_events(|glutin::Event::WindowEvent {
                                  window_id: _,
                                  event,
                              }| {
                    match event {
                        glutin::WindowEvent::KeyboardInput(_,
                                                       _,
                                                       Some(glutin::VirtualKeyCode::Escape),
                                                       _) |
                    glutin::WindowEvent::Closed => running = false,
                        glutin::WindowEvent::Resized(_width, _height) => {
                            if !self.uploading {
                                running = false
                            }
                        } // handle eventually
                        _ => {}
                    }
                });
            self.render();
            self.encoder.flush(&mut self.device);
            self.window.swap_buffers().unwrap();
            self.device.cleanup();
        }
    }
}
