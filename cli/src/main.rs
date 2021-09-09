use std::{
    io::{BufWriter, Write},
    mem::size_of,
    num::NonZeroU32,
    path::PathBuf,
};

use bytemuck::offset_of;
use pretzel::Bc1Encoder;
use structopt::StructOpt;
use wgpu::{
    util::DeviceExt, Backends, BufferDescriptor, BufferUsages, CommandEncoderDescriptor, DeviceDescriptor, Extent3d,
    ImageCopyBuffer, ImageDataLayout, Instance, Maintain, MapMode, TextureDescriptor, TextureDimension, TextureFormat,
    TextureUsages, TextureViewDescriptor,
};

#[repr(C)]
#[derive(Debug, Clone, Copy)]
struct Ktx2Header {
    ident: [u8; 12],
    format: u32,
    type_size: u32,
    pixel_width: u32,
    pixel_height: u32,
    pixel_depth: u32,
    layer_count: u32,
    face_count: u32,
    level_count: u32,
    supercompression_scheme: u32,

    dfd_byte_offset: u32,
    dfd_byte_length: u32,
    kvd_byte_offset: u32,
    kvd_byte_length: u32,
    sgd_byte_offset: u64,
    sgd_byte_length: u64,

    byte_offset: u64,
    byte_length: u64,
    uncompressed_byte_length: u64,

    dfd_total_size: u32,

    dfd_byte0: u32,
    dfd_byte1: u32,
    dfd_byte2: u32,
    dfd_byte3: u32,
    dfd_byte4: u32,
    dfd_byte5: u32,

    sample_byte0: u32,
    sample_byte1: u32,
    sample_byte2: u32,
    sample_byte3: u32,
}

unsafe impl bytemuck::Zeroable for Ktx2Header {}
unsafe impl bytemuck::Pod for Ktx2Header {}

#[derive(Debug, StructOpt)]
#[structopt(name = "example", about = "An example of StructOpt usage.")]
struct Opt {
    /// Input image, can be any common image file `image` can parse.
    input: PathBuf,
    /// Destination KTX2 file
    output: PathBuf,
}

pub fn main() {
    env_logger::init();

    let opt = Opt::from_args();
    let backends = Backends::all();
    let instance = Instance::new(backends);
    let adapter = pollster::block_on(wgpu::util::initialize_adapter_from_env_or_default(&instance, backends)).unwrap();
    let (device, queue) = pollster::block_on(adapter.request_device(
        &DeviceDescriptor {
            label: None,
            features: wgpu::Features::SPIRV_SHADER_PASSTHROUGH,
            limits: wgpu::Limits::downlevel_defaults(),
        },
        None,
    ))
    .unwrap();

    log::info!("Creating Encoding Pipelines");
    let bc1 = Bc1Encoder::new(&device);

    log::info!("Loading image {}", opt.input.display());
    let input = image::open(opt.input).expect("could not load image").into_rgba8();
    let width = input.width();
    let height = input.height();

    let size = Extent3d {
        width,
        height,
        depth_or_array_layers: 1,
    };

    log::info!("Uploading image");
    let src_view = device
        .create_texture_with_data(
            &queue,
            &TextureDescriptor {
                label: Some("input image"),
                size,
                mip_level_count: 1,
                sample_count: 1,
                dimension: TextureDimension::D2,
                format: TextureFormat::Rgba8Unorm,
                usage: TextureUsages::TEXTURE_BINDING,
            },
            &input.into_raw(),
        )
        .create_view(&TextureViewDescriptor::default());

    let squash_size = Extent3d {
        width: width / 4,
        height: height / 4,
        depth_or_array_layers: 1,
    };

    log::info!("Creating Resources");
    let dst = device.create_texture(&TextureDescriptor {
        label: Some("dest image"),
        size: Extent3d {
            width: width / 4,
            height: height / 4,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: TextureDimension::D2,
        format: TextureFormat::Rg32Uint,
        usage: TextureUsages::STORAGE_BINDING | TextureUsages::COPY_SRC,
    });

    let buffer_size = squash_size.width as u64 * squash_size.height as u64 * 8;

    let dst_buffer = device.create_buffer(&BufferDescriptor {
        label: Some("dest buffer"),
        size: buffer_size,
        usage: BufferUsages::COPY_DST | BufferUsages::MAP_READ,
        mapped_at_creation: false,
    });

    let dst_view = dst.create_view(&TextureViewDescriptor::default());

    log::info!("Recording");
    let mut encoder = device.create_command_encoder(&CommandEncoderDescriptor::default());

    bc1.execute(&device, &mut encoder, &src_view, &dst_view, size);

    encoder.copy_texture_to_buffer(
        dst.as_image_copy(),
        ImageCopyBuffer {
            buffer: &dst_buffer,
            layout: ImageDataLayout {
                offset: 0,
                bytes_per_row: NonZeroU32::new(size.width * 2),
                rows_per_image: None,
            },
        },
        squash_size,
    );

    log::info!("Submitting");
    queue.submit(Some(encoder.finish()));
    log::info!("Waiting");
    let _ = dst_buffer.slice(..).map_async(MapMode::Read);
    device.poll(Maintain::Wait);

    let data = dst_buffer.slice(..).get_mapped_range();
    log::info!("Writing File");
    let mut writer = BufWriter::new(std::fs::File::create(opt.output).unwrap());

    assert_eq!(data.len(), buffer_size as _);

    let header = Ktx2Header {
        ident: [0xAB, 0x4B, 0x54, 0x58, 0x20, 0x32, 0x30, 0xBB, 0x0D, 0x0A, 0x1A, 0x0A],
        format: 132,
        type_size: 1,
        pixel_width: size.width,
        pixel_height: size.height,
        pixel_depth: 0,
        layer_count: 0,
        face_count: 1,
        level_count: 1,
        supercompression_scheme: 0,

        dfd_byte_offset: 0x68,
        dfd_byte_length: 44,
        kvd_byte_offset: 0,
        kvd_byte_length: 0,
        sgd_byte_offset: 0,
        sgd_byte_length: 0,

        byte_offset: size_of::<Ktx2Header>() as _,
        byte_length: buffer_size,
        uncompressed_byte_length: buffer_size,

        dfd_total_size: 44,

        dfd_byte0: 0x0000 << 17 | 0x0000 << 0,
        dfd_byte1: (24 + 16 * 1) << 16 | 0x0002 << 0,
        dfd_byte2: 0 << 24 | 2 << 16 | 1 << 8 | 128 << 0,
        dfd_byte3: 0 << 24 | 0 << 16 | 3 << 8 | 3 << 0,
        dfd_byte4: 0 << 24 | 0 << 16 | 0 << 8 | 8 << 0,
        dfd_byte5: 0 << 24 | 0 << 16 | 0 << 8 | 0 << 0,

        sample_byte0: 0 << 24 | 63 << 16 | 0 << 0,
        sample_byte1: 0 << 24 | 0 << 16 | 0 << 8 | 0 << 0,
        sample_byte2: 0,
        sample_byte3: u32::MAX,
    };

    writer.write(bytemuck::bytes_of(&header)).unwrap();
    writer.write(&*data).unwrap();
}
