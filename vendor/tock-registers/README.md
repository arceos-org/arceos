# Tock Register Interface

This crate provides an interface and types for defining and
manipulating registers and bitfields.

## Defining registers

The crate provides three types for working with memory mapped registers:
`ReadWrite`, `ReadOnly`, and `WriteOnly`, providing read-write, read-only, and
write-only functionality, respectively. These types implement the `Readable`,
`Writeable` and `ReadWriteable` traits.

Defining the registers is done with the `register_structs` macro, which expects
for each register an offset, a field name, and a type. Registers must be
declared in increasing order of offsets and contiguously. Gaps when defining the
registers must be explicitly annotated with an offset and gap identifier (by
convention using a field named `_reservedN`), but without a type. The macro will
then automatically take care of calculating the gap size and inserting a
suitable filler struct. The end of the struct is marked with its size and the
`@END` keyword, effectively pointing to the offset immediately past the list of
registers.

```rust
use tock_registers::registers::{ReadOnly, ReadWrite, WriteOnly};

register_structs! {
    Registers {
        // Control register: read-write
        // The 'Control' parameter constrains this register to only use fields from
        // a certain group (defined below in the bitfields section).
        (0x000 => cr: ReadWrite<u8, Control::Register>),

        // Status register: read-only
        (0x001 => s: ReadOnly<u8, Status::Register>),

        // Registers can be bytes, halfwords, or words:
        // Note that the second type parameter can be omitted, meaning that there
        // are no bitfields defined for these registers.
        (0x002 => byte0: ReadWrite<u8>),
        (0x003 => byte1: ReadWrite<u8>),
        (0x004 => short: ReadWrite<u16>),

        // Empty space between registers must be marked with a padding field,
        // declared as follows. The length of this padding is automatically
        // computed by the macro.
        (0x006 => _reserved),
        (0x008 => word: ReadWrite<u32>),

        // The type for a register can be anything. Conveniently, you can use an
        // array when there are a bunch of similar registers.
        (0x00C => array: [ReadWrite<u32>; 4]),
        (0x01C => ... ),

        // Etc.

        // The end of the struct is marked as follows.
        (0x100 => @END),
    }
}
```

This generates a C-style struct of the following form.

```rust
#[repr(C)]
struct Registers {
    // Control register: read-write
    // The 'Control' parameter constrains this register to only use fields from
    // a certain group (defined below in the bitfields section).
    cr: ReadWrite<u8, Control::Register>,

    // Status register: read-only
    s: ReadOnly<u8, Status::Register>

    // Registers can be bytes, halfwords, or words:
    // Note that the second type parameter can be omitted, meaning that there
    // are no bitfields defined for these registers.
    byte0: ReadWrite<u8>,
    byte1: ReadWrite<u8>,
    short: ReadWrite<u16>,

    // The padding length was automatically computed as 0x008 - 0x006.
    _reserved: [u8; 2],
    word: ReadWrite<u32>,

    // Arrays are expanded as-is, like any other type.
    array: [ReadWrite<u32>; 4],

    // Etc.
}
```

This crate will generate additional, compile time (`const`) assertions
to validate various invariants of the register structs, such as

- proper start offset of padding fields,
- proper start and end offsets of actual fields,
- invalid alignment of field types,
- the `@END` marker matching the size of the struct.

For more information on the generated assertions, check out the [`test_fields!`
macro documentation](https://docs.tockos.org/tock_registers/macro.test_fields.html).

By default, the visibility of the generated structs and fields is private. You
can make them public using the `pub` keyword, just before the struct name or the
field identifier.

For example, the following call to the macro:

```rust
register_structs! {
    pub Registers {
        (0x000 => foo: ReadOnly<u32>),
        (0x004 => pub bar: ReadOnly<u32>),
        (0x008 => @END),
    }
}
```

will generate the following struct.

```rust
#[repr(C)]
pub struct Registers {
    foo: ReadOnly<u32>,
    pub bar: ReadOnly<u32>,
}
```

## Defining bitfields

Bitfields are defined through the `register_bitfields!` macro:

```rust
register_bitfields! [
    // First parameter is the register width. Can be u8, u16, u32, or u64.
    u32,

    // Each subsequent parameter is a register abbreviation, its descriptive
    // name, and its associated bitfields.
    // The descriptive name defines this 'group' of bitfields. Only registers
    // defined as ReadWrite<_, Control::Register> can use these bitfields.
    Control [
        // Bitfields are defined as:
        // name OFFSET(shift) NUMBITS(num) [ /* optional values */ ]

        // This is a two-bit field which includes bits 4 and 5
        RANGE OFFSET(4) NUMBITS(2) [
            // Each of these defines a name for a value that the bitfield can be
            // written with or matched against. Note that this set is not exclusive--
            // the field can still be written with arbitrary constants.
            VeryHigh = 0,
            High = 1,
            Low = 2
        ],

        // A common case is single-bit bitfields, which usually just mean
        // 'enable' or 'disable' something.
        EN  OFFSET(3) NUMBITS(1) [],
        INT OFFSET(2) NUMBITS(1) []
    ],

    // Another example:
    // Status register
    Status [
        TXCOMPLETE  OFFSET(0) NUMBITS(1) [],
        TXINTERRUPT OFFSET(1) NUMBITS(1) [],
        RXCOMPLETE  OFFSET(2) NUMBITS(1) [],
        RXINTERRUPT OFFSET(3) NUMBITS(1) [],
        MODE        OFFSET(4) NUMBITS(3) [
            FullDuplex = 0,
            HalfDuplex = 1,
            Loopback = 2,
            Disabled = 3
        ],
        ERRORCOUNT OFFSET(6) NUMBITS(3) []
    ],

    // In a simple case, offset can just be a number, and the number of bits
    // is set to 1:
    InterruptFlags [
        UNDES   10,
        TXEMPTY  9,
        NSSR     8,
        OVRES    3,
        MODF     2,
        TDRE     1,
        RDRF     0
    ]
]
```

## Register Interface Summary

There are four types provided by the register interface: `ReadOnly`,
`WriteOnly`, `ReadWrite`, and `Aliased`. They expose the following
methods, through the implementations of the `Readable`, `Writeable`
and `ReadWriteable` traits respectively:

```rust
ReadOnly<T: UIntLike, R: RegisterLongName = ()>: Readable
.get() -> T                                    // Get the raw register value
.read(field: Field<T, R>) -> T                 // Read the value of the given field
.read_as_enum<E>(field: Field<T, R>) -> Option<E> // Read value of the given field as a enum member
.is_set(field: Field<T, R>) -> bool            // Check if one or more bits in a field are set
.matches_any(value: FieldValue<T, R>) -> bool  // Check if any specified parts of a field match
.matches_all(value: FieldValue<T, R>) -> bool  // Check if all specified parts of a field match
.extract() -> LocalRegisterCopy<T, R>          // Make local copy of register

WriteOnly<T: UIntLike, R: RegisterLongName = ()>: Writeable
.set(value: T)                                 // Set the raw register value
.write(value: FieldValue<T, R>)                // Write the value of one or more fields,
                                               //  overwriting other fields to zero
ReadWrite<T: UIntLike, R: RegisterLongName = ()>: Readable + Writeable + ReadWriteable
.get() -> T                                    // Get the raw register value
.set(value: T)                                 // Set the raw register value
.read(field: Field<T, R>) -> T                 // Read the value of the given field
.read_as_enum<E>(field: Field<T, R>) -> Option<E> // Read value of the given field as a enum member
.write(value: FieldValue<T, R>)                // Write the value of one or more fields,
                                               //  overwriting other fields to zero
.modify(value: FieldValue<T, R>)               // Write the value of one or more fields,
                                               //  leaving other fields unchanged
.modify_no_read(                               // Write the value of one or more fields,
      original: LocalRegisterCopy<T, R>,       //  leaving other fields unchanged, but pass in
      value: FieldValue<T, R>)                 //  the original value, instead of doing a register read
.is_set(field: Field<T, R>) -> bool            // Check if one or more bits in a field are set
.matches_any(value: FieldValue<T, R>) -> bool  // Check if any specified parts of a field match
.matches_all(value: FieldValue<T, R>) -> bool  // Check if all specified parts of a field match
.extract() -> LocalRegisterCopy<T, R>          // Make local copy of register

Aliased<T: UIntLike, R: RegisterLongName = (), W: RegisterLongName = ()>: Readable + Writeable
.get() -> T                                    // Get the raw register value
.set(value: T)                                 // Set the raw register value
.read(field: Field<T, R>) -> T                 // Read the value of the given field
.read_as_enum<E>(field: Field<T, R>) -> Option<E> // Read value of the given field as a enum member
.write(value: FieldValue<T, W>)                // Write the value of one or more fields,
                                               //  overwriting other fields to zero
.is_set(field: Field<T, R>) -> bool            // Check if one or more bits in a field are set
.matches_any(value: FieldValue<T, R>) -> bool  // Check if any specified parts of a field match
.matches_all(value: FieldValue<T, R>) -> bool  // Check if all specified parts of a field match
.extract() -> LocalRegisterCopy<T, R>          // Make local copy of register
```

The `Aliased` type represents cases where read-only and write-only registers,
with different meanings, are aliased to the same memory location.

The first type parameter (the `UIntLike` type) is `u8`, `u16`, `u32`,
`u64`, `u128` or `usize`.

## Example: Using registers and bitfields

Assuming we have defined a `Registers` struct and the corresponding bitfields as
in the previous two sections. We also have an immutable reference to the
`Registers` struct, named `registers`.

```rust
// -----------------------------------------------------------------------------
// RAW ACCESS
// -----------------------------------------------------------------------------

// Get or set the raw value of the register directly. Nothing fancy:
registers.cr.set(registers.cr.get() + 1);


// -----------------------------------------------------------------------------
// READ
// -----------------------------------------------------------------------------

// `range` will contain the value of the RANGE field, e.g. 0, 1, 2, or 3.
// The type annotation is not necessary, but provided for clarity here.
let range: u8 = registers.cr.read(Control::RANGE);

// Or one can read `range` as a enum and `match` over it.
let range = registers.cr.read_as_enum(Control::RANGE);
match range {
    Some(Control::RANGE::Value::VeryHigh) => { /* ... */ }
    Some(Control::RANGE::Value::High) => { /* ... */ }
    Some(Control::RANGE::Value::Low) => { /* ... */ }

    None => unreachable!("invalid value")
}

// `en` will be 0 or 1
let en: u8 = registers.cr.read(Control::EN);


// -----------------------------------------------------------------------------
// MODIFY
// -----------------------------------------------------------------------------

// Write a value to a bitfield without altering the values in other fields:
registers.cr.modify(Control::RANGE.val(2)); // Leaves EN, INT unchanged

// Named constants can be used instead of the raw values:
registers.cr.modify(Control::RANGE::VeryHigh);

// Enum values can also be used:
registers.cr.modify(Control::RANGE::Value::VeryHigh.into())

// Another example of writing a field with a raw value:
registers.cr.modify(Control::EN.val(0)); // Leaves RANGE, INT unchanged

// For one-bit fields, the named values SET and CLEAR are automatically
// defined:
registers.cr.modify(Control::EN::SET);

// Write multiple values at once, without altering other fields:
registers.cr.modify(Control::EN::CLEAR + Control::RANGE::Low); // INT unchanged

// Any number of non-overlapping fields can be combined:
registers.cr.modify(Control::EN::CLEAR + Control::RANGE::High + CR::INT::SET);

// In some cases (such as a protected register) .modify() may not be appropriate.
// To enable updating a register without coupling the read and write, use
// modify_no_read():
let original = registers.cr.extract();
registers.cr.modify_no_read(original, Control::EN::CLEAR);


// -----------------------------------------------------------------------------
// WRITE
// -----------------------------------------------------------------------------

// Same interface as modify, except that all unspecified fields are overwritten to zero.
registers.cr.write(Control::RANGE.val(1)); // implictly sets all other bits to zero

// -----------------------------------------------------------------------------
// BITFLAGS
// -----------------------------------------------------------------------------

// For one-bit fields, easily check if they are set or clear:
let txcomplete: bool = registers.s.is_set(Status::TXCOMPLETE);

// -----------------------------------------------------------------------------
// MATCHING
// -----------------------------------------------------------------------------

// You can also query a specific register state easily with `matches_[any|all]`:

// Doesn't care about the state of any field except TXCOMPLETE and MODE:
let ready: bool = registers.s.matches_all(Status::TXCOMPLETE:SET +
                                     Status::MODE::FullDuplex);

// This is very useful for awaiting for a specific condition:
while !registers.s.matches_all(Status::TXCOMPLETE::SET +
                          Status::RXCOMPLETE::SET +
                          Status::TXINTERRUPT::CLEAR) {}

// Or for checking whether any interrupts are enabled:
let any_ints = registers.s.matches_any(Status::TXINTERRUPT + Status::RXINTERRUPT);

// Also you can read a register with set of enumerated values as a enum and `match` over it:
let mode = registers.cr.read_as_enum(Status::MODE);

match mode {
    Some(Status::MODE::Value::FullDuplex) => { /* ... */ }
    Some(Status::MODE::Value::HalfDuplex) => { /* ... */ }

    None => unreachable!("invalid value")
}

// -----------------------------------------------------------------------------
// LOCAL COPY
// -----------------------------------------------------------------------------

// More complex code may want to read a register value once and then keep it in
// a local variable before using the normal register interface functions on the
// local copy.

// Create a copy of the register value as a local variable.
let local = registers.cr.extract();

// Now all the functions for a ReadOnly register work.
let txcomplete: bool = local.is_set(Status::TXCOMPLETE);

// -----------------------------------------------------------------------------
// In-Memory Registers
// -----------------------------------------------------------------------------

// In some cases, code may want to edit a memory location with all of the
// register features described above, but the actual memory location is not a
// fixed MMIO register but instead an arbitrary location in memory. If this
// location is then shared with the hardware (i.e. via DMA) then the code
// must do volatile reads and writes since the value may change without the
// software knowing. To support this, the library includes an `InMemoryRegister`
// type.

let control: InMemoryRegister<u32, Control::Register> = InMemoryRegister::new(0)
control.write(Contol::BYTE_COUNT.val(0) +
              Contol::ENABLE::Yes +
              Contol::LENGTH.val(10));
```

Note that `modify` performs exactly one volatile load and one volatile store,
`write` performs exactly one volatile store, and `read` performs exactly one
volatile load. Thus, you are ensured that a single call will set or query all
fields simultaneously.

## Performance

Examining the binaries while testing this interface, everything compiles
down to the optimal inlined bit twiddling instructions--in other words, there is
zero runtime cost, as far as an informal preliminary study has found.

## Nice type checking

This interface helps the compiler catch some common types of bugs via type checking.

If you define the bitfields for e.g. a control register, you can give them a
descriptive group name like `Control`. This group of bitfields will only work
with a register of the type `ReadWrite<_, Control>` (or `ReadOnly/WriteOnly`,
etc). For instance, if we have the bitfields and registers as defined above,

```rust
// This line compiles, because registers.cr is associated with the Control group
// of bitfields.
registers.cr.modify(Control::RANGE.val(1));

// This line will not compile, because registers.s is associated with the Status
// group, not the Control group.
let range = registers.s.read(Control::RANGE);
```

## Naming conventions

There are several related names in the register definitions. Below is a
description of the naming convention for each:

```rust
use tock_registers::registers::ReadWrite;

#[repr(C)]
struct Registers {
    // The register name in the struct should be a lowercase version of the
    // register abbreviation, as written in the datasheet:
    cr: ReadWrite<u8, Control::Register>,
}

register_bitfields! [
    u8,

    // The name should be the long descriptive register name,
    // camelcase, without the word 'register'.
    Control [
        // The field name should be the capitalized abbreviated
        // field name, as given in the datasheet.
        RANGE OFFSET(4) NUMBITS(3) [
            // Each of the field values should be camelcase,
            // as descriptive of their value as possible.
            VeryHigh = 0,
            High = 1,
            Low = 2
        ]
    ]
]
```

## Implementing custom register types

The `Readable`, `Writeable` and `ReadWriteable` traits make it
possible to implement custom register types, even outside of this
crate. A particularly useful application area for this are CPU
registers, such as ARM SPSRs or RISC-V CSRs. It is sufficient to
implement the `Readable::get` and `Writeable::set` methods for the
rest of the API to be automatically implemented by the crate-provided
traits. For more in-depth documentation on how this works, [refer to
the `interfaces` module
documentation](https://docs.tockos.org/tock_registers/index.html).
