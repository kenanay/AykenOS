pub mod ai_stub;
pub mod capability_gate;
pub mod suggestion;

pub use ai_stub::{AiStub, AiError};
pub use capability_gate::{AiCapability, AiCapabilitySet};
pub use suggestion::{suggest, validate_boundary, AiSuggestion, SuggestionEngine};
