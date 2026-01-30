use criterion::{black_box, criterion_group, criterion_main, Criterion};
use abdf_builder::{AbdfBuilder, decode_abdf};
use abdf::segment::{SegmentKind, MetaContainer};

fn benchmark_abdf_build(c: &mut Criterion) {
    c.bench_function("abdf_build_small", |b| {
        b.iter(|| {
            let mut builder = AbdfBuilder::new();
            
            let name_idx = builder.intern_string("test_data");
            let type_idx = builder.intern_string("table/generic");
            let schema_idx = builder.intern_string("id:u64,value:f64");
            
            let meta = MetaContainer {
                name_idx,
                type_idx,
                schema_idx,
                permissions: 0,
                embedding_idx: 0,
            };
            
            let data = vec![0u8; 1024]; // 1KB data
            builder.add_segment(SegmentKind::Tabular(meta), &data);
            
            black_box(builder.build())
        })
    });
}

fn benchmark_abdf_decode(c: &mut Criterion) {
    // Prepare test data
    let mut builder = AbdfBuilder::new();
    let name_idx = builder.intern_string("test_data");
    let type_idx = builder.intern_string("table/generic");
    let schema_idx = builder.intern_string("id:u64,value:f64");
    
    let meta = MetaContainer {
        name_idx,
        type_idx,
        schema_idx,
        permissions: 0,
        embedding_idx: 0,
    };
    
    let data = vec![0u8; 1024];
    builder.add_segment(SegmentKind::Tabular(meta), &data);
    let buffer = builder.build();
    
    c.bench_function("abdf_decode_small", |b| {
        b.iter(|| {
            black_box(decode_abdf(&buffer).unwrap())
        })
    });
}

criterion_group!(benches, benchmark_abdf_build, benchmark_abdf_decode);
criterion_main!(benches);