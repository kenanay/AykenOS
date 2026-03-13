//! Basic ABDF and BCIB usage examples

use abdf::segment::{MetaContainer, SegmentKind};
use abdf_builder::{decode_abdf, AbdfBuilder};
use bcib::{BcibHeader, BcibInstruction, BcibOpcode};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== ABDF Basic Usage ===");
    abdf_example()?;

    println!("\n=== BCIB Basic Usage ===");
    bcib_example()?;

    Ok(())
}

fn abdf_example() -> Result<(), Box<dyn std::error::Error>> {
    // Create a new ABDF builder
    let mut builder = AbdfBuilder::new();

    // Add some strings to the string pool
    let users_name = builder.intern_string("users");
    let table_type = builder.intern_string("table/generic");
    let schema_str = builder.intern_string("id:u64,name:string,email:string");

    // Create metadata for a user table
    let user_meta = MetaContainer {
        name_idx: users_name,
        type_idx: table_type,
        schema_idx: schema_str,
        permissions: 0,
        embedding_idx: 0,
    };

    // Sample user data (in a real scenario, this would be properly serialized)
    let user_data = b"1,John Doe,john@example.com\n2,Jane Smith,jane@example.com\n";

    // Add the user table segment
    let segment_idx = builder.add_segment(SegmentKind::Tabular(user_meta), user_data);
    println!("Added user table segment at index: {}", segment_idx);

    // Add a raw data segment
    let raw_data = &[0xDE, 0xAD, 0xBE, 0xEF, 0xCA, 0xFE, 0xBA, 0xBE];
    let raw_idx = builder.add_segment(SegmentKind::Raw, raw_data);
    println!("Added raw data segment at index: {}", raw_idx);

    // Build the ABDF buffer
    let buffer = builder.build();
    println!("Built ABDF buffer with {} bytes", buffer.len());

    // Decode the buffer
    let view = decode_abdf(&buffer)?;
    println!("Decoded ABDF buffer successfully");
    println!("Header version: {}", view.header.version);
    println!("Number of segments: {}", view.segments.len());

    // Access the user table
    if let Some(name) = view.segment_name(0) {
        println!("Segment 0 name: {}", name);
    }

    if let Some(data) = view.segment_data(0) {
        println!("Segment 0 data: {}", String::from_utf8_lossy(data));
    }

    // Access the raw data
    if let Some(data) = view.segment_data(1) {
        println!("Segment 1 data: {:02X?}", data);
    }

    Ok(())
}

fn bcib_example() -> Result<(), Box<dyn std::error::Error>> {
    // Create BCIB header
    let mut header = BcibHeader::new();
    header.instruction_count = 4;
    header.string_pool_offset = 16 + (4 * 8); // header + 4 instructions

    println!("BCIB Header:");
    println!(
        "  Magic: {:?}",
        std::str::from_utf8(&header.magic).unwrap_or("Invalid")
    );
    println!("  Version: {}", header.version);
    println!("  Instruction count: {}", header.instruction_count);

    // Create instructions
    let instructions = vec![
        // Select container 0 (users table)
        BcibInstruction::new(BcibOpcode::CtxSelect, 0, 0, 0, 0),
        // Query data using string at index 0
        BcibInstruction::new(BcibOpcode::DataQuery, 0, 0, 0, 0),
        // Render UI scene
        BcibInstruction::new(BcibOpcode::UiRender, 0, 1, 0, 0),
        // End execution
        BcibInstruction::new(BcibOpcode::End, 0, 0, 0, 0),
    ];

    println!("\nInstructions:");
    for (i, instr) in instructions.iter().enumerate() {
        println!(
            "  {}: {:?} (args: {}, {}, {})",
            i, instr.opcode, instr.arg0, instr.arg1, instr.arg2
        );
    }

    // String pool
    let string_pool = "SELECT * FROM users WHERE active = 1\0";
    println!("\nString pool: {:?}", string_pool);

    // In a real implementation, you would serialize this to a binary buffer
    println!(
        "\nTotal BCIB size would be: {} bytes",
        std::mem::size_of::<BcibHeader>()
            + instructions.len() * std::mem::size_of::<BcibInstruction>()
            + string_pool.len()
    );

    Ok(())
}
