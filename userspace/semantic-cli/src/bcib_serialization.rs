// bcib_serialization.rs
// BCIB serialization for kernel submission
// Converts semantic-cli BCIB to byte format for BcibGraph

use crate::bcib_simple::{BCIB, BCIBInstruction, BCIBOperand};
use crate::error::{ErrorCode, SemanticCLIError};

/// Serialize BCIB to byte format for kernel submission
/// 
/// This is a simplified serialization for the minimal BCIB subset.
/// Production implementation would use proper binary format.
pub fn serialize_bcib(bcib: &BCIB) -> Result<Vec<u8>, SemanticCLIError> {
    let mut bytes = Vec::new();
    
    // Magic header (placeholder)
    bytes.extend_from_slice(b"BCIB");
    
    // Version
    bytes.push(1);
    
    // Instruction count
    bytes.extend_from_slice(&(bcib.instructions.len() as u32).to_le_bytes());
    
    // Serialize each instruction
    for instr in &bcib.instructions {
        serialize_instruction(instr, &mut bytes)?;
    }
    
    Ok(bytes)
}

fn serialize_instruction(instr: &BCIBInstruction, bytes: &mut Vec<u8>) -> Result<(), SemanticCLIError> {
    match instr {
        BCIBInstruction::DataQuery { target, context, filter } => {
            // Opcode: 0x01
            bytes.push(0x01);
            
            // Target operand
            serialize_operand(target, bytes)?;
            
            // Context (length-prefixed string)
            let context_bytes = context.as_bytes();
            bytes.extend_from_slice(&(context_bytes.len() as u32).to_le_bytes());
            bytes.extend_from_slice(context_bytes);
            
            // Filter (optional, length-prefixed)
            if let Some(f) = filter {
                bytes.push(1); // has filter
                let filter_bytes = f.as_bytes();
                bytes.extend_from_slice(&(filter_bytes.len() as u32).to_le_bytes());
                bytes.extend_from_slice(filter_bytes);
            } else {
                bytes.push(0); // no filter
            }
        }
        
        BCIBInstruction::DataCreate { target, context, data } => {
            // Opcode: 0x02
            bytes.push(0x02);
            
            serialize_operand(target, bytes)?;
            
            let context_bytes = context.as_bytes();
            bytes.extend_from_slice(&(context_bytes.len() as u32).to_le_bytes());
            bytes.extend_from_slice(context_bytes);
            
            let data_bytes = data.as_bytes();
            bytes.extend_from_slice(&(data_bytes.len() as u32).to_le_bytes());
            bytes.extend_from_slice(data_bytes);
        }
        
        BCIBInstruction::End { result } => {
            // Opcode: 0xFF
            bytes.push(0xFF);
            
            serialize_operand(result, bytes)?;
        }
        
        BCIBInstruction::TraceEmit { message } => {
            // Opcode: 0xFE
            bytes.push(0xFE);
            
            let msg_bytes = message.as_bytes();
            bytes.extend_from_slice(&(msg_bytes.len() as u32).to_le_bytes());
            bytes.extend_from_slice(msg_bytes);
        }
        
        BCIBInstruction::Nop => {
            return Err(SemanticCLIError::execution_error(
                "Cannot serialize Nop instruction",
                ErrorCode::E760,
            ));
        }
    }
    
    Ok(())
}

fn serialize_operand(operand: &BCIBOperand, bytes: &mut Vec<u8>) -> Result<(), SemanticCLIError> {
    match operand {
        BCIBOperand::Register(reg) => {
            bytes.push(0x00); // Register tag
            bytes.extend_from_slice(&(*reg as u32).to_le_bytes());
        }
        BCIBOperand::Literal(lit) => {
            bytes.push(0x01); // Literal tag
            let lit_bytes = lit.as_bytes();
            bytes.extend_from_slice(&(lit_bytes.len() as u32).to_le_bytes());
            bytes.extend_from_slice(lit_bytes);
        }
    }
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_serialize_simple_bcib() {
        let bcib = BCIB {
            instructions: vec![
                BCIBInstruction::DataQuery {
                    target: BCIBOperand::Register(0),
                    context: "users".to_string(),
                    filter: None,
                },
                BCIBInstruction::End {
                    result: BCIBOperand::Register(0),
                },
            ],
        };

        let bytes = serialize_bcib(&bcib).expect("serialization failed");
        
        // Verify magic header
        assert_eq!(&bytes[0..4], b"BCIB");
        
        // Verify version
        assert_eq!(bytes[4], 1);
        
        // Verify instruction count
        let instr_count = u32::from_le_bytes([bytes[5], bytes[6], bytes[7], bytes[8]]);
        assert_eq!(instr_count, 2);
    }

    #[test]
    fn test_serialize_rejects_nop() {
        let bcib = BCIB {
            instructions: vec![BCIBInstruction::Nop],
        };

        let result = serialize_bcib(&bcib);
        assert!(result.is_err());
    }
}
