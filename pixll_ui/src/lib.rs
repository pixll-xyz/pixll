use std::ffi::c_char;

// External JavaScript functions that Rust will call
unsafe extern "C" {
    unsafe fn js_create_shader_module(device_handle: u32, shader_code_ptr: *const c_char) -> u32;
    unsafe fn js_create_buffer(device_handle: u32, data_ptr: *const u8, size: u32) -> u32;
    unsafe fn js_create_render_pipeline(device_handle: u32, vs_handle: u32, fs_handle: u32) -> u32;
    unsafe fn js_render(device_handle: u32, pipeline_handle: u32, vertex_buffer_handle: u32);
}

// Vertex shader in WGSL: Uses vertex buffer for positions
const VERTEX_SHADER: &[u8] = b"
@vertex
fn vs_main(@location(0) pos: vec2<f32>) -> @builtin(position) vec4<f32> {
    return vec4<f32>(pos, 0.0, 1.0);
}
\0";

// Fragment shader in WGSL: Colors the rectangle gray
const FRAGMENT_SHADER: &[u8] = b"
@fragment
fn fs_main() -> @location(0) vec4<f32> {
    return vec4<f32>(0.5, 0.5, 0.5, 1.0); // Gray color
}
\0";

// Global handles for the pipeline and vertex buffer
static mut PIPELINE_HANDLE: u32 = 0;
static mut VERTEX_BUFFER_HANDLE: u32 = 0;

// Canvas dimensions (assumed fixed for simplicity)
const CANVAS_WIDTH: f32 = 800.0;
const CANVAS_HEIGHT: f32 = 600.0;

// Convert pixel coordinates to clip space (-1 to 1)
fn pixel_to_clip(x: f32, y: f32) -> (f32, f32) {
    let x_clip = (x / CANVAS_WIDTH) * 2.0 - 1.0;
    let y_clip = 1.0 - (y / CANVAS_HEIGHT) * 2.0;
    (x_clip, y_clip)
}

// Button definition
struct Button {
    x: f32,
    y: f32,
    width: f32,
    height: f32,
}

static BUTTON: Button = Button {
    x: 100.0,    // Top-left x in pixels
    y: 100.0,    // Top-left y in pixels
    width: 200.0,
    height: 50.0,
};

// Generate vertex data for the button (triangle strip: bl, br, tl, tr)
fn get_button_vertices() -> [f32; 8] {
    let (left, _) = pixel_to_clip(BUTTON.x, 0.0);
    let (right, _) = pixel_to_clip(BUTTON.x + BUTTON.width, 0.0);
    let (_, top) = pixel_to_clip(0.0, BUTTON.y);
    let (_, bottom) = pixel_to_clip(0.0, BUTTON.y + BUTTON.height);

    [
        left, bottom,  // bottom-left
        right, bottom, // bottom-right
        left, top,     // top-left
        right, top,    // top-right
    ]
}

// Called by JavaScript when the WebGPU device is ready
#[unsafe(no_mangle)]
pub extern "C" fn on_device_ready(device_handle: u32) {
    unsafe {
        // Create shader modules
        let vs_handle = js_create_shader_module(device_handle, VERTEX_SHADER.as_ptr() as *const c_char);
        let fs_handle = js_create_shader_module(device_handle, FRAGMENT_SHADER.as_ptr() as *const c_char);
        
        // Create render pipeline
        PIPELINE_HANDLE = js_create_render_pipeline(device_handle, vs_handle, fs_handle);
        
        // Create vertex buffer for the button
        let vertices = get_button_vertices();
        VERTEX_BUFFER_HANDLE = js_create_buffer(
            device_handle,
            vertices.as_ptr() as *const u8,
            (vertices.len() * core::mem::size_of::<f32>()) as u32
        );
    }
}

// Called by JavaScript each frame
#[unsafe(no_mangle)]
pub extern "C" fn render_frame(device_handle: u32) {
    unsafe {
        js_render(device_handle, PIPELINE_HANDLE, VERTEX_BUFFER_HANDLE);
    }
}

// Called by JavaScript when a click occurs
#[unsafe(no_mangle)]
pub extern "C" fn handle_click(x: f32, y: f32) {
    // Check if the click is within the button's bounds (in pixel space)
    if x >= BUTTON.x && x <= BUTTON.x + BUTTON.width && 
       y >= BUTTON.y && y <= BUTTON.y + BUTTON.height {
        // Note: In a real application, you'd want to trigger some action here.
        // For this example, we just print to the console (visible in browser dev tools).
        println!("Button clicked at ({}, {})", x, y);
    }
}

// Required for Rust to allocate memory correctly in WebAssembly
#[unsafe(no_mangle)]
pub extern "C" fn __wasm_call_ctors() {}