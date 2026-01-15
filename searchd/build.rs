fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Compile protobuf files
    // Note: This requires protoc to be installed and available in PATH
    // Download from: https://github.com/protocolbuffers/protobuf/releases
    prost_build::compile_protos(&["../proto/search.proto"], &["../proto/"])?;
    Ok(())
}
