//! Provide basic [capability-based security].
//!
//! The wrapper type [`WithCap`] associates a **capability** to an object, that
//! is a set of access rights. When accessing the object, we must explicitly
//! specify the access capability, and it must not violate the capability
//! associated with the object at initialization.
//!
//! # Examples
//!
//! ```
//! use capability::{Cap, WithCap};
//!
//! let data = WithCap::new(42, Cap::READ | Cap::WRITE);
//!
//! // Access with the correct capability.
//! assert_eq!(data.access(Cap::READ).unwrap(), &42);
//! assert_eq!(data.access(Cap::WRITE).unwrap(), &42);
//! assert_eq!(data.access(Cap::READ | Cap::WRITE).unwrap(), &42);
//!
//! // Access with the incorrect capability.
//! assert!(data.access(Cap::EXECUTE).is_err());
//! assert!(data.access(Cap::READ | Cap::EXECUTE).is_err());
//! ```
//!
//! [capability-based security]:
//!     https://en.wikipedia.org/wiki/Capability-based_security
//!

#![no_std]

bitflags::bitflags! {
    /// Capabilities (access rights).
    #[derive(Default, Debug, Clone, Copy)]
    pub struct Cap: u32 {
        /// Readable access.
        const READ = 1 << 0;
        /// Writable access.
        const WRITE = 1 << 1;
        /// Executable access.
        const EXECUTE = 1 << 2;
    }
}

/// Error type for capability violation.
#[derive(Debug, Default, Eq, PartialEq)]
#[non_exhaustive]
pub struct CapError;

/// A wrapper that holds a type with a capability.
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
    ///
    /// # Examples
    ///
    /// ```
    /// use capability::{Cap, WithCap};
    ///
    /// let data = WithCap::new(42, Cap::READ);
    ///
    /// assert!(data.can_access(Cap::READ));
    /// assert!(!data.can_access(Cap::WRITE));
    /// ```
    pub const fn can_access(&self, cap: Cap) -> bool {
        self.cap.contains(cap)
    }

    /// Access the inner value without capability check.
    ///
    /// # Safety
    ///
    /// Caller must ensure not to violate the capability.
    pub unsafe fn access_unchecked(&self) -> &T {
        &self.inner
    }

    /// Access the inner value with the given capability, or return `CapError`
    /// if cannot access.
    ///
    /// # Examples
    ///
    /// ```
    /// use capability::{Cap, CapError, WithCap};
    ///
    /// let data = WithCap::new(42, Cap::READ);
    ///
    /// assert_eq!(data.access(Cap::READ).unwrap(), &42);
    /// assert_eq!(data.access(Cap::WRITE).err(), Some(CapError::default()));
    /// ```
    pub const fn access(&self, cap: Cap) -> Result<&T, CapError> {
        if self.can_access(cap) {
            Ok(&self.inner)
        } else {
            Err(CapError)
        }
    }

    /// Access the inner value with the given capability, or return the given
    /// `err` if cannot access.
    ///
    /// # Examples
    ///
    /// ```
    /// use capability::{Cap, WithCap};
    ///
    /// let data = WithCap::new(42, Cap::READ);
    ///
    /// assert_eq!(data.access_or_err(Cap::READ, "cannot read").unwrap(), &42);
    /// assert_eq!(data.access_or_err(Cap::WRITE, "cannot write").err(), Some("cannot write"));
    /// ```
    pub fn access_or_err<E>(&self, cap: Cap, err: E) -> Result<&T, E> {
        if self.can_access(cap) {
            Ok(&self.inner)
        } else {
            Err(err)
        }
    }
}

impl From<CapError> for axerrno::AxError {
    #[inline]
    fn from(_: CapError) -> Self {
        Self::PermissionDenied
    }
}
