pub struct ResultTexture {
    pub texture: wgpu::Texture,
    pub bind_group_layout: wgpu::BindGroupLayout,
    pub bind_group: wgpu::BindGroup,
    pub rendering_bind_group_layout: wgpu::BindGroupLayout,
    pub rendering_bind_group: wgpu::BindGroup,
}

impl ResultTexture {
    pub fn new(device: &wgpu::Device) -> Self {
        let main_texture = create_texture(&device, 1, 1);

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Texture Bind Group Layout"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::StorageTexture {
                    access: wgpu::StorageTextureAccess::WriteOnly,
                    format: wgpu::TextureFormat::Rgba32Float,
                    view_dimension: wgpu::TextureViewDimension::D2,
                },
                count: None,
            }],
        });
        let bind_group = create_texture_bind_group(&device, &bind_group_layout, &main_texture);

        let rendering_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Rendering Texture Bind Group Layout"),
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            sample_type: wgpu::TextureSampleType::Float { filterable: false },
                            view_dimension: wgpu::TextureViewDimension::D2,
                            multisampled: false,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::NonFiltering),
                        count: None,
                    },
                ],
            });
        let rendering_bind_group = create_rendering_texture_bind_group(
            &device,
            &rendering_bind_group_layout,
            &main_texture,
        );

        Self {
            texture: main_texture,
            bind_group_layout,
            bind_group,
            rendering_bind_group_layout,
            rendering_bind_group,
        }
    }

    pub fn resize(&mut self, device: &wgpu::Device, width: u32, height: u32) {
        self.texture = create_texture(device, width, height);
        self.bind_group = create_texture_bind_group(device, &self.bind_group_layout, &self.texture);
        self.rendering_bind_group = create_rendering_texture_bind_group(
            device,
            &self.rendering_bind_group_layout,
            &self.texture,
        );
    }
}

fn create_texture(device: &wgpu::Device, width: u32, height: u32) -> wgpu::Texture {
    device.create_texture(&wgpu::wgt::TextureDescriptor {
        label: Some("Texture"),
        size: wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba32Float,
        usage: wgpu::TextureUsages::STORAGE_BINDING | wgpu::TextureUsages::TEXTURE_BINDING,
        view_formats: &[],
    })
}

fn create_texture_bind_group(
    device: &wgpu::Device,
    main_texture_bind_group_layout: &wgpu::BindGroupLayout,
    main_texture: &wgpu::Texture,
) -> wgpu::BindGroup {
    device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("Texture Bind Group"),
        layout: main_texture_bind_group_layout,
        entries: &[wgpu::BindGroupEntry {
            binding: 0,
            resource: wgpu::BindingResource::TextureView(
                &main_texture.create_view(&Default::default()),
            ),
        }],
    })
}

fn create_rendering_texture_bind_group(
    device: &wgpu::Device,
    rendering_main_texture_bind_group_layout: &wgpu::BindGroupLayout,
    main_texture: &wgpu::Texture,
) -> wgpu::BindGroup {
    device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("Rendering Texture Bind Group"),
        layout: rendering_main_texture_bind_group_layout,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(
                    &main_texture.create_view(&Default::default()),
                ),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: wgpu::BindingResource::Sampler(&device.create_sampler(
                    &wgpu::SamplerDescriptor {
                        label: Some("Rendering Texture Sampler"),
                        address_mode_u: wgpu::AddressMode::ClampToEdge,
                        address_mode_v: wgpu::AddressMode::ClampToEdge,
                        address_mode_w: wgpu::AddressMode::ClampToEdge,
                        mag_filter: wgpu::FilterMode::Nearest,
                        min_filter: wgpu::FilterMode::Nearest,
                        mipmap_filter: wgpu::FilterMode::Nearest,
                        ..Default::default()
                    },
                )),
            },
        ],
    })
}
