use std::num::NonZeroU32;

use bevy::{
    math::{Size, UVec2},
    prelude::{Handle, Image, Res},
    render::{
        render_asset::RenderAssets,
        render_resource::{
            AddressMode, CommandEncoderDescriptor, Extent3d, FilterMode, ImageCopyTexture, Origin3d, SamplerDescriptor,
            TextureAspect, TextureDescriptor, TextureDimension, TextureFormat, TextureUsages, TextureViewDescriptor,
            TextureViewDimension,
        },
        renderer::{RenderDevice, RenderQueue},
        texture::{BevyDefault, GpuImage},
    },
    utils::{HashMap, HashSet},
};

/* --- Based on code from: https://github.com/StarArawn/bevy_ecs_tilemap/blob/main/src/render/texture_array_cache.rs --- */

#[derive(Debug, Clone)]
struct AtlasInfo {
    pub tile_size: UVec2,
    pub texture_size: UVec2,
    pub padding: UVec2,
    pub filter: FilterMode,
}

#[derive(Default, Debug, Clone)]
pub struct TextureArrayCache {
    textures: HashMap<Handle<Image>, GpuImage>,
    atlas_info: HashMap<Handle<Image>, AtlasInfo>,
    prepare_queue: HashSet<Handle<Image>>,
    queue_queue: HashSet<Handle<Image>>,
}

impl TextureArrayCache {
    /// Adds an atlas to the texture array cache.
    pub fn add(
        &mut self,
        atlas_texture: &Handle<Image>,
        tile_size: UVec2,
        texture_size: UVec2,
        padding: UVec2,
        filter: FilterMode,
    ) {
        if self.atlas_info.contains_key(atlas_texture) {
            return;
        }

        let atlas_info = AtlasInfo {
            tile_size,
            texture_size,
            padding,
            filter,
        };

        self.atlas_info.insert(atlas_texture.clone_weak(), atlas_info);

        self.prepare_queue.insert(atlas_texture.clone_weak());
    }

    pub fn get(&self, image_handle: &Handle<Image>) -> &GpuImage {
        self.textures.get(image_handle).unwrap()
    }

    pub fn contains(&self, image_handle: &Handle<Image>) -> bool {
        self.textures.contains_key(image_handle)
    }

    /// Prepares each texture array texture
    pub fn prepare(&mut self, render_device: &RenderDevice) {
        let prepare_queue = self.prepare_queue.drain().collect::<Vec<_>>();

        for item in prepare_queue {
            let AtlasInfo {
                tile_size,
                texture_size,
                padding,
                filter,
            } = self.atlas_info.get(&item).unwrap();

            let tile_count = (*texture_size + *padding) / (*tile_size + *padding);
            let count = tile_count.x * tile_count.y;

            let texture = render_device.create_texture(&TextureDescriptor {
                label: Some("texture_array"),
                size: Extent3d {
                    width: tile_size.x,
                    height: tile_size.y,
                    depth_or_array_layers: count,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: TextureDimension::D2,
                format: TextureFormat::Rgba8UnormSrgb,
                usage: TextureUsages::COPY_DST | TextureUsages::TEXTURE_BINDING,
            });

            let sampler = render_device.create_sampler(&SamplerDescriptor {
                label: Some("texture_array_sampler"),
                address_mode_u: AddressMode::ClampToEdge,
                address_mode_v: AddressMode::ClampToEdge,
                address_mode_w: AddressMode::ClampToEdge,
                mag_filter: *filter,
                min_filter: *filter,
                mipmap_filter: *filter,
                lod_min_clamp: 0.0,
                lod_max_clamp: std::f32::MAX,
                compare: None,
                anisotropy_clamp: None,
                border_color: None,
            });

            let texture_view = texture.create_view(&TextureViewDescriptor {
                label: Some("texture_array_view"),
                format: None,
                dimension: Some(TextureViewDimension::D2Array),
                aspect: TextureAspect::All,
                base_mip_level: 0,
                mip_level_count: None,
                base_array_layer: 0,
                array_layer_count: NonZeroU32::new(count),
            });

            let gpu_image = GpuImage {
                texture_format: TextureFormat::bevy_default(),
                texture,
                sampler,
                texture_view,
                size: Size::new(tile_size.x as f32, tile_size.y as f32),
            };

            self.textures.insert(item.clone_weak(), gpu_image);
            self.queue_queue.insert(item.clone_weak());
        }
    }

    pub fn queue(
        &mut self,
        render_device: &RenderDevice,
        render_queue: &RenderQueue,
        gpu_images: &Res<RenderAssets<Image>>,
    ) {
        let queue_queue = self.queue_queue.drain().collect::<Vec<_>>();

        for item in queue_queue {
            let atlas_image = if let Some(atlas_image) = gpu_images.get(&item) {
                atlas_image
            } else {
                self.prepare_queue.insert(item);
                continue;
            };

            let AtlasInfo {
                tile_size,
                texture_size,
                padding,
                ..
            } = self.atlas_info.get(&item).unwrap();

            let array_gpu_image = self.textures.get(&item).unwrap();
            let tile_count = (*texture_size + *padding) / (*tile_size + *padding);
            let count = tile_count.x * tile_count.y;

            let mut command_encoder = render_device.create_command_encoder(&CommandEncoderDescriptor {
                label: Some("create_texture_array_from_atlas"),
            });

            for i in 0..count {
                let columns = (texture_size.x + padding.x) / (tile_size.x + padding.x);
                let sprite_sheet_x = (i % columns) * (tile_size.x + padding.x);
                let sprite_sheet_y = (i / columns) * (tile_size.y + padding.y);

                command_encoder.copy_texture_to_texture(
                    ImageCopyTexture {
                        texture: &atlas_image.texture,
                        mip_level: 0,
                        origin: Origin3d {
                            x: sprite_sheet_x,
                            y: sprite_sheet_y,
                            z: 0,
                        },
                        aspect: TextureAspect::All,
                    },
                    ImageCopyTexture {
                        texture: &array_gpu_image.texture,
                        mip_level: 0,
                        origin: Origin3d { x: 0, y: 0, z: i },
                        aspect: TextureAspect::All,
                    },
                    Extent3d {
                        width: tile_size.x,
                        height: tile_size.y,
                        depth_or_array_layers: 1,
                    },
                );
            }

            let command_buffer = command_encoder.finish();
            render_queue.submit(vec![command_buffer]);
        }
    }
}
