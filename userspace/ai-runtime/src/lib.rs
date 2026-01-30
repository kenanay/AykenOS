pub mod ai_stub;
pub mod streaming;
pub mod abdf;
pub mod simd;
pub mod gguf;
pub mod security;
pub mod perf;
pub mod compatibility;

pub use ai_stub::{AiStub, AiError};
pub use streaming::{
    StreamingEngine, StreamingEngineImpl,
    StreamHandle, TokenStream, StreamProgress,
    CircularBuffer, BufferState
};
pub use abdf::{
    ABDFHeader, ABDFMetadata, ABDFModel, ABDFParser, ABDFParseError,
    TensorTable, TensorEntry, FormatDetector, Architecture, QuantizationInfo
};
pub use simd::{
    FusedKernel, FusedKernelDispatcher, KernelParams, KernelError,
    HardwareInfo, InstructionSet, NumericalAccuracyFramework,
    NumericalTolerance, PerformanceHint, SIMDIntegrationManager,
    ExecutionMetrics, IntegratedExecutionResult,
    IntegrationConfig, PerformanceTargets, FusedKernelBenchmark,
    BenchmarkConfig, BenchmarkReport, KernelBenchmarkResult,
    BaselineBenchmarkResult, BenchmarkTiming
};
pub use gguf::{
    QuantizationFormat, QuantizationError, QuantizationRegistry,
    get_quantization_registry, RecommendationEngine, RecommendationCriteria,
    QuantizationRecommendation, Int4Integration, Int4Compatibility
};
pub use security::{
    SecurityChain, SecurityChainImpl, IntegrityResult, SignatureResult,
    TrustDatabase, ModelTrustRecord, TrustLevel, VerificationStatus, ReputationScore,
    TrustDomain, IsolationLevel, IsolationManager, QuarantineReason,
    TrustEvent, TrustEventType, TrustImpact, EventContext
};
pub use perf::{
    PerformanceMonitor, SystemPerformanceMonitor, ComponentMetrics,
    SemanticProcessingMetrics, StreamingMetrics, ABDFMetrics,
    SecurityMetrics, PerformanceDegradationDetector, DiagnosticInfo,
    PerformanceHistory, TrendAnalysis, MetricType, MetricValue,
    PerformanceAlert, AlertSeverity, PerformanceThresholds,
    TelemetryCollector, TelemetryInterface, TelemetryExporter,
    MetricExport, TelemetryConfig, ExportFormat, ExportDestination,
    UnifiedTelemetrySystem, TelemetryError
};
pub use compatibility::{
    CompatibilityManager, Feature, FeatureState, DegradationPolicy,
    FallbackStrategy, ResourcePriorities, DegradationEvent,
    CompatibilityResult, TraditionalCLI, BasicModelLoader,
    StandardSIMD, BasicSecurity, ModelInfo
};