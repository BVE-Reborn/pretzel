use std::{mem::size_of, num::NonZeroU64};

use arrayvec::ArrayVec;
use wgpu::{
    util::{BufferInitDescriptor, DeviceExt},
    BindGroupDescriptor, BindGroupEntry, BindGroupLayout, BindGroupLayoutDescriptor, BindGroupLayoutEntry,
    BindingResource, BindingType, Buffer, BufferBindingType, BufferUsages, CommandEncoder, ComputePassDescriptor,
    ComputePipeline, ComputePipelineDescriptor, Device, Extent3d, PipelineLayoutDescriptor, Sampler, SamplerDescriptor,
    ShaderStages, StorageTextureAccess, TextureFormat, TextureSampleType, TextureView, TextureViewDimension,
};

#[derive(Debug, Copy, Clone)]
struct Bc1Tables {
    match5: [[f32; 2]; 256],
    match6: [[f32; 2]; 256],
}
impl Bc1Tables {
    pub fn new() -> Self {
        let match5 = TABLE_OMATCH5
            .iter()
            .map(|&arr| [arr[0] as f32, arr[1] as f32])
            .collect::<ArrayVec<_, 256>>();
        let match6 = TABLE_OMATCH6
            .iter()
            .map(|&arr| [arr[0] as f32, arr[1] as f32])
            .collect::<ArrayVec<_, 256>>();

        Self {
            match5: match5.into_inner().unwrap(),
            match6: match6.into_inner().unwrap(),
        }
    }
}

unsafe impl bytemuck::Zeroable for Bc1Tables {}
unsafe impl bytemuck::Pod for Bc1Tables {}

#[rustfmt::skip]
static TABLE_OMATCH5: [[u8; 2]; 256] = [
    [0, 0],   [0, 0],   [0, 1],   [0, 1],   [1, 0],   [1, 0],   [1, 0],   [1, 1],
    [1, 1],   [2, 0],   [2, 0],   [0, 4],   [2, 1],   [2, 1],   [2, 1],   [3, 0],
    [3, 0],   [3, 0],   [3, 1],   [1, 5],   [3, 2],   [3, 2],   [4, 0],   [4, 0],
    [4, 1],   [4, 1],   [4, 2],   [4, 2],   [4, 2],   [3, 5],   [5, 1],   [5, 1],
    [5, 2],   [4, 4],   [5, 3],   [5, 3],   [5, 3],   [6, 2],   [6, 2],   [6, 2],
    [6, 3],   [5, 5],   [6, 4],   [6, 4],   [4, 8],   [7, 3],   [7, 3],   [7, 3],
    [7, 4],   [7, 4],   [7, 4],   [7, 5],   [5, 9],   [7, 6],   [7, 6],   [8, 4],
    [8, 4],   [8, 5],   [8, 5],   [8, 6],   [8, 6],   [8, 6],   [7, 9],   [9, 5],
    [9, 5],   [9, 6],   [8, 8],   [9, 7],   [9, 7],   [9, 7],   [10, 6],  [10, 6],
    [10, 6],  [10, 7],  [9, 9],   [10, 8],  [10, 8],  [8, 12],  [11, 7],  [11, 7],
    [11, 7],  [11, 8],  [11, 8],  [11, 8],  [11, 9],  [9, 13],  [11, 10], [11, 10],
    [12, 8],  [12, 8],  [12, 9],  [12, 9],  [12, 10], [12, 10], [12, 10], [11, 13],
    [13, 9],  [13, 9],  [13, 10], [12, 12], [13, 11], [13, 11], [13, 11], [14, 10],
    [14, 10], [14, 10], [14, 11], [13, 13], [14, 12], [14, 12], [12, 16], [15, 11],
    [15, 11], [15, 11], [15, 12], [15, 12], [15, 12], [15, 13], [13, 17], [15, 14],
    [15, 14], [16, 12], [16, 12], [16, 13], [16, 13], [16, 14], [16, 14], [16, 14],
    [15, 17], [17, 13], [17, 13], [17, 14], [16, 16], [17, 15], [17, 15], [17, 15],
    [18, 14], [18, 14], [18, 14], [18, 15], [17, 17], [18, 16], [18, 16], [16, 20],
    [19, 15], [19, 15], [19, 15], [19, 16], [19, 16], [19, 16], [19, 17], [17, 21],
    [19, 18], [19, 18], [20, 16], [20, 16], [20, 17], [20, 17], [20, 18], [20, 18],
    [20, 18], [19, 21], [21, 17], [21, 17], [21, 18], [20, 20], [21, 19], [21, 19],
    [21, 19], [22, 18], [22, 18], [22, 18], [22, 19], [21, 21], [22, 20], [22, 20],
    [20, 24], [23, 19], [23, 19], [23, 19], [23, 20], [23, 20], [23, 20], [23, 21],
    [21, 25], [23, 22], [23, 22], [24, 20], [24, 20], [24, 21], [24, 21], [24, 22],
    [24, 22], [24, 22], [23, 25], [25, 21], [25, 21], [25, 22], [24, 24], [25, 23],
    [25, 23], [25, 23], [26, 22], [26, 22], [26, 22], [26, 23], [25, 25], [26, 24],
    [26, 24], [24, 28], [27, 23], [27, 23], [27, 23], [27, 24], [27, 24], [27, 24],
    [27, 25], [25, 29], [27, 26], [27, 26], [28, 24], [28, 24], [28, 25], [28, 25],
    [28, 26], [28, 26], [28, 26], [27, 29], [29, 25], [29, 25], [29, 26], [28, 28],
    [29, 27], [29, 27], [29, 27], [30, 26], [30, 26], [30, 26], [30, 27], [29, 29],
    [30, 28], [30, 28], [30, 28], [31, 27], [31, 27], [31, 27], [31, 28], [31, 28],
    [31, 28], [31, 29], [31, 29], [31, 30], [31, 30], [31, 30], [31, 31], [31, 31]
];

#[rustfmt::skip]
static TABLE_OMATCH6: [[u8; 2]; 256] = [
     [0, 0],   [0, 1],   [1, 0],   [1, 0],   [1, 1],   [2, 0],   [2, 1],   [3, 0],
     [3, 0],   [3, 1],   [4, 0],   [4, 0],   [4, 1],   [5, 0],   [5, 1],   [6, 0],
     [6, 0],   [6, 1],   [7, 0],   [7, 0],   [7, 1],   [8, 0],   [8, 1],   [8, 1],
     [8, 2],   [9, 1],   [9, 2],   [9, 2],   [9, 3],   [10, 2],  [10, 3],  [10, 3],
     [10, 4],  [11, 3],  [11, 4],  [11, 4],  [11, 5],  [12, 4],  [12, 5],  [12, 5],
     [12, 6],  [13, 5],  [13, 6],  [8, 16],  [13, 7],  [14, 6],  [14, 7],  [9, 17],
     [14, 8],  [15, 7],  [15, 8],  [11, 16], [15, 9],  [15, 10], [16, 8],  [16, 9],
     [16, 10], [15, 13], [17, 9],  [17, 10], [17, 11], [15, 16], [18, 10], [18, 11],
     [18, 12], [16, 16], [19, 11], [19, 12], [19, 13], [17, 17], [20, 12], [20, 13],
     [20, 14], [19, 16], [21, 13], [21, 14], [21, 15], [20, 17], [22, 14], [22, 15],
     [25, 10], [22, 16], [23, 15], [23, 16], [26, 11], [23, 17], [24, 16], [24, 17],
     [27, 12], [24, 18], [25, 17], [25, 18], [28, 13], [25, 19], [26, 18], [26, 19],
     [29, 14], [26, 20], [27, 19], [27, 20], [30, 15], [27, 21], [28, 20], [28, 21],
     [28, 21], [28, 22], [29, 21], [29, 22], [24, 32], [29, 23], [30, 22], [30, 23],
     [25, 33], [30, 24], [31, 23], [31, 24], [27, 32], [31, 25], [31, 26], [32, 24],
     [32, 25], [32, 26], [31, 29], [33, 25], [33, 26], [33, 27], [31, 32], [34, 26],
     [34, 27], [34, 28], [32, 32], [35, 27], [35, 28], [35, 29], [33, 33], [36, 28],
     [36, 29], [36, 30], [35, 32], [37, 29], [37, 30], [37, 31], [36, 33], [38, 30],
     [38, 31], [41, 26], [38, 32], [39, 31], [39, 32], [42, 27], [39, 33], [40, 32],
     [40, 33], [43, 28], [40, 34], [41, 33], [41, 34], [44, 29], [41, 35], [42, 34],
     [42, 35], [45, 30], [42, 36], [43, 35], [43, 36], [46, 31], [43, 37], [44, 36],
     [44, 37], [44, 37], [44, 38], [45, 37], [45, 38], [40, 48], [45, 39], [46, 38],
     [46, 39], [41, 49], [46, 40], [47, 39], [47, 40], [43, 48], [47, 41], [47, 42],
     [48, 40], [48, 41], [48, 42], [47, 45], [49, 41], [49, 42], [49, 43], [47, 48],
     [50, 42], [50, 43], [50, 44], [48, 48], [51, 43], [51, 44], [51, 45], [49, 49],
     [52, 44], [52, 45], [52, 46], [51, 48], [53, 45], [53, 46], [53, 47], [52, 49],
     [54, 46], [54, 47], [57, 42], [54, 48], [55, 47], [55, 48], [58, 43], [55, 49],
     [56, 48], [56, 49], [59, 44], [56, 50], [57, 49], [57, 50], [60, 45], [57, 51],
     [58, 50], [58, 51], [61, 46], [58, 52], [59, 51], [59, 52], [62, 47], [59, 53],
     [60, 52], [60, 53], [60, 53], [60, 54], [61, 53], [61, 54], [61, 54], [61, 55],
     [62, 54], [62, 55], [62, 55], [62, 56], [63, 55], [63, 56], [63, 56], [63, 57],
     [63, 58], [63, 59], [63, 59], [63, 60], [63, 61], [63, 62], [63, 62], [63, 63]
];

pub struct Bc1Encoder {
    sampler: Sampler,
    bgl: BindGroupLayout,
    pipeline: ComputePipeline,
    table_buffer: Buffer,
}
impl Bc1Encoder {
    pub fn new(device: &Device) -> Self {
        let bgl = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("BC1 bgl"),
            entries: &[
                BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStages::COMPUTE,
                    ty: BindingType::Texture {
                        sample_type: TextureSampleType::Float { filterable: false },
                        view_dimension: TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: 1,
                    visibility: ShaderStages::COMPUTE,
                    ty: BindingType::Sampler {
                        comparison: false,
                        filtering: false,
                    },
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: 2,
                    visibility: ShaderStages::COMPUTE,
                    ty: BindingType::StorageTexture {
                        access: StorageTextureAccess::WriteOnly,
                        format: TextureFormat::Rg32Float,
                        view_dimension: TextureViewDimension::D2,
                    },
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: 3,
                    visibility: ShaderStages::COMPUTE,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: NonZeroU64::new(size_of::<Bc1Tables>() as _),
                    },
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: 4,
                    visibility: ShaderStages::COMPUTE,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: NonZeroU64::new(size_of::<u32>() as _),
                    },
                    count: None,
                },
            ],
        });

        let sampler = device.create_sampler(&SamplerDescriptor::default());

        let table = Bc1Tables::new();

        let table_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("bc1 table buffer"),
            contents: bytemuck::bytes_of(&table),
            usage: BufferUsages::STORAGE,
        });

        let sm = unsafe { device.create_shader_module_spirv(&wgpu::include_spirv_raw!("../shaders/spirv/bc1.spv")) };

        let pll = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: Some("bc1 pll"),
            bind_group_layouts: &[&bgl],
            push_constant_ranges: &[],
        });

        let pipeline = device.create_compute_pipeline(&ComputePipelineDescriptor {
            label: Some("bc1 pl"),
            layout: Some(&pll),
            module: &sm,
            entry_point: "main",
        });

        Self {
            bgl,
            pipeline,
            table_buffer,
            sampler,
        }
    }

    pub fn execute(
        &self,
        device: &Device,
        encoder: &mut CommandEncoder,
        source: &TextureView,
        dest: &TextureView,
        size: Extent3d,
    ) {
        let refinements = 2_u32;
        let refinement_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("bc1 refinement buffer"),
            contents: bytemuck::bytes_of(&refinements),
            usage: BufferUsages::UNIFORM,
        });

        let bg = device.create_bind_group(&BindGroupDescriptor {
            label: Some("bc1 bg"),
            layout: &self.bgl,
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: BindingResource::TextureView(source),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: BindingResource::Sampler(&self.sampler),
                },
                BindGroupEntry {
                    binding: 2,
                    resource: BindingResource::TextureView(dest),
                },
                BindGroupEntry {
                    binding: 3,
                    resource: BindingResource::Buffer(self.table_buffer.as_entire_buffer_binding()),
                },
                BindGroupEntry {
                    binding: 4,
                    resource: BindingResource::Buffer(refinement_buffer.as_entire_buffer_binding()),
                },
            ],
        });

        let mut cpass = encoder.begin_compute_pass(&ComputePassDescriptor {
            label: Some("bc1 encode"),
        });

        cpass.set_pipeline(&self.pipeline);
        cpass.set_bind_group(0, &bg, &[]);
        cpass.dispatch((size.width + 31) & !31, (size.height + 31) & !31, 1);

        drop(cpass);
    }
}
