//! Aether-TUI Build Script
//! Compiles the Protobuf definitions into Rust client code.
//!
//! Created by Joaquín Escalante (https://github.com/joaquinescalante23)

fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_build::configure()
        .build_server(false)
        .build_client(true)
        .compile(
            &["../proto/chronos.proto"],
            &["../proto"],
        )?;
    Ok(())
}

// Created by Joaquín Escalante (https://github.com/joaquinescalante23)
