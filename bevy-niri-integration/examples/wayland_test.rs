use std::env;
use bevy_niri_integration::wayland_client::NiriScreencopyClient;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Niri Wayland Screencopy Test ===");
    
    println!("XDG_RUNTIME_DIR: {:?}", env::var("XDG_RUNTIME_DIR"));
    println!("WAYLAND_DISPLAY: {:?}", env::var("WAYLAND_DISPLAY"));
    
    println!("\n--- Testing Wayland Connection ---");
    match NiriScreencopyClient::new() {
        Ok(mut client) => {
            println!("✅ Successfully connected to Wayland compositor");
            
            if let Err(e) = client.refresh_outputs() {
                println!("⚠️  Warning: Failed to refresh outputs: {:?}", e);
            }
            
            let outputs = client.get_outputs();
            println!("📺 Found {} output(s):", outputs.len());
            
            for output in &outputs {
                println!("  - {} ({}x{} @ {}Hz, scale: {})", 
                    output.name, output.width, output.height, 
                    output.refresh_rate, output.scale);
            }
            
            if let Some(first_output) = outputs.first() {
                println!("\n--- Testing Screen Capture ---");
                match client.capture_output(&first_output.name) {
                    Ok(buffer) => {
                        match buffer {
                            bevy_niri_integration::wayland_client::CaptureBuffer::Shm { 
                                width, height, stride, data, .. 
                            } => {
                                println!("✅ Successfully captured SHM buffer:");
                                println!("  Size: {}x{}, Stride: {}, Data: {} bytes", 
                                    width, height, stride, data.len());
                                
                                let non_zero_bytes = data.iter().filter(|&&b| b != 0).count();
                                println!("  Non-zero bytes: {} ({:.1}%)", 
                                    non_zero_bytes, 
                                    (non_zero_bytes as f32 / data.len() as f32) * 100.0);
                            }
                            bevy_niri_integration::wayland_client::CaptureBuffer::Dmabuf { 
                                width, height, .. 
                            } => {
                                println!("✅ Successfully captured DMA buffer: {}x{}", width, height);
                            }
                        }
                    }
                    Err(e) => {
                        println!("❌ Failed to capture screen: {:?}", e);
                    }
                }
                
                println!("\n--- Testing Event Polling ---");
                match client.poll_events() {
                    Ok(()) => println!("✅ Successfully polled Wayland events"),
                    Err(e) => println!("❌ Failed to poll events: {:?}", e),
                }
            } else {
                println!("⚠️  No outputs found to test capture");
            }
        }
        Err(e) => {
            println!("❌ Failed to connect to Wayland compositor: {:?}", e);
            return Err(Box::new(e));
        }
    }
    
    println!("\n=== Test Complete ===");
    Ok(())
}
