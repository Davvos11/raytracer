use crate::camera::Camera;
use crate::gpu::types::Vertex;
use std::io;
use std::io::Write;
use image::ImageFormat;
use wgpu::util::{BufferInitDescriptor, DeviceExt};
use wgpu::{include_wgsl, Backends, BlendState, Color, ColorTargetState, ColorWrites, CommandEncoderDescriptor, DeviceDescriptor, Face, Features, FragmentState, FrontFace, InstanceDescriptor, Limits, LoadOp, MemoryHints, MultisampleState, Operations, PipelineCompilationOptions, PipelineLayoutDescriptor, PolygonMode, PowerPreference, PrimitiveState, PrimitiveTopology, RenderPassColorAttachment, RenderPassDescriptor, RenderPipeline, RenderPipelineDescriptor, RequestAdapterOptions, StoreOp, Texture, TextureView, VertexState};

pub struct GPUState {
    device: wgpu::Device,
    queue: wgpu::Queue,
    texture_size: wgpu::Extent3d,
    texture: Texture,
    texture_view: TextureView,
    texture_width: u32,
    texture_height: u32,
    render_pipeline: RenderPipeline,
    output_buffer: wgpu::Buffer,
    vertex_buffer: wgpu::Buffer,
}

impl GPUState {
    pub async fn new(cam: &Camera) -> Self {
        let instance = wgpu::Instance::new(InstanceDescriptor {
            backends: Backends::all(),
            ..Default::default()
        });

        // Adapter: a handler for our graphics card
        // alternatively, instance.enumerate_adapters can be used to find adapters
        let adapter = instance
            .request_adapter(&RequestAdapterOptions {
                // power_p    usage: wgpu::TextureUsages::COPY_SRCreference can be LowPower or HighPerformance.
                power_preference: PowerPreference::default(),
                // Surface = None because we will draw to a file
                compatible_surface: None,
                // force_fallback_adapter forces an adapter for all hardware,
                // usually this means software rendering.
                force_fallback_adapter: false,
            })
            .await
            .expect("Could not get adapter");

        eprintln!("Connected to adapter: {:?}", adapter.get_info());

        // Device and queue
        // adapter.features() or device.features() can be used to find supported features
        let (device, queue) = adapter
            .request_device(
                &DeviceDescriptor {
                    required_features: Features::empty(),
                    required_limits: Limits::default(),
                    label: None,
                    memory_hints: MemoryHints::default(),
                },
                // Trace path:
                None,
            )
            .await
            .expect("Could not get device and queue");

        let texture_width = cam.image_width;
        let texture_height = cam.image_height();

        // Set up texture for our output file
        let texture_desc = wgpu::TextureDescriptor {
            size: wgpu::Extent3d {
                width: texture_width,
                height: texture_height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::COPY_SRC | wgpu::TextureUsages::RENDER_ATTACHMENT,
            label: None,
            view_formats: &[],
        };
        let texture = device.create_texture(&texture_desc);
        let texture_view = texture.create_view(&Default::default());

        // Output buffer where we will copy the pixels out of after rendering
        let output_buffer_size =
            (size_of::<u32>() as u32 * texture_width * texture_height) as wgpu::BufferAddress;
        let output_buffer_desc = wgpu::BufferDescriptor {
            size: output_buffer_size,
            usage: wgpu::BufferUsages::COPY_DST
                // this tells wpgu that we want to read this buffer from the cpu
                | wgpu::BufferUsages::MAP_READ,
            label: None,
            mapped_at_creation: false,
        };
        let output_buffer = device.create_buffer(&output_buffer_desc);

        // Load WGSL shaders
        let shader = device.create_shader_module(include_wgsl!("shader.wgsl"));

        // Create pipeline layout
        let render_pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: Some("Rener Pipeline layout"),
            bind_group_layouts: &[],
            push_constant_ranges: &[],
        });

        // Create pipeline
        // TODO add world and camera position and stuff (tm)
        let render_pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            // We need no vertex shader (I think)
            vertex: VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                // Vertex shader input
                buffers: &[],
                compilation_options: PipelineCompilationOptions::default(),
            },
            fragment: Some(FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                // Color outputs to set up (one for the surface)
                targets: &[Some(ColorTargetState {
                    format: texture_desc.format,
                    blend: Some(BlendState::REPLACE),
                    write_mask: ColorWrites::ALL,
                })],
                compilation_options: PipelineCompilationOptions::default(),
            }),
            primitive: PrimitiveState {
                // Each vertex is a point
                topology: PrimitiveTopology::TriangleList,
                strip_index_format: None,
                // Counter clockwise triangles
                front_face: FrontFace::Ccw,
                // Cull (= don't include) backsides of triangles
                cull_mode: Some(Face::Back),
                // These three options need features if you change them
                polygon_mode: PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            // Depth buffer, will be used later
            depth_stencil: None,
            multisample: MultisampleState {
                // Amount of samples for the pipeline
                count: 1,
                // Which samples should be active (!0 is all of them)
                mask: !0,
                // Used for antialiasing
                alpha_to_coverage_enabled: false,
            },
            // Array layers for render attachments
            multiview: None,
            // Only really useful for Android
            cache: None,
        });

        // Initialise vertex buffer
        let vertex_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("Vertex Buffer (triangle)"),
            contents: bytemuck::cast_slice(&[Vertex::default()]),
            usage: wgpu::BufferUsages::VERTEX,
        });

        Self {
            device,
            queue,
            texture_size: texture_desc.size,
            texture,
            texture_view,
            texture_width,
            texture_height,
            render_pipeline,
            output_buffer,
            vertex_buffer,
        }
    }

    pub async fn render(&mut self, writer: &mut (impl Write + io::Seek)) -> Result<(), image::ImageError> {
        // Create encoder for commands to ben sent to the GPU
        let mut encoder = self
            .device
            .create_command_encoder(&CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });

        {
            // Clear the screen
            let mut render_pass = encoder.begin_render_pass(&RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(RenderPassColorAttachment {
                    view: &self.texture_view,
                    resolve_target: None,
                    ops: Operations {
                        load: LoadOp::Clear(Color::BLACK),
                        store: StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                occlusion_query_set: None,
                timestamp_writes: None,
            });

            // Set shader pipeline
            render_pass.set_pipeline(&self.render_pipeline);
            // Draw (using single vertex)
            // render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
            render_pass.draw(0..3, 0..1);
        }

        let u32_size = size_of::<u32>() as u32;

        // Set the encoder to copy the result to our output buffer
        encoder.copy_texture_to_buffer(
            wgpu::ImageCopyTexture {
                aspect: wgpu::TextureAspect::All,
                texture: &self.texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
            },
            wgpu::ImageCopyBuffer {
                buffer: &self.output_buffer,
                layout: wgpu::ImageDataLayout {
                    offset: 0,
                    bytes_per_row: Some(u32_size * self.texture_width),
                    rows_per_image: Some(self.texture_height),
                },
            },
            self.texture_size,
        );

        // Submit the command buffer
        self.queue.submit(std::iter::once(encoder.finish()));
        
        {
            // Get the data out of the buffer
            let buffer_slice = self.output_buffer.slice(..);
            let (tx, rx) = futures_intrusive::channel::shared::oneshot_channel();
            buffer_slice.map_async(wgpu::MapMode::Read, move |result| {
                tx.send(result).unwrap();
            });
            self.device.poll(wgpu::Maintain::Wait);
            rx.receive().await.unwrap().unwrap();

            let data = buffer_slice.get_mapped_range();

            use image::{ImageBuffer, Rgba};
            let buffer =
                ImageBuffer::<Rgba<u8>, _>::from_raw(self.texture_width, self.texture_height, data).unwrap();
            buffer.write_to(writer, ImageFormat::Png)?;

            // // Write to ppm file
            // let header = format!("P3\n{} {}\n255\n", self.texture_width, self.texture_height);
            // writer.write_all(header.as_bytes())?;
            // 
            // for pixel in data.windows(4) {
            //     if let &[r, g, b, _] = pixel {
            //         let data = format!("{r} {g} {b}\n");
            //         writer.write_all(data.as_bytes())?;
            //     } else {
            //         panic!("Expected 4 bytes per pixel in output buffer");
            //     }
            // }
        }
        
        // Unmap the output buffer
        self.output_buffer.unmap();
        
        Ok(())
    }
    
    pub async fn generate(&self, camera: &Camera) {
        let camera_params = [camera.image_width as f32, camera.image_height() as f32,
            camera.center.x() as f32, camera.look_from.y() as f32, camera.look_from.z() as f32,
            camera.pixel00_loc.x() as f32, camera.pixel00_loc.y() as f32, camera.pixel00_loc.z() as f32,
            camera.pixel_delta_u.x() as f32, camera.pixel00_loc.y() as f32, camera.pixel00_loc.z() as f32,
            camera.pixel_delta_v.x() as f32, camera.pixel_delta_v.y() as f32, camera.pixel00_loc.z() as f32
        ];
        let camera_buffer = self.device.create_buffer_init(&BufferInitDescriptor {
            label: Some("Camera Buffer"),
            contents: bytemuck::cast_slice(&camera_params),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });
        
        let ray_buffer_size = (camera.image_width * camera.image_height() * size_of::<[f32; 6]>() as u32) as u64;
        let ray_buffer = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Ray buffer"),
            size: ray_buffer_size,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
            mapped_at_creation: false
        });
        
        // todo: pipeline, then read the ray buffer
    }
}
