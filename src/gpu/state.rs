use crate::camera::Camera;
use crate::gpu::types::{CameraData, SphereData, TriangleData};
use crate::hittable::hittable_list::HittableList;
use crate::value::vec3::Vec3;
use image::ImageFormat;
use std::cmp::min;
use std::io;
use std::io::Write;
use wgpu::util::{BufferInitDescriptor, DeviceExt};
use wgpu::{include_wgsl, Backends, BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayoutDescriptor, BindGroupLayoutEntry, BindingType, BufferBindingType, BufferSize, Color, CommandEncoderDescriptor, ComputePassDescriptor, ComputePipeline, ComputePipelineDescriptor, DeviceDescriptor, Features, InstanceDescriptor, Limits, LoadOp, MemoryHints, Operations, PipelineLayoutDescriptor, PowerPreference, RenderPassColorAttachment, RenderPassDescriptor, RequestAdapterOptions, ShaderStages, StoreOp, Texture, TextureView};
use crate::utils::get_non_zero;
use crate::utils::rtweekend::random_float;

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
    bind_group: BindGroup,
}

struct Buffers {
    output: wgpu::Buffer,
    debug: wgpu::Buffer,
    camera: wgpu::Buffer,
    max_depth: wgpu::Buffer,
    ray: wgpu::Buffer,
    triangle: wgpu::Buffer,
    sphere: wgpu::Buffer,
    random_unit: wgpu::Buffer,
    random_double: wgpu::Buffer,
    pixel: wgpu::Buffer,
}

struct Pipelines {
    generate: ComputePipeline,
    extend: ComputePipeline,
    shade: ComputePipeline,
}

impl GPUState {
    pub async fn new(cam: &mut Camera, world: &HittableList) -> Self {
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
            label: Some("Output Buffer"),
            mapped_at_creation: false,
        };
        let output_buffer = device.create_buffer(&output_buffer_desc);

        // Debug buffer
        let debug_buffer_desc = wgpu::BufferDescriptor {
            size: output_buffer_size,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
            label: Some("Debug Buffer"),
            mapped_at_creation: false,
        };
        let debug_buffer = device.create_buffer(&debug_buffer_desc);
        
        // Setup layout
        // We set this up globally so that all compute shaders can use the same buffers
        let bind_group_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: None,
            entries: &[
                BindGroupLayoutEntry {
                    // Camera data
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
                    // Ray buffer
                    binding: 1,
                    visibility: ShaderStages::COMPUTE,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Storage { read_only: false},
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                BindGroupLayoutEntry {
                    // Triangle data
                    binding: 2,
                    visibility: ShaderStages::COMPUTE,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Storage { read_only: true},
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                BindGroupLayoutEntry {
                    // Sphere data
                    binding: 3,
                    visibility: ShaderStages::COMPUTE,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Storage { read_only: true},
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                BindGroupLayoutEntry {
                    // Random unit buffer
                    binding: 6,
                    visibility: ShaderStages::COMPUTE,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Storage { read_only: true},
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                BindGroupLayoutEntry {
                    // Pixel buffer
                    binding: 7,
                    visibility: ShaderStages::COMPUTE,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Storage { read_only: false},
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                BindGroupLayoutEntry {
                    // Random double buffer
                    binding: 8,
                    visibility: ShaderStages::COMPUTE,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Storage { read_only: true},
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                BindGroupLayoutEntry {
                    // Max depth data
                    binding: 98,
                    visibility: ShaderStages::COMPUTE,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                BindGroupLayoutEntry {
                    // Debug data
                    binding: 99,
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


        ////////////////////////////////////////////////////////////////////////////
        // Generate kernel
        ////////////////////////////////////////////////////////////////////////////
        let generate_shader = device.create_shader_module(include_wgsl!("generate.wgsl"));

        let camera_data = [CameraData::from(cam)];
        let camera_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("Camera Buffer"),
            contents: bytemuck::cast_slice(&camera_data),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });
        
        let max_depth = [10u32];
        let max_depth_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("Max Depth Buffer"),
            contents: bytemuck::cast_slice(&max_depth),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST
        });

        let ray_item_size = (size_of::<f32>() * 10 + size_of::<u32>()) as u32;
        let ray_buffer_size =
            (texture_width * texture_height * ray_item_size) as u64;
        let ray_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Ray Buffer"),
            size: ray_buffer_size,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
            mapped_at_creation: false,
        });

        // Set up pipeline
        let generate_pipeline = device.create_compute_pipeline(&ComputePipelineDescriptor {
            label: Some("Generate kernel"),
            layout: Some(&layout),
            module: &generate_shader,
            entry_point: Some("main"),
            compilation_options: Default::default(),
            cache: None,
        });
        
        ////////////////////////////////////////////////////////////////////////////
        // Extend kernel
        ////////////////////////////////////////////////////////////////////////////
        let extend_shader = device.create_shader_module(include_wgsl!("extend.wgsl"));

        let mut triangle_data: Vec<_> = world.objects.iter()
            .filter_map(|o| o.as_triangle())
            .map(TriangleData::from)
            .collect();
        if triangle_data.is_empty() {
            // Empty storage buffers are not allowed
            triangle_data.push(TriangleData::default());
        }
        let triangle_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("Triangle Buffer"),
            contents: bytemuck::cast_slice(&triangle_data),
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
        });

        let mut sphere_data: Vec<_> = world.objects.iter()
            .filter_map(|o| o.as_sphere())
            .map(SphereData::from)
            .collect();
        if sphere_data.is_empty() {
            // Empty storage buffers are not allowed
            sphere_data.push(SphereData::default());
        }
        let sphere_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("Sphere Buffer"),
            contents: bytemuck::cast_slice(&sphere_data),
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
        });

        // Set up pipeline
        let extend_pipeline = device.create_compute_pipeline(&ComputePipelineDescriptor {
            label: Some("Extend kernel"),
            layout: Some(&layout),
            module: &extend_shader,
            entry_point: Some("main"),
            compilation_options: Default::default(),
            cache: None,
        });

        ////////////////////////////////////////////////////////////////////////////
        // Shade kernel
        ////////////////////////////////////////////////////////////////////////////
        let shade_shader = device.create_shader_module(include_wgsl!("shade.wgsl"));

        let vector_size = (size_of::<f32>() * 3 * 3) as u32;
        let random_unit_buffer_size =
            (texture_width * texture_height * vector_size) as u64;
        let random_unit_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Random unit Buffer"),
            size: random_unit_buffer_size,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let random_double_buffer_size =
            (texture_width * texture_height * (size_of::<f32>() as u32)) as u64;
        let random_double_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Random double Buffer"),
            size: random_double_buffer_size,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        // Set up pipeline
        let shade_pipeline = device.create_compute_pipeline(&ComputePipelineDescriptor {
            label: Some("Shade kernel"),
            layout: Some(&layout),
            module: &shade_shader,
            entry_point: Some("main"),
            compilation_options: Default::default(),
            cache: None,
        });

        ////////////////////////////////////////////////////////////////////////////
        // Connect kernel
        ////////////////////////////////////////////////////////////////////////////
        //let connect_shader = device.create_shader_module(include_wgsl!("connect.wgsl"));

        let pixel_buffer_size =
            (texture_width * texture_height * vector_size) as u64;
        let pixel_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Pixel Buffer"),
            size: pixel_buffer_size,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
            mapped_at_creation: false,
        });

        // Set up pipeline
        /*let connect_pipeline = device.create_compute_pipeline(&ComputePipelineDescriptor {
            label: Some("Connect kernel"),
            layout: Some(&layout),
            module: &connect_shader,
            entry_point: Some("main"),
            compilation_options: Default::default(),
            cache: None,
        });*/


        ////////////////////////////////////////////////////////////////////////////

        let buffers = Buffers {
            output: output_buffer,
            debug: debug_buffer,
            camera: camera_buffer,
            max_depth: max_depth_buffer,
            ray: ray_buffer,
            triangle: triangle_buffer,
            sphere: sphere_buffer,
            random_unit: random_unit_buffer,
            random_double: random_double_buffer,
            pixel: pixel_buffer,
        };

        let pipelines = Pipelines {
            generate: generate_pipeline,
            extend: extend_pipeline,
            shade: shade_pipeline,
        };

        // Instantiate bind groups
        let bind_group = device.create_bind_group(&BindGroupDescriptor {
            label: None,
            layout: &bind_group_layout,
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: buffers.camera.as_entire_binding(),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: buffers.ray.as_entire_binding(),
                },
                BindGroupEntry {
                    binding: 2,
                    resource: buffers.triangle.as_entire_binding(),
                },
                BindGroupEntry {
                    binding: 3,
                    resource: buffers.sphere.as_entire_binding(),
                },
                BindGroupEntry {
                    binding: 6,
                    resource: buffers.random_unit.as_entire_binding(),
                },
                BindGroupEntry {
                    binding: 7,
                    resource: buffers.pixel.as_entire_binding(),
                },
                BindGroupEntry {
                    binding: 8,
                    resource: buffers.random_double.as_entire_binding()
                },
                BindGroupEntry {
                    binding: 98,
                    resource: buffers.max_depth.as_entire_binding()
                },
                BindGroupEntry {
                    binding: 99,
                    resource: buffers.debug.as_entire_binding(),
                },
            ],
        });


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
            bind_group,
        }
    }

    #[allow(dead_code)]
    // TODO get the code to render to an image from here, then remove this function
    pub async fn old_render(
        &self,
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

    pub async fn get_output_buffer<T: bytemuck::Pod>(&self) -> Vec<T> {
        let mut result: Vec<T> = vec![];
        {
            // Get the data out of the buffer
            let buffer_slice = self.buffers.output.slice(..);
            let (tx, rx) = futures_intrusive::channel::shared::oneshot_channel();
            buffer_slice.map_async(wgpu::MapMode::Read, move |result| {
                tx.send(result).unwrap();
            });
            self.device.poll(wgpu::Maintain::Wait).panic_on_timeout();
            rx.receive().await.unwrap().unwrap();

            let data = buffer_slice.get_mapped_range();
            bytemuck::cast_slice(&data).clone_into(&mut result);
        }

        // Unmap the output buffer
        self.buffers.output.unmap();
        result
    }

    async fn run_pipeline<T: bytemuck::Pod>(&self, pipeline: &ComputePipeline, label: &str, debug: bool) -> Option<Vec<T>> {
        let mut encoder = self
            .device
            .create_command_encoder(&CommandEncoderDescriptor {
                label: Some(&format!("{label} encoder")),
            });

        {
            let mut compute_pass = encoder.begin_compute_pass(&ComputePassDescriptor {
                label: Some(&format!("{label} pass")),
                timestamp_writes: None,
            });
            compute_pass.set_pipeline(pipeline);
            compute_pass.set_bind_group(0, &self.bind_group, &[]);
            compute_pass.insert_debug_marker(&format!("{label} kernel"));
            compute_pass.dispatch_workgroups(self.texture_width, self.texture_height, 1);
        }

        if debug {
            // Copy ray buffer to output (CPU) buffer
            encoder.copy_buffer_to_buffer(
                &self.buffers.debug,
                0,
                &self.buffers.output,
                0,
                min(self.buffers.debug.size(), self.buffers.output.size()),
            );
        }

        self.queue.submit(Some(encoder.finish()));

        if debug {
            Some(self.get_output_buffer().await)
        } else {
            None
        }
    }
    
    pub async fn render(&self, _writer: &mut (impl Write + io::Seek)) -> Result<(), image::ImageError> {
        let debug = true;
        let generate_debug = self.generate::<f32>(debug).await;
        let extend_debug = self.extend::<u32>(debug).await;
        let shade_debug = self.shade::<f32>(debug).await;
        //let connect_debug = self.connect::<f32>(debug).await;
        if debug {
            let generate_debug = generate_debug.unwrap();
            let extend_debug = extend_debug.unwrap();
            let shade_debug = shade_debug.unwrap();
            //let connect_debug = connect_debug.unwrap();
            
            println!("Generate:    {:?}", &generate_debug[0..20]);
            
            println!("Extend:      {:?}", &extend_debug[0..20]);
            let extend_nonzero = get_non_zero(&extend_debug);
            println!("Extend != 0: {:?}", extend_nonzero.iter().take(20).collect::<Vec<_>>());
            println!("Extend != 0: {}", extend_nonzero.len());

            println!("Shade:       {:?}", &shade_debug[0..20]);
            let shade_nonzero = get_non_zero(&shade_debug);
            println!("Shade != 0:  {:?}", shade_nonzero.iter().take(20).collect::<Vec<_>>());
            println!("Shade != 0:  {}", shade_nonzero.len());
            
            //println!("Connect:     {:?}", &connect_debug[0..20]);
            //let connect_nonzero = get_non_zero(&connect_debug);
            //println!("Connect != 0:{:?}", connect_nonzero.iter().take(20).collect::<Vec<_>>());
            //println!("Connect != 0:{}", connect_nonzero.len());
        }
        
        Ok(())
    }
    
    async fn generate<T: bytemuck::Pod>(&self, debug: bool) -> Option<Vec<T>> {
        self.run_pipeline(&self.pipelines.generate, "Generate", debug).await
    }

    async fn extend<T: bytemuck::Pod>(&self, debug: bool) -> Option<Vec<T>> {
        self.run_pipeline(&self.pipelines.extend, "Extend", debug).await
    }

    /// Make sure to call `self.queue.submit([])` after this function.
    /// Or use `self.write_to_buffer_submit` instead.
    fn write_to_buffer<T: bytemuck::Pod>(&self, buffer: &wgpu::Buffer, data: &[T]) {
        // Cast data to bytes
        let data = bytemuck::cast_slice(data);
        let data_len = BufferSize::new(data.len() as u64).expect("Size should be > 0");
        // Move vectors to buffer
        let mut write = self.queue.write_buffer_with(buffer, 0, data_len)
            .expect("Failed to create buffer writer for random unit buffer");
        write.as_mut().copy_from_slice(data);
    }

    #[allow(dead_code)]
    fn write_to_buffer_submit<T: bytemuck::Pod>(&self, buffer: &wgpu::Buffer, data: &[T]) {
        self.write_to_buffer(buffer, data);
        self.queue.submit([]);
    }

    async fn shade<T: bytemuck::Pod>(&self, debug: bool) -> Option<Vec<T>> {
        // Generate new random unit vectors
        let amount = self.buffers.random_unit.size() / 4 / 3;
        let random_unit_vectors: Vec<[f32; 3]> = (0..amount)
            .map(|_| Vec3::random_unit().into())
            .collect();
        self.write_to_buffer(&self.buffers.random_unit, &random_unit_vectors);
        // Generate new random floats
        let amount = self.buffers.random_double.size() / 4;
        let random_floats: Vec<f32> = (0..amount)
            .map(|_| random_float())
            .collect();
        self.write_to_buffer(&self.buffers.random_double, &random_floats);
        self.queue.submit([]);

        self.run_pipeline(&self.pipelines.shade, "Shade", debug).await
    }

    /*async fn connect<T: bytemuck::Pod>(&self, debug: bool) -> Option<Vec<T>> {
        self.run_pipeline(&self.pipelines.connect, "Connect", debug).await
    }*/
}
