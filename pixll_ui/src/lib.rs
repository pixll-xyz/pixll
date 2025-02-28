use std::ffi::c_char;

// Declare external JavaScript functions that Rust will call
unsafe extern "C" {
    unsafe fn js_create_shader_module(device_handle: u32, shader_code_ptr: *const c_char) -> u32;
    unsafe fn js_create_render_pipeline(device_handle: u32, vs_handle: u32, fs_handle: u32) -> u32;
    unsafe fn js_render(device_handle: u32, pipeline_handle: u32);
}

// Vertex shader in WGSL: Positions a triangle based on vertex index
const VERTEX_SHADER: &[u8] = b"
@vertex
fn vs_main(@builtin(vertex_index) in_vertex_index: u32) -> @builtin(position) vec4<f32> {
    let x = f32(i32(in_vertex_index) - 1);
    let y = f32(i32(in_vertex_index & 1u) * 2 - 1);
    return vec4<f32>(x, y, 0.0, 1.0);
}
\0";

// Fragment shader in WGSL: Colors the triangle red
const FRAGMENT_SHADER: &[u8] = b"
@fragment
fn fs_main() -> @location(0) vec4<f32> {
    return vec4<f32>(1.0, 0.0, 0.0, 1.0);
}
\0";

// Callback function called by JavaScript once WebGPU is initialized
#[unsafe(no_mangle)]
pub extern "C" fn on_device_ready(device_handle: u32) {
    unsafe {
        // Create shader modules by passing shader code pointers to JavaScript
        let vs_handle = js_create_shader_module(device_handle, VERTEX_SHADER.as_ptr() as *const c_char);
        let fs_handle = js_create_shader_module(device_handle, FRAGMENT_SHADER.as_ptr() as *const c_char);

        // Create the render pipeline using the shader module handles
        let pipeline_handle = js_create_render_pipeline(device_handle, vs_handle, fs_handle);

        // Trigger the render pass
        js_render(device_handle, pipeline_handle);
    }
}