use BitIndex;

#[test]
fn bit() {
    assert_eq!(0b00000001u8.bit(0), true);
    assert_eq!(0b10000000u8.bit(7), true);
    assert_eq!(0b11111110u8.bit(0), false);
    assert_eq!(0b01111111u8.bit(7), false);
}

#[test]
#[should_panic]
fn bit_panic() {
    0u8.bit(8);
}

#[test]
fn set_bit() {
    assert_eq!(*0b00000001u8.set_bit(0, false), 0);
    assert_eq!(*0b10000000u8.set_bit(7, false), 0);
    assert_eq!(*0b11111110u8.set_bit(0, true), 0b11111111);
    assert_eq!(*0b01111111u8.set_bit(7, true), 0b11111111);
    assert_eq!(*0b00000001u8.set_bit(0, true), 1);
    assert_eq!(*0b10000000u8.set_bit(7, true), 0b10000000);
    assert_eq!(*0b11111110u8.set_bit(0, false), 0b11111110);
    assert_eq!(*0b01111111u8.set_bit(7, false), 0b01111111);
}

#[test]
#[should_panic]
fn set_bit_panic() {
    0u8.set_bit(8, false);
}

#[test]
fn bit_range() {
    assert_eq!(0b10101010u8.bit_range(0..3), 0b10);
    assert_eq!(0b10101010u8.bit_range(4..8), 0b1010);
}

#[test]
#[should_panic]
fn bit_range_panic() {
    0u8.bit_range(5..9);
}

#[test]
fn set_bit_range() {
    assert_eq!(*0b10101010u8.set_bit_range(0..3, 0b110), 0b10101110);
    assert_eq!(*0b10101010u8.set_bit_range(4..8, 0b1100), 0b11001010);
}

#[test]
#[should_panic]
fn set_bit_range_bounds_panic() {
    0u8.set_bit_range(5..9, 0);
}

#[test]
#[should_panic]
fn set_bit_value_length_panic() {
    0u8.set_bit_range(5..9, 0b11111);
}
