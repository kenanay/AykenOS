# Runtime Integration Guide

## Overview

Bu dokuman, Ayken Core formatlarının (ABDF ve BCIB) runtime sistemlere nasıl entegre edileceğini açıklar.

## ABDF Runtime Integration

### Memory Management

```rust
// Zero-copy data access
let view = decode_abdf(&buffer)?;
let segment_data = view.segment_data(0)?; // No allocation

// Memory mapping for large files
use memmap2::MmapOptions;
let mmap = unsafe { MmapOptions::new().map(&file)? };
let view = decode_abdf(&mmap)?;
```

### GPU Integration

```rust
// GPU buffer creation from ABDF segment
let gpu_data = view.segment_data(gpu_segment_idx)?;
let gpu_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
    label: Some("ABDF GPU Buffer"),
    contents: gpu_data,
    usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
});
```

### Streaming Support

```rust
// Streaming decoder for large datasets
struct AbdfStreamer {
    reader: BufReader<File>,
    header: AbdfHeader,
    current_segment: usize,
}

impl AbdfStreamer {
    fn next_segment(&mut self) -> Option<Vec<u8>> {
        // Stream next segment without loading entire file
    }
}
```

## BCIB Runtime Integration

### Instruction Executor

```rust
pub struct BcibExecutor {
    context: ExecutionContext,
    abdf_data: HashMap<u32, AbdfView<'static>>,
}

impl BcibExecutor {
    pub fn execute(&mut self, buffer: &[u8]) -> Result<(), ExecutionError> {
        let header = self.parse_header(buffer)?;
        let instructions = self.parse_instructions(buffer, &header)?;
        
        for instruction in instructions {
            self.execute_instruction(instruction)?;
        }
        
        Ok(())
    }
    
    fn execute_instruction(&mut self, instr: BcibInstruction) -> Result<(), ExecutionError> {
        match instr.opcode {
            BcibOpcode::CtxSelect => self.select_context(instr.arg0),
            BcibOpcode::DataQuery => self.query_data(instr.arg0),
            BcibOpcode::UiRender => self.render_ui(instr.arg0),
            // ... other opcodes
        }
    }
}
```

### Context Management

```rust
pub struct ExecutionContext {
    active_container: Option<u32>,
    string_pool: Vec<String>,
    ui_state: UiState,
    ai_modules: HashMap<String, AiModule>,
}

impl ExecutionContext {
    pub fn select_container(&mut self, container_id: u32) {
        self.active_container = Some(container_id);
    }
    
    pub fn get_string(&self, idx: u32) -> Option<&str> {
        self.string_pool.get(idx as usize).map(|s| s.as_str())
    }
}
```

## Performance Considerations

### Memory Layout

```rust
// Align data structures for optimal cache performance
#[repr(C, align(64))] // Cache line alignment
struct OptimizedSegmentDescriptor {
    meta_idx: u32,
    offset: u64,
    length: u64,
    _padding: [u8; 44], // Fill cache line
}
```

### SIMD Operations

```rust
// Vectorized operations on ABDF data
use std::simd::*;

fn process_f32_vector(data: &[f32]) -> Vec<f32> {
    let chunks = data.chunks_exact(8);
    let mut result = Vec::with_capacity(data.len());
    
    for chunk in chunks {
        let simd_chunk = f32x8::from_slice(chunk);
        let processed = simd_chunk * f32x8::splat(2.0); // Example operation
        result.extend_from_slice(processed.as_array());
    }
    
    result
}
```

### Async Processing

```rust
use tokio::task;

pub async fn process_abdf_async(buffer: Vec<u8>) -> Result<ProcessedData, Error> {
    // Decode in background thread
    let view = task::spawn_blocking(move || {
        decode_abdf(&buffer)
    }).await??;
    
    // Process segments concurrently
    let mut tasks = Vec::new();
    for i in 0..view.segments.len() {
        let segment_data = view.segment_data(i).unwrap().to_vec();
        tasks.push(task::spawn(async move {
            process_segment(segment_data).await
        }));
    }
    
    // Collect results
    let results = futures::future::join_all(tasks).await;
    Ok(ProcessedData::from_results(results))
}
```

## Error Handling

### Graceful Degradation

```rust
pub enum RuntimeError {
    InvalidFormat,
    UnsupportedVersion,
    CorruptedData,
    OutOfMemory,
    ExecutionTimeout,
}

impl BcibExecutor {
    fn handle_error(&mut self, error: RuntimeError) -> RecoveryAction {
        match error {
            RuntimeError::UnsupportedVersion => RecoveryAction::Fallback,
            RuntimeError::CorruptedData => RecoveryAction::Skip,
            RuntimeError::OutOfMemory => RecoveryAction::Cleanup,
            _ => RecoveryAction::Abort,
        }
    }
}
```

## Integration Patterns

### Plugin Architecture

```rust
pub trait FormatPlugin {
    fn can_handle(&self, magic: &[u8; 4]) -> bool;
    fn decode(&self, buffer: &[u8]) -> Result<Box<dyn DataView>, Error>;
}

pub struct AbdfPlugin;

impl FormatPlugin for AbdfPlugin {
    fn can_handle(&self, magic: &[u8; 4]) -> bool {
        magic == b"ABDF"
    }
    
    fn decode(&self, buffer: &[u8]) -> Result<Box<dyn DataView>, Error> {
        let view = decode_abdf(buffer)?;
        Ok(Box::new(view))
    }
}
```

### Event-Driven Processing

```rust
pub enum RuntimeEvent {
    SegmentLoaded(u32),
    InstructionExecuted(BcibOpcode),
    ContextChanged(u32),
    Error(RuntimeError),
}

pub trait EventHandler {
    fn handle_event(&mut self, event: RuntimeEvent);
}
```

## Best Practices

1. **Use memory mapping** for large files
2. **Implement streaming** for real-time data
3. **Cache frequently accessed** segments
4. **Validate data integrity** before processing
5. **Handle version compatibility** gracefully
6. **Profile memory usage** regularly
7. **Use SIMD** for bulk operations
8. **Implement proper error recovery**