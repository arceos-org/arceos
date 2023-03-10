pub mod display;
pub use axgpu;

use driver_gpu::GpuDriverOps;

pub fn framebuffer() -> isize {
	let mut device = axgpu::gpu_devices().inner.lock();
	let fb = device.0.get_framebuffer();
	let len = fb.len();
	debug!("[kernel] FrameBuffer: addr 0x{:X}, len {}", fb.as_ptr() as usize , len);
	fb.as_ptr() as isize
}

pub fn framebuffer_flush() -> isize {
	let mut device = axgpu::gpu_devices().inner.lock();
    device.0.flush().unwrap();
    0
}