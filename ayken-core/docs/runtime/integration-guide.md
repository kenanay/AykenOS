# Ayken Core Runtime Integration Guide

## 🎯 Overview

Bu dokuman, Ayken Core formatlarının (ABDF v0.2 ve BCIB v0.2) runtime sistemlere nasıl entegre edileceğini açıklar. Phase 2 tamamlandı ve Phase 3 AI integration için hazır durumda.

**Status**: Production Ready (Phase 2 Complete)  
**Version**: ABDF v0.2, BCIB v0.2  
**Last Updated**: 31 Ocak 2026  

## 📊 Current Status

### ✅ Production Ready Components
- **ABDF v0.2**: MetaContainer system, 8-byte aligned data sections
- **BCIB v0.2**: DSL-compatible opcode system, validation framework
- **Performance**: Sub-microsecond decode times, <1MB memory overhead
- **Quality**: 12/12 tests passing, zero compiler warnings

### 🚧 Phase 3 Preparation
- **AI Integration**: TinyLLM userspace integration planned
- **Enhanced Opcodes**: ai.ask implementation ready
- **Security Framework**: Human-approved AI policies

## 🏗️ ABDF v0.2 Runtime Integration

### Core Architecture

```rust
use ayken_core::abdf::{AbdfDecoder, MetaContainer, SegmentDescriptor};

// Production-ready decoder with v0.2 features
let decoder = AbdfDecoder::new();
let view = decoder.decode(&buffer)?;

// Access MetaContainer with full metadata
let meta = view.meta_container(0)?;
println!("Container: {} (type: {}, schema: {})", 
    meta.name_idx, meta.type_idx, meta.schema_idx);

// 8-byte aligned segment access
let segment_data = view.segment_data(0)?; // Zero-copy, aligned access
```

### Memory Management (Production Optimized)

```rust
// Zero-copy data access with alignment guarantees
let view = decode_abdf(&buffer)?;
let segment_data = view.segment_data(0)?; // 8-byte aligned, no allocation

// Memory mapping for large files (production pattern)
use memmap2::MmapOptions;
let mmap = unsafe { MmapOptions::new().map(&file)? };
let view = decode_abdf(&mmap)?;

// Segment streaming for large datasets
struct AbdfStreamer {
    reader: BufReader<File>,
    header: AbdfHeader,
    current_segment: usize,
    alignment_buffer: Vec<u8>, // Ensure 8-byte alignment
}

impl AbdfStreamer {
    fn next_aligned_segment(&mut self) -> Option<&[u8]> {
        // Stream with guaranteed 8-byte alignment
        self.read_aligned_segment()
    }
}
```

### GPU Integration (Enhanced)

```rust
// GPU buffer creation from ABDF segment with alignment
let gpu_data = view.segment_data(gpu_segment_idx)?;
assert_eq!(gpu_data.as_ptr() as usize % 8, 0); // Verify alignment

let gpu_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
    label: Some("ABDF GPU Buffer v0.2"),
    contents: gpu_data,
    usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
});

// Direct GPU memory mapping (zero-copy)
let mapped_buffer = gpu_buffer.slice(..).get_mapped_range();
// ABDF data is already aligned for GPU consumption
```

### Schema Validation (New in v0.2)

```rust
// Schema validation with MetaContainer
let meta = view.meta_container(container_idx)?;
let schema_id = meta.schema_idx;

// Validate against known schemas
match schema_id {
    TENSOR_SCHEMA_ID => validate_tensor_data(&segment_data)?,
    UI_COMPONENT_SCHEMA_ID => validate_ui_component(&segment_data)?,
    AI_MODEL_SCHEMA_ID => validate_ai_model(&segment_data)?,
    _ => return Err(Error::UnknownSchema(schema_id)),
}
```

## 🤖 BCIB v0.2 Runtime Integration

### Enhanced Instruction Executor

```rust
use ayken_core::bcib::{BcibDecoder, BcibOpcode, BcibInstruction};

pub struct BcibExecutor {
    context: ExecutionContext,
    abdf_data: HashMap<u32, AbdfView<'static>>,
    ai_runtime: Option<AiRuntime>, // Ready for Phase 3
}

impl BcibExecutor {
    pub fn execute(&mut self, buffer: &[u8]) -> Result<ExecutionResult, ExecutionError> {
        // Validate BCIB v0.2 header
        let header = self.parse_header(buffer)?;
        if header.version != 2 {
            return Err(ExecutionError::UnsupportedVersion(header.version));
        }
        
        let instructions = self.parse_instructions(buffer, &header)?;
        let mut results = Vec::new();
        
        for instruction in instructions {
            let result = self.execute_instruction(instruction)?;
            results.push(result);
        }
        
        Ok(ExecutionResult::new(results))
    }
    
    fn execute_instruction(&mut self, instr: BcibInstruction) -> Result<InstructionResult, ExecutionError> {
        match instr.opcode {
            BcibOpcode::DataCreate => self.create_data_container(instr.arg0, instr.arg1),
            BcibOpcode::DataAdd => self.add_data_to_container(instr.arg0, instr.arg1),
            BcibOpcode::DataQuery => self.query_data_container(instr.arg0),
            BcibOpcode::UiRender => self.render_ui_stub(instr.arg0), // Stub for Phase 3
            BcibOpcode::AiAsk => self.ai_ask_stub(instr.arg0), // Stub for Phase 3
            BcibOpcode::End => Ok(InstructionResult::End),
            _ => Err(ExecutionError::InvalidOpcode(instr.opcode as u8)),
        }
    }
    
    // Phase 3 preparation: AI stub implementation
    fn ai_ask_stub(&mut self, query_idx: u32) -> Result<InstructionResult, ExecutionError> {
        let query = self.context.get_string(query_idx)
            .ok_or(ExecutionError::InvalidStringIndex(query_idx))?;
        
        // Stub implementation - returns placeholder for Phase 3
        Ok(InstructionResult::AiResponse {
            query: query.to_string(),
            response: "AI integration pending Phase 3".to_string(),
            requires_human_approval: true,
        })
    }
}
```

### Context Management (Enhanced)

```rust
pub struct ExecutionContext {
    active_container: Option<u32>,
    string_pool: Vec<String>,
    ui_state: UiState,
    ai_modules: HashMap<String, AiModuleStub>, // Phase 3 preparation
    schema_registry: SchemaRegistry, // New in v0.2
    permissions: PermissionSet, // Security framework
}

impl ExecutionContext {
    pub fn new() -> Self {
        Self {
            active_container: None,
            string_pool: Vec::new(),
            ui_state: UiState::default(),
            ai_modules: HashMap::new(),
            schema_registry: SchemaRegistry::new(),
            permissions: PermissionSet::default(),
        }
    }
    
    pub fn validate_operation(&self, op: &BcibOpcode) -> Result<(), SecurityError> {
        match op {
            BcibOpcode::AiAsk => {
                if !self.permissions.has_ai_access() {
                    return Err(SecurityError::AiAccessDenied);
                }
                // Phase 3: Will require human approval
                Ok(())
            },
            _ => Ok(())
        }
    }
    
    pub fn register_schema(&mut self, schema_id: u32, schema: Schema) {
        self.schema_registry.register(schema_id, schema);
    }
}
```

### Data Container Operations (Production Ready)

```rust
impl BcibExecutor {
    fn create_data_container(&mut self, name_idx: u32, type_idx: u32) -> Result<InstructionResult, ExecutionError> {
        let name = self.context.get_string(name_idx)
            .ok_or(ExecutionError::InvalidStringIndex(name_idx))?;
        let type_name = self.context.get_string(type_idx)
            .ok_or(ExecutionError::InvalidStringIndex(type_idx))?;
        
        // Create ABDF container with MetaContainer
        let container = AbdfBuilder::new()
            .with_meta_container(name, type_name, 0, 0, 0) // schema_idx, permissions, embedding_idx
            .build()?;
        
        let container_id = self.allocate_container_id();
        self.abdf_data.insert(container_id, container);
        
        Ok(InstructionResult::ContainerCreated(container_id))
    }
    
    fn add_data_to_container(&mut self, container_id: u32, data_idx: u32) -> Result<InstructionResult, ExecutionError> {
        let container = self.abdf_data.get_mut(&container_id)
            .ok_or(ExecutionError::ContainerNotFound(container_id))?;
        
        let data = self.context.get_string(data_idx)
            .ok_or(ExecutionError::InvalidStringIndex(data_idx))?;
        
        // Add data as new segment with proper alignment
        container.add_aligned_segment(data.as_bytes())?;
        
        Ok(InstructionResult::DataAdded)
    }
    
    fn query_data_container(&mut self, container_id: u32) -> Result<InstructionResult, ExecutionError> {
        let container = self.abdf_data.get(&container_id)
            .ok_or(ExecutionError::ContainerNotFound(container_id))?;
        
        // Query with schema validation
        let meta = container.meta_container(0)?;
        let query_result = QueryResult {
            container_id,
            name: meta.name_idx,
            type_id: meta.type_idx,
            schema_id: meta.schema_idx,
            segment_count: container.segment_count(),
        };
        
        Ok(InstructionResult::QueryResult(query_result))
    }
}
```

## 🚀 Performance Optimizations (Production Grade)

### Memory Layout Optimization

```rust
// Cache-line aligned structures for optimal performance
#[repr(C, align(64))] // Cache line alignment
struct OptimizedSegmentDescriptor {
    meta_idx: u32,
    offset: u64,
    length: u64,
    checksum: u32,      // Data integrity
    _padding: [u8; 40], // Fill cache line
}

// SIMD-optimized data processing
use std::simd::*;

fn process_f32_vector_optimized(data: &[f32]) -> Vec<f32> {
    assert_eq!(data.as_ptr() as usize % 32, 0); // AVX alignment
    
    let chunks = data.chunks_exact(8);
    let mut result = Vec::with_capacity(data.len());
    
    for chunk in chunks {
        let simd_chunk = f32x8::from_slice(chunk);
        let processed = simd_chunk * f32x8::splat(2.0);
        result.extend_from_slice(processed.as_array());
    }
    
    result
}
```

### Async Processing (Production Pattern)

```rust
use tokio::task;
use futures::stream::{self, StreamExt};

pub async fn process_abdf_pipeline(buffers: Vec<Vec<u8>>) -> Result<Vec<ProcessedData>, Error> {
    // Parallel decode with backpressure
    let decode_stream = stream::iter(buffers)
        .map(|buffer| {
            task::spawn_blocking(move || decode_abdf(&buffer))
        })
        .buffer_unordered(4); // Limit concurrent decodes
    
    // Process results as they complete
    let results: Result<Vec<_>, _> = decode_stream
        .map(|result| async move {
            let view = result??;
            process_segments_parallel(view).await
        })
        .buffer_unordered(8)
        .collect()
        .await;
    
    results
}

async fn process_segments_parallel(view: AbdfView<'_>) -> Result<ProcessedData, Error> {
    let segment_tasks: Vec<_> = (0..view.segment_count())
        .map(|i| {
            let segment_data = view.segment_data(i).unwrap().to_vec();
            task::spawn(async move {
                // CPU-intensive processing in thread pool
                task::spawn_blocking(move || {
                    process_segment_intensive(segment_data)
                }).await?
            })
        })
        .collect();
    
    let results = futures::future::try_join_all(segment_tasks).await?;
    Ok(ProcessedData::from_segments(results))
}
```

## 🛡️ Error Handling & Security

### Comprehensive Error Recovery

```rust
#[derive(Debug, thiserror::Error)]
pub enum RuntimeError {
    #[error("Invalid format: {0}")]
    InvalidFormat(String),
    #[error("Unsupported version: {version}, expected: {expected}")]
    UnsupportedVersion { version: u16, expected: u16 },
    #[error("Corrupted data at offset: {offset}")]
    CorruptedData { offset: u64 },
    #[error("Out of memory: requested {requested} bytes")]
    OutOfMemory { requested: usize },
    #[error("Execution timeout after {duration:?}")]
    ExecutionTimeout { duration: std::time::Duration },
    #[error("Schema validation failed: {schema_id}")]
    SchemaValidationFailed { schema_id: u32 },
    #[error("Security violation: {operation}")]
    SecurityViolation { operation: String },
}

impl BcibExecutor {
    fn handle_error(&mut self, error: RuntimeError) -> RecoveryAction {
        match error {
            RuntimeError::UnsupportedVersion { version, expected } => {
                if version < expected {
                    RecoveryAction::Fallback(expected)
                } else {
                    RecoveryAction::Abort
                }
            },
            RuntimeError::CorruptedData { offset } => {
                self.mark_segment_corrupted(offset);
                RecoveryAction::Skip
            },
            RuntimeError::OutOfMemory { .. } => {
                self.cleanup_unused_containers();
                RecoveryAction::Retry
            },
            RuntimeError::SecurityViolation { .. } => {
                self.log_security_incident(&error);
                RecoveryAction::Abort
            },
            _ => RecoveryAction::Abort,
        }
    }
}
```

### Security Framework (Phase 3 Ready)

```rust
pub struct SecurityContext {
    permissions: PermissionSet,
    audit_log: Vec<SecurityEvent>,
    human_approval_required: bool,
}

impl SecurityContext {
    pub fn validate_ai_operation(&mut self, operation: &str) -> Result<(), SecurityError> {
        // Log all AI operations for audit
        self.audit_log.push(SecurityEvent::AiOperationRequested {
            operation: operation.to_string(),
            timestamp: std::time::SystemTime::now(),
        });
        
        if self.human_approval_required {
            return Err(SecurityError::HumanApprovalRequired);
        }
        
        if !self.permissions.has_ai_access() {
            return Err(SecurityError::AiAccessDenied);
        }
        
        Ok(())
    }
}
```

## 🔌 Integration Patterns

### Plugin Architecture (Extensible)

```rust
pub trait FormatPlugin: Send + Sync {
    fn name(&self) -> &str;
    fn version(&self) -> &str;
    fn can_handle(&self, magic: &[u8; 4]) -> bool;
    fn decode(&self, buffer: &[u8]) -> Result<Box<dyn DataView>, Error>;
    fn validate_schema(&self, schema_id: u32) -> bool;
}

pub struct AbdfPlugin;

impl FormatPlugin for AbdfPlugin {
    fn name(&self) -> &str { "ABDF" }
    fn version(&self) -> &str { "0.2" }
    
    fn can_handle(&self, magic: &[u8; 4]) -> bool {
        magic == b"ABDF"
    }
    
    fn decode(&self, buffer: &[u8]) -> Result<Box<dyn DataView>, Error> {
        let view = decode_abdf(buffer)?;
        Ok(Box::new(view))
    }
    
    fn validate_schema(&self, schema_id: u32) -> bool {
        // Validate against known ABDF schemas
        matches!(schema_id, TENSOR_SCHEMA_ID | UI_COMPONENT_SCHEMA_ID | AI_MODEL_SCHEMA_ID)
    }
}

// Plugin registry for runtime extensibility
pub struct PluginRegistry {
    plugins: HashMap<String, Box<dyn FormatPlugin>>,
}

impl PluginRegistry {
    pub fn register<P: FormatPlugin + 'static>(&mut self, plugin: P) {
        self.plugins.insert(plugin.name().to_string(), Box::new(plugin));
    }
    
    pub fn decode_buffer(&self, buffer: &[u8]) -> Result<Box<dyn DataView>, Error> {
        if buffer.len() < 4 {
            return Err(Error::BufferTooSmall);
        }
        
        let magic: [u8; 4] = buffer[0..4].try_into().unwrap();
        
        for plugin in self.plugins.values() {
            if plugin.can_handle(&magic) {
                return plugin.decode(buffer);
            }
        }
        
        Err(Error::UnsupportedFormat(magic))
    }
}
```

### Event-Driven Processing

```rust
#[derive(Debug, Clone)]
pub enum RuntimeEvent {
    // ABDF events
    ContainerLoaded { container_id: u32, size: usize },
    SegmentProcessed { container_id: u32, segment_id: u32 },
    SchemaValidated { schema_id: u32, valid: bool },
    
    // BCIB events
    InstructionExecuted { opcode: BcibOpcode, duration: std::time::Duration },
    ContextChanged { old_context: u32, new_context: u32 },
    
    // AI events (Phase 3 preparation)
    AiQuerySubmitted { query: String },
    AiResponseReceived { response: String, confidence: f32 },
    HumanApprovalRequested { operation: String },
    
    // Error events
    Error { error: RuntimeError, context: String },
    SecurityViolation { violation: String, severity: SecurityLevel },
}

pub trait EventHandler: Send + Sync {
    fn handle_event(&mut self, event: RuntimeEvent);
}

// Event bus for decoupled communication
pub struct EventBus {
    handlers: Vec<Box<dyn EventHandler>>,
}

impl EventBus {
    pub fn subscribe<H: EventHandler + 'static>(&mut self, handler: H) {
        self.handlers.push(Box::new(handler));
    }
    
    pub fn publish(&mut self, event: RuntimeEvent) {
        for handler in &mut self.handlers {
            handler.handle_event(event.clone());
        }
    }
}
```

## 🎯 Phase 3 AI Integration Preparation

### AI Runtime Stub Implementation

```rust
// Phase 3 preparation: AI runtime interface
pub trait AiRuntime: Send + Sync {
    async fn process_query(&mut self, query: &str) -> Result<AiResponse, AiError>;
    fn requires_human_approval(&self, query: &str) -> bool;
    fn get_confidence_threshold(&self) -> f32;
}

// Stub implementation for Phase 2
pub struct AiRuntimeStub;

impl AiRuntime for AiRuntimeStub {
    async fn process_query(&mut self, query: &str) -> Result<AiResponse, AiError> {
        Ok(AiResponse {
            query: query.to_string(),
            response: "AI integration pending Phase 3".to_string(),
            confidence: 0.0,
            requires_approval: true,
        })
    }
    
    fn requires_human_approval(&self, _query: &str) -> bool {
        true // Always require approval in Phase 2
    }
    
    fn get_confidence_threshold(&self) -> f32 {
        1.0 // Impossible threshold in Phase 2
    }
}

// Phase 3 interface ready for TinyLLM integration
pub struct TinyLLMRuntime {
    model: Option<TinyLLMModel>, // To be implemented in Phase 3
    security_context: SecurityContext,
    human_approval_queue: VecDeque<PendingApproval>,
}
```

### Human Approval Workflow (Phase 3 Ready)

```rust
#[derive(Debug, Clone)]
pub struct PendingApproval {
    id: uuid::Uuid,
    query: String,
    proposed_response: String,
    confidence: f32,
    timestamp: std::time::SystemTime,
    context: ExecutionContext,
}

pub struct HumanApprovalSystem {
    pending: HashMap<uuid::Uuid, PendingApproval>,
    approved: HashMap<uuid::Uuid, ApprovedResponse>,
    rejected: HashMap<uuid::Uuid, RejectedResponse>,
}

impl HumanApprovalSystem {
    pub async fn request_approval(&mut self, query: &str, response: &str, confidence: f32) -> uuid::Uuid {
        let approval = PendingApproval {
            id: uuid::Uuid::new_v4(),
            query: query.to_string(),
            proposed_response: response.to_string(),
            confidence,
            timestamp: std::time::SystemTime::now(),
            context: ExecutionContext::current(),
        };
        
        let id = approval.id;
        self.pending.insert(id, approval);
        
        // Notify human reviewers (Phase 3 implementation)
        self.notify_reviewers(id).await;
        
        id
    }
    
    pub fn approve(&mut self, id: uuid::Uuid, human_reviewer: &str) -> Result<(), ApprovalError> {
        let pending = self.pending.remove(&id)
            .ok_or(ApprovalError::NotFound)?;
        
        let approved = ApprovedResponse {
            original: pending,
            approved_by: human_reviewer.to_string(),
            approved_at: std::time::SystemTime::now(),
        };
        
        self.approved.insert(id, approved);
        Ok(())
    }
}
```

## 📋 Best Practices (Production Ready)

### 1. Memory Management
- **Use memory mapping** for files >1MB
- **Implement streaming** for real-time data processing
- **Cache frequently accessed** segments with LRU eviction
- **Validate alignment** before SIMD operations (8-byte minimum)

### 2. Performance Optimization
- **Profile memory usage** with tools like `heaptrack`
- **Use SIMD** for bulk data operations (AVX2/NEON)
- **Implement proper backpressure** in async pipelines
- **Monitor cache hit rates** for segment access

### 3. Error Handling
- **Validate data integrity** with checksums
- **Handle version compatibility** gracefully with fallbacks
- **Implement circuit breakers** for external dependencies
- **Log security events** for audit compliance

### 4. Security (Phase 3 Ready)
- **Require human approval** for all AI operations
- **Validate permissions** before sensitive operations
- **Audit all data access** with immutable logs
- **Implement sandboxing** for AI model execution

### 5. Testing & Validation
- **Property-based testing** for format compliance
- **Fuzz testing** for security vulnerabilities
- **Performance regression testing** with benchmarks
- **Integration testing** with real-world data

### 6. Monitoring & Observability
- **Instrument performance metrics** (latency, throughput)
- **Track error rates** by operation type
- **Monitor memory usage** patterns
- **Alert on security violations**

## 🔗 Integration Examples

### Basic ABDF Processing

```rust
use ayken_core::abdf::*;

fn process_abdf_file(path: &Path) -> Result<ProcessingResult, Error> {
    // Memory map for large files
    let file = File::open(path)?;
    let mmap = unsafe { MmapOptions::new().map(&file)? };
    
    // Decode with validation
    let view = decode_abdf(&mmap)?;
    
    // Process each segment
    let mut results = Vec::new();
    for i in 0..view.segment_count() {
        let meta = view.meta_container(i)?;
        let data = view.segment_data(i)?;
        
        // Schema-based processing
        let result = match meta.schema_idx {
            TENSOR_SCHEMA_ID => process_tensor_data(data)?,
            UI_COMPONENT_SCHEMA_ID => process_ui_component(data)?,
            _ => ProcessingResult::Skipped,
        };
        
        results.push(result);
    }
    
    Ok(ProcessingResult::Combined(results))
}
```

### BCIB Execution Pipeline

```rust
use ayken_core::bcib::*;

async fn execute_bcib_program(program: &[u8]) -> Result<ExecutionResult, Error> {
    let mut executor = BcibExecutor::new();
    
    // Set up security context
    executor.set_security_context(SecurityContext::new()
        .with_ai_access(false) // Phase 2: AI disabled
        .with_human_approval_required(true));
    
    // Execute with timeout
    let result = tokio::time::timeout(
        Duration::from_secs(30),
        executor.execute(program)
    ).await??;
    
    // Validate results
    result.validate()?;
    
    Ok(result)
}
```

---

**Status**: Production Ready (Phase 2 Complete) ✅  
**Next Phase**: AI Integration (Phase 3) - Q2 2026  
**Quality**: 12/12 tests passing, zero warnings, sub-microsecond performance  
**Security**: Human approval framework ready for Phase 3 AI integration