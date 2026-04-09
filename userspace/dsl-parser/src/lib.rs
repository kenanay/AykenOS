pub mod bcib_ir;
pub mod parser;

#[cfg(test)]
mod test_parser;

#[cfg(test)]
mod integration_test;

pub use bcib_ir::command_to_bcib_ir;
pub use parser::{Command, DispatchRequest, DslParser, ExecutionContext, ParseError};
