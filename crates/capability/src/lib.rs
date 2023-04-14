#![no_std]
#![feature(const_trait_impl)]

bitflags::bitflags! {
    /// Capabilities.
    #[derive(Default, Debug, Clone, Copy)]
    pub struct Cap: u32 {
        const READ = 1 << 0;
        const WRITE = 1 << 1;
        const EXECUTE = 1 << 2;
    }
}

#[derive(Debug)]
pub struct CapError;

/// A wrapper that holds a value with a capability.
pub struct WithCap<T> {
    inner: T,
    cap: Cap,
}

impl<T> WithCap<T> {
    /// Create a new instance with the given capability.
    pub fn new(inner: T, cap: Cap) -> Self {
        Self { inner, cap }
    }

    /// Get the capability.
    pub const fn cap(&self) -> Cap {
        self.cap
    }

    /// Check if the inner data can be accessed with the given capability.
    pub const fn can_access(&self, cap: Cap) -> bool {
        self.cap.contains(cap)
    }

    /// # Safety
    ///
    /// Caller must ensure not to violate the capability.
    pub unsafe fn access_unchecked(&self) -> &T {
        &self.inner
    }

    /// Access the inner value with the given capability, or return `CapError`
    /// if cannot access.
    pub const fn access(&self, cap: Cap) -> Result<&T, CapError> {
        if self.can_access(cap) {
            Ok(&self.inner)
        } else {
            Err(CapError)
        }
    }

    /// Access the inner value with the given capability, or return the given
    /// `err` if cannot access.
    pub fn access_or_err<E>(&self, cap: Cap, err: E) -> Result<&T, E> {
        if self.can_access(cap) {
            Ok(&self.inner)
        } else {
            Err(err)
        }
    }
}

impl const From<CapError> for axerrno::AxError {
    fn from(_: CapError) -> Self {
        Self::PermissionDenied
    }
}
