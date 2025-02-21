pub mod device;
pub mod buffer;
pub mod pipeline;

pub trait GpuBackend {
    fn create_device() -> Self;
    fn create_buffer(&self, size: usize) -> pixll::gpu::buffer::GpuBuffer;
    fn render_frame(&self);
}
