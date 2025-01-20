use crate::camera::Camera;
use crate::gpu::types::CameraData;
use image::ImageFormat;
use std::io;
use std::io::Write;
use wgpu::util::{BufferInitDescriptor, DeviceExt};
use wgpu::{include_wgsl, Backends, BindGroupDescriptor, BindGroupEntry, BindGroupLayout, BindGroupLayoutDescriptor, BindGroupLayoutEntry, BindingType, BufferBindingType, Color, CommandEncoderDescriptor, ComputePassDescriptor, ComputePipeline, ComputePipelineDescriptor, DeviceDescriptor, Features, InstanceDescriptor, Limits, LoadOp, MemoryHints, Operations, PipelineLayoutDescriptor, PowerPreference, RenderPassColorAttachment, RenderPassDescriptor, RequestAdapterOptions, ShaderStages, StoreOp, Texture, TextureView};

pub struct GPUState {
    device: wgpu::Device,
    queue: wgpu::Queue,
    texture_size: wgpu::Extent3d,
    texture: Texture,
    texture_view: TextureView,
    texture_width: u32,
    texture_height: u32,
    pipelines: Pipelines,
    buffers: Buffers,
    bind_group_layout: BindGroupLayout,
}

struct Buffers {
    camera: wgpu::Buffer,
    ray: wgpu::Buffer,
    output: wgpu::Buffer,
}

struct Pipelines {
    generate: ComputePipeline,
}

impl GPUState {
    pub async fn new(cam: &mut Camera) -> Self {
        // Initialise camera
        cam.initialise();
        // Drop mutability of camera object
        let cam = &*cam;
        
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

        ////////////////////////////////////////////////////////////////////////////
        // Generate kernel
        ////////////////////////////////////////////////////////////////////////////
        // Load shader
        let generate_shader = device.create_shader_module(include_wgsl!("generate.wgsl"));

        let camera_params = [CameraData::from(cam)];
        let camera_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("Camera Buffer"),
            contents: bytemuck::cast_slice(&camera_params),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let ray_buffer_size =
            (cam.image_width * cam.image_height() * size_of::<[f32; 6]>() as u32) as u64;
        let ray_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Ray buffer"),
            size: ray_buffer_size,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
            mapped_at_creation: false,
        });

        // Instantiate pipeline
        let bind_group_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: None,
            entries: &[
                BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStages::COMPUTE,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: 1,
                    visibility: ShaderStages::COMPUTE,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Storage { read_only: false},
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
        });
        let layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: Some("GPU Pipeline Layout"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        let generate_pipeline = device.create_compute_pipeline(&ComputePipelineDescriptor {
            label: Some("Generate kernel"),
            layout: Some(&layout),
            module: &generate_shader,
            entry_point: Some("main"),
            compilation_options: Default::default(),
            cache: None,
        });

        let buffers = Buffers {
            camera: camera_buffer,
            ray: ray_buffer,
            output: output_buffer,
        };

        let pipelines = Pipelines {
            generate: generate_pipeline,
        };

        Self {
            device,
            queue,
            texture_size: texture_desc.size,
            texture,
            texture_view,
            texture_width,
            texture_height,
            buffers,
            pipelines,
            bind_group_layout,
        }
    }

    pub async fn render(
        &mut self,
        writer: &mut (impl Write + io::Seek),
    ) 
        -> Result<(), image::ImageError> {
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
            // render_pass.set_pipeline(&self.render_pipeline);
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
                buffer: &self.buffers.output,
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
            let buffer_slice = self.buffers.output.slice(..);
            let (tx, rx) = futures_intrusive::channel::shared::oneshot_channel();
            buffer_slice.map_async(wgpu::MapMode::Read, move |result| {
                tx.send(result).unwrap();
            });
            self.device.poll(wgpu::Maintain::Wait);
            rx.receive().await.unwrap().unwrap();

            let data = buffer_slice.get_mapped_range();

            use image::{ImageBuffer, Rgba};
            let buffer =
                ImageBuffer::<Rgba<u8>, _>::from_raw(self.texture_width, self.texture_height, data)
                    .unwrap();
            buffer.write_to(writer, ImageFormat::Png)?;
        }

        // Unmap the output buffer
        self.buffers.output.unmap();

        Ok(())
    }

    pub async fn print_output_buffer(&self) {
        // Get the data out of the buffer
        let buffer_slice = self.buffers.output.slice(..);
        let (tx, rx) = futures_intrusive::channel::shared::oneshot_channel();
        buffer_slice.map_async(wgpu::MapMode::Read, move |result| {
            tx.send(result).unwrap();
        });
        self.device.poll(wgpu::Maintain::Wait).panic_on_timeout();
        rx.receive().await.unwrap().unwrap();

        let data = buffer_slice.get_mapped_range();
        let result: &[f32] = bytemuck::cast_slice(&data);
        dbg!(&result[0..100]);
    }

    pub async fn generate(&self, debug: bool) {
        // Instantiate bind groups
        let bind_group = self.device.create_bind_group(&BindGroupDescriptor {
            label: None,
            layout: &self.bind_group_layout,
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: self.buffers.camera.as_entire_binding(),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: self.buffers.ray.as_entire_binding(),
                },
            ],
        });

        let mut encoder = self
            .device
            .create_command_encoder(&CommandEncoderDescriptor {
                label: Some("Generate encoder"),
            });

        {
            let mut compute_pass = encoder.begin_compute_pass(&ComputePassDescriptor {
                label: Some("Generate pass"),
                timestamp_writes: None,
            });
            compute_pass.set_pipeline(&self.pipelines.generate);
            compute_pass.set_bind_group(0, &bind_group, &[]);
            compute_pass.insert_debug_marker("Generate kernel");
            compute_pass.dispatch_workgroups(self.texture_width, self.texture_height, 1);
        }

        if debug {
            // Copy ray buffer to output (CPU) buffer
            encoder.copy_buffer_to_buffer(
                &self.buffers.ray,
                0,
                &self.buffers.output,
                0,
                self.buffers.output.size(),
            );
        }

        self.queue.submit(Some(encoder.finish()));

        if debug {
            self.print_output_buffer().await;
        }
    }
}
