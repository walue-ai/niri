use std::collections::HashMap;
use wayland_client::{Connection, Dispatch, QueueHandle, EventQueue, globals::{GlobalList, GlobalListContents}};
use wayland_protocols_wlr::screencopy::v1::client::{
    zwlr_screencopy_frame_v1, zwlr_screencopy_manager_v1,
};
use wayland_client::protocol::{wl_output, wl_registry, wl_shm, wl_buffer};
use crate::error::CaptureError;

#[derive(Debug, Clone)]
pub struct OutputInfo {
    pub name: String,
    pub width: u32,
    pub height: u32,
    pub refresh_rate: u32,
    pub scale: f64,
}

#[derive(Debug)]
pub enum CaptureBuffer {
    Shm { 
        data: Vec<u8>, 
        stride: u32,
        width: u32,
        height: u32,
        format: u32,
    },
    Dmabuf { 
        width: u32,
        height: u32,
        format: u32,
        modifier: u64,
        num_planes: u32,
        fds: Vec<i32>,
        offsets: Vec<u32>,
        strides: Vec<u32>,
    },
}

#[derive(Debug)]
struct CaptureState {
    frame: Option<zwlr_screencopy_frame_v1::ZwlrScreencopyFrameV1>,
    buffer: Option<CaptureBuffer>,
    ready: bool,
    failed: bool,
}

pub struct NiriScreencopyClient {
    connection: Connection,
    event_queue: EventQueue<AppData>,
    screencopy_manager: Option<zwlr_screencopy_manager_v1::ZwlrScreencopyManagerV1>,
    shm: Option<wl_shm::WlShm>,
    outputs: HashMap<String, (wl_output::WlOutput, OutputInfo)>,
    active_captures: HashMap<String, CaptureState>,
}

struct AppData {
    outputs: HashMap<String, (wl_output::WlOutput, OutputInfo)>,
    screencopy_manager: Option<zwlr_screencopy_manager_v1::ZwlrScreencopyManagerV1>,
    shm: Option<wl_shm::WlShm>,
}

impl NiriScreencopyClient {
    pub fn new() -> Result<Self, CaptureError> {
        let connection = Connection::connect_to_env()
            .map_err(|e| CaptureError::WaylandConnection(e.to_string()))?;
        
        let (globals, event_queue) = wayland_client::globals::registry_queue_init::<AppData>(&connection)
            .map_err(|e| CaptureError::WaylandConnection(e.to_string()))?;
        
        let mut app_data = AppData {
            outputs: HashMap::new(),
            screencopy_manager: None,
            shm: None,
        };
        
        app_data.screencopy_manager = Some(globals.bind(&event_queue.handle(), 1..=3, ())
            .map_err(|e| CaptureError::Protocol(e.to_string()))?);
        
        app_data.shm = Some(globals.bind(&event_queue.handle(), 1..=1, ())
            .map_err(|e| CaptureError::Protocol(e.to_string()))?);
        
        let output_globals = globals.contents().with_list(|list| {
            list.iter()
                .filter(|global| global.interface == "wl_output")
                .map(|global| global.name)
                .collect::<Vec<_>>()
        });
        
        let mut outputs = Vec::new();
        for name in output_globals {
            let output: wl_output::WlOutput = globals.registry().bind::<wl_output::WlOutput, (), AppData>(name, 1, &event_queue.handle(), ());
            outputs.push(output);
        }
        
        for (i, output) in outputs.into_iter().enumerate() {
            let output_info = OutputInfo {
                name: format!("output-{}", i),
                width: 1920,
                height: 1080,
                refresh_rate: 60,
                scale: 1.0,
            };
            app_data.outputs.insert(output_info.name.clone(), (output, output_info));
        }
        
        Ok(Self {
            connection,
            event_queue,
            screencopy_manager: app_data.screencopy_manager.clone(),
            shm: app_data.shm.clone(),
            outputs: app_data.outputs.clone(),
            active_captures: HashMap::new(),
        })
    }
    
    pub fn get_outputs(&self) -> Vec<OutputInfo> {
        self.outputs.values().map(|(_, info)| info.clone()).collect()
    }
    
    pub fn capture_output(&mut self, output_name: &str) -> Result<CaptureBuffer, CaptureError> {
        let (output, output_info) = self.outputs.get(output_name)
            .ok_or_else(|| CaptureError::OutputNotFound(output_name.to_string()))?;
        
        let screencopy_manager = self.screencopy_manager.as_ref()
            .ok_or_else(|| CaptureError::Protocol("No screencopy manager".to_string()))?;
        
        let frame = screencopy_manager.capture_output(0, output, &self.event_queue.handle(), ());
        
        let capture_state = CaptureState {
            frame: Some(frame),
            buffer: None,
            ready: false,
            failed: false,
        };
        
        self.active_captures.insert(output_name.to_string(), capture_state);
        
        self.create_shm_buffer(output_info)
    }
    
    fn create_shm_buffer(&self, output_info: &OutputInfo) -> Result<CaptureBuffer, CaptureError> {
        let stride = output_info.width * 4;
        let size = stride * output_info.height;
        let data = vec![0u8; size as usize];
        
        Ok(CaptureBuffer::Shm {
            data,
            stride,
            width: output_info.width,
            height: output_info.height,
            format: 0x34325258, // DRM_FORMAT_XRGB8888
        })
    }
    
    pub fn poll_events(&mut self) -> Result<(), CaptureError> {
        self.event_queue.blocking_dispatch(&mut AppData {
            outputs: self.outputs.clone(),
            screencopy_manager: self.screencopy_manager.clone(),
            shm: self.shm.clone(),
        }).map_err(|e| CaptureError::Protocol(e.to_string()))?;
        
        Ok(())
    }
}

impl Dispatch<wl_registry::WlRegistry, GlobalListContents> for AppData {
    fn event(
        _state: &mut Self,
        _proxy: &wl_registry::WlRegistry,
        _event: wl_registry::Event,
        _data: &GlobalListContents,
        _conn: &Connection,
        _qhandle: &QueueHandle<AppData>,
    ) {
    }
}

impl Dispatch<wl_output::WlOutput, ()> for AppData {
    fn event(
        _state: &mut Self,
        _proxy: &wl_output::WlOutput,
        _event: wl_output::Event,
        _data: &(),
        _conn: &Connection,
        _qhandle: &QueueHandle<AppData>,
    ) {
    }
}

impl Dispatch<zwlr_screencopy_manager_v1::ZwlrScreencopyManagerV1, ()> for AppData {
    fn event(
        _state: &mut Self,
        _proxy: &zwlr_screencopy_manager_v1::ZwlrScreencopyManagerV1,
        _event: zwlr_screencopy_manager_v1::Event,
        _data: &(),
        _conn: &Connection,
        _qhandle: &QueueHandle<AppData>,
    ) {
    }
}

impl Dispatch<zwlr_screencopy_frame_v1::ZwlrScreencopyFrameV1, ()> for AppData {
    fn event(
        _state: &mut Self,
        _proxy: &zwlr_screencopy_frame_v1::ZwlrScreencopyFrameV1,
        _event: zwlr_screencopy_frame_v1::Event,
        _data: &(),
        _conn: &Connection,
        _qhandle: &QueueHandle<AppData>,
    ) {
    }
}

impl Dispatch<wl_shm::WlShm, ()> for AppData {
    fn event(
        _state: &mut Self,
        _proxy: &wl_shm::WlShm,
        _event: wl_shm::Event,
        _data: &(),
        _conn: &Connection,
        _qhandle: &QueueHandle<AppData>,
    ) {
    }
}

impl Dispatch<wl_buffer::WlBuffer, ()> for AppData {
    fn event(
        _state: &mut Self,
        _proxy: &wl_buffer::WlBuffer,
        _event: wl_buffer::Event,
        _data: &(),
        _conn: &Connection,
        _qhandle: &QueueHandle<AppData>,
    ) {
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_capture_buffer_creation() {
        let output_info = OutputInfo {
            name: "test-output".to_string(),
            width: 1920,
            height: 1080,
            refresh_rate: 60,
            scale: 1.0,
        };
        
        let client = NiriScreencopyClient::new();
        if let Ok(client) = client {
            let buffer = client.create_shm_buffer(&output_info);
            assert!(buffer.is_ok());
            
            if let Ok(CaptureBuffer::Shm { width, height, stride, .. }) = buffer {
                assert_eq!(width, 1920);
                assert_eq!(height, 1080);
                assert_eq!(stride, 1920 * 4);
            }
        }
    }
}
