use core::marker::PhantomData;
use embassy_executor::raw;
use std::sync::{Condvar, Mutex};
use std::boxed::Box;


#[unsafe(export_name = "__pender")]
fn __pender(context: *mut ()) {
	let signaler: &'static Signaler = unsafe { std::mem::transmute(context) };
	signaler.signal()
}

pub struct Executor {
	inner: raw::Executor,
	not_send: PhantomData<*mut ()>,
	signaler: &'static Signaler,
}

impl Executor {
	pub fn new() -> Self {
		let signaler = Box::leak(Box::new(Signaler::new()));
		Self {
			inner: raw::Executor::new(signaler as *mut Signaler as *mut ()),
			not_send: PhantomData,
			signaler,
		}
	}
	
	pub fn run(&'static mut self, init: impl FnOnce(embassy_executor::Spawner)) -> ! {
		init(self.inner.spawner());

		loop {
			unsafe { self.inner.poll() };
			self.signaler.wait()
		}
	}
}

struct Signaler {
	mutex: Mutex<bool>,
	condvar: Condvar,
}

impl Signaler {
	fn new() -> Self {
		Self {
			mutex: Mutex::new(false),
			condvar: Condvar::new(),
		}
	}

	fn signal(&self) {
		let mut guard= self.mutex.lock();
		*guard = true;
		self.condvar.notify_one();
	}

	fn wait(&self) {
		let mut guard = self.mutex.lock();
		while !*guard {
			guard = self.condvar.wait(guard);
		}
		*guard = false;
	}
}

