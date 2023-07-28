# buddy_system_allocator

[![Crates.io version][crate-img]][crate]
[![docs.rs][docs-img]][docs]

An (almost) drop-in replacement for [phil-opp/linked-list-allocator](https://github.com/phil-opp/linked-list-allocator). But it uses buddy system instead.

## Usage

To use buddy_system_allocator for global allocator:

```rust
use buddy_system_allocator::LockedHeap;

#[global_allocator]
static HEAP_ALLOCATOR: LockedHeap = LockedHeap::<32>::empty();
```

To init the allocator:

```rust
unsafe {
    HEAP_ALLOCATOR.lock().init(heap_start, heap_size);
    // or
    HEAP_ALLOCATOR.lock().add_to_heap(heap_start, heap_end);
}
```

You can also use `FrameAllocator` and `LockedHeapWithRescue`, see their documentation for usage.

## Features

- **`use_spin`** (default): Provide a `LockedHeap` type that implements the [`GlobalAlloc`] trait by using a spinlock.
- **`const_fn`** (nightly only): Provide const fn version of `LockedHeapWithRescue::new`.

[`GlobalAlloc`]: https://doc.rust-lang.org/nightly/core/alloc/trait.GlobalAlloc.html

## License

Some code comes from phil-opp's linked-list-allocator.

Licensed under MIT License. Thanks phill-opp's linked-list-allocator for inspirations and interface.

[crate-img]:     https://img.shields.io/crates/v/buddy_system_allocator.svg
[crate]:         https://crates.io/crates/buddy_system_allocator
[docs-img]:      https://docs.rs/buddy_system_allocator/badge.svg
[docs]:          https://docs.rs/buddy_system_allocator
