use thiserror::Error;

#[derive(Error, Debug)]
pub enum CaptureError {
    #[error("Wayland connection failed: {0}")]
    WaylandConnection(String),
    
    #[error("Output not found: {0}")]
    OutputNotFound(String),
    
    #[error("Buffer allocation failed: {0}")]
    BufferAllocation(String),
    
    #[error("Capture timeout")]
    Timeout,
    
    #[error("Protocol error: {0}")]
    Protocol(String),
}

#[derive(Error, Debug)]
pub enum ConversionError {
    #[error("Unsupported format: {0}")]
    UnsupportedFormat(String),
    
    #[error("GPU import failed: {0}")]
    GpuImport(String),
    
    #[error("Memory mapping failed: {0}")]
    MemoryMapping(String),
    
    #[error("Texture creation failed: {0}")]
    TextureCreation(String),
}
