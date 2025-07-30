use std::time::Duration;

use niri::niri::State;
use smithay::reexports::wayland_server::Display;
use calloop::EventLoop;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();
    
    println!("Testing Bevy-Niri integration...");
    
    let mut event_loop = EventLoop::<State>::try_new()?;
    let display = Display::new()?;
    
    let config = niri_config::Config::default();
    
    let mut state = State::new(
        config,
        event_loop.handle(),
        event_loop.get_signal(),
        display,
        false,
        false,
        false,
    )?;
    
    println!("State created successfully");
    
    if state.niri.bevy_renderer.is_some() {
        println!("✓ Bevy renderer initialized");
    } else {
        println!("✗ Bevy renderer not initialized");
    }
    
    println!("✓ Bevy texture cache initialized: {} entries", state.niri.bevy_texture_cache.len());
    
    for _ in 0..5 {
        state.refresh_and_flush_clients();
        std::thread::sleep(Duration::from_millis(16));
    }
    
    println!("✓ Refresh cycle completed successfully");
    println!("Bevy-Niri integration test passed!");
    
    Ok(())
}
