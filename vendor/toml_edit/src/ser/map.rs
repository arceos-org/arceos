use super::{Error, KeySerializer};

#[doc(hidden)]
pub enum SerializeMap {
    Datetime(SerializeDatetime),
    Table(SerializeInlineTable),
}

impl SerializeMap {
    pub(crate) fn table() -> Self {
        Self::Table(SerializeInlineTable::new())
    }

    pub(crate) fn table_with_capacity(len: usize) -> Self {
        Self::Table(SerializeInlineTable::with_capacity(len))
    }

    pub(crate) fn datetime() -> Self {
        Self::Datetime(SerializeDatetime::new())
    }
}

impl serde::ser::SerializeMap for SerializeMap {
    type Ok = crate::Value;
    type Error = Error;

    fn serialize_key<T: ?Sized>(&mut self, input: &T) -> Result<(), Self::Error>
    where
        T: serde::ser::Serialize,
    {
        match self {
            Self::Datetime(s) => s.serialize_key(input),
            Self::Table(s) => s.serialize_key(input),
        }
    }

    fn serialize_value<T: ?Sized>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: serde::ser::Serialize,
    {
        match self {
            Self::Datetime(s) => s.serialize_value(value),
            Self::Table(s) => s.serialize_value(value),
        }
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        match self {
            Self::Datetime(s) => s.end().map(|items| items.into()),
            Self::Table(s) => s.end().map(|items| items.into()),
        }
    }
}

impl serde::ser::SerializeStruct for SerializeMap {
    type Ok = crate::Value;
    type Error = Error;

    fn serialize_field<T: ?Sized>(
        &mut self,
        key: &'static str,
        value: &T,
    ) -> Result<(), Self::Error>
    where
        T: serde::ser::Serialize,
    {
        match self {
            Self::Datetime(s) => s.serialize_field(key, value),
            Self::Table(s) => s.serialize_field(key, value),
        }
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        match self {
            Self::Datetime(s) => s.end().map(|items| items.into()),
            Self::Table(s) => s.end().map(|items| items.into()),
        }
    }
}

#[doc(hidden)]
pub struct SerializeDatetime {
    value: Option<crate::Datetime>,
}

impl SerializeDatetime {
    pub(crate) fn new() -> Self {
        Self { value: None }
    }
}

impl serde::ser::SerializeMap for SerializeDatetime {
    type Ok = crate::Datetime;
    type Error = Error;

    fn serialize_key<T: ?Sized>(&mut self, _input: &T) -> Result<(), Self::Error>
    where
        T: serde::ser::Serialize,
    {
        unreachable!("datetimes should only be serialized as structs, not maps")
    }

    fn serialize_value<T: ?Sized>(&mut self, _value: &T) -> Result<(), Self::Error>
    where
        T: serde::ser::Serialize,
    {
        unreachable!("datetimes should only be serialized as structs, not maps")
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        unreachable!("datetimes should only be serialized as structs, not maps")
    }
}

impl serde::ser::SerializeStruct for SerializeDatetime {
    type Ok = crate::Datetime;
    type Error = Error;

    fn serialize_field<T: ?Sized>(
        &mut self,
        key: &'static str,
        value: &T,
    ) -> Result<(), Self::Error>
    where
        T: serde::ser::Serialize,
    {
        if key == toml_datetime::__unstable::FIELD {
            self.value = Some(value.serialize(DatetimeFieldSerializer::default())?);
        }

        Ok(())
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        self.value.ok_or(Error::UnsupportedNone)
    }
}

#[doc(hidden)]
pub struct SerializeInlineTable {
    items: crate::table::KeyValuePairs,
    key: Option<crate::InternalString>,
}

impl SerializeInlineTable {
    pub(crate) fn new() -> Self {
        Self {
            items: Default::default(),
            key: Default::default(),
        }
    }

    pub(crate) fn with_capacity(len: usize) -> Self {
        let mut s = Self::new();
        s.items.reserve(len);
        s
    }
}

impl serde::ser::SerializeMap for SerializeInlineTable {
    type Ok = crate::InlineTable;
    type Error = Error;

    fn serialize_key<T: ?Sized>(&mut self, input: &T) -> Result<(), Self::Error>
    where
        T: serde::ser::Serialize,
    {
        self.key = None;
        self.key = Some(input.serialize(KeySerializer)?);
        Ok(())
    }

    fn serialize_value<T: ?Sized>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: serde::ser::Serialize,
    {
        let res = value.serialize(super::ValueSerializer {});
        match res {
            Ok(item) => {
                let key = self.key.take().unwrap();
                let kv = crate::table::TableKeyValue::new(
                    crate::Key::new(&key),
                    crate::Item::Value(item),
                );
                self.items.insert(key, kv);
            }
            Err(e) => {
                if e != Error::UnsupportedNone {
                    return Err(e);
                }
            }
        }
        Ok(())
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(crate::InlineTable::with_pairs(self.items))
    }
}

impl serde::ser::SerializeStruct for SerializeInlineTable {
    type Ok = crate::InlineTable;
    type Error = Error;

    fn serialize_field<T: ?Sized>(
        &mut self,
        key: &'static str,
        value: &T,
    ) -> Result<(), Self::Error>
    where
        T: serde::ser::Serialize,
    {
        let res = value.serialize(super::ValueSerializer {});
        match res {
            Ok(item) => {
                let kv = crate::table::TableKeyValue::new(
                    crate::Key::new(key),
                    crate::Item::Value(item),
                );
                self.items.insert(crate::InternalString::from(key), kv);
            }
            Err(e) => {
                if e != Error::UnsupportedNone {
                    return Err(e);
                }
            }
        };
        Ok(())
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(crate::InlineTable::with_pairs(self.items))
    }
}

#[derive(Default)]
struct DatetimeFieldSerializer {}

impl serde::ser::Serializer for DatetimeFieldSerializer {
    type Ok = toml_datetime::Datetime;
    type Error = Error;
    type SerializeSeq = serde::ser::Impossible<Self::Ok, Self::Error>;
    type SerializeTuple = serde::ser::Impossible<Self::Ok, Self::Error>;
    type SerializeTupleStruct = serde::ser::Impossible<Self::Ok, Self::Error>;
    type SerializeTupleVariant = serde::ser::Impossible<Self::Ok, Self::Error>;
    type SerializeMap = serde::ser::Impossible<Self::Ok, Self::Error>;
    type SerializeStruct = serde::ser::Impossible<Self::Ok, Self::Error>;
    type SerializeStructVariant = serde::ser::Impossible<Self::Ok, Self::Error>;

    fn serialize_bool(self, _value: bool) -> Result<Self::Ok, Self::Error> {
        Err(Error::DateInvalid)
    }

    fn serialize_i8(self, _value: i8) -> Result<Self::Ok, Self::Error> {
        Err(Error::DateInvalid)
    }

    fn serialize_i16(self, _value: i16) -> Result<Self::Ok, Self::Error> {
        Err(Error::DateInvalid)
    }

    fn serialize_i32(self, _value: i32) -> Result<Self::Ok, Self::Error> {
        Err(Error::DateInvalid)
    }

    fn serialize_i64(self, _value: i64) -> Result<Self::Ok, Self::Error> {
        Err(Error::DateInvalid)
    }

    fn serialize_u8(self, _value: u8) -> Result<Self::Ok, Self::Error> {
        Err(Error::DateInvalid)
    }

    fn serialize_u16(self, _value: u16) -> Result<Self::Ok, Self::Error> {
        Err(Error::DateInvalid)
    }

    fn serialize_u32(self, _value: u32) -> Result<Self::Ok, Self::Error> {
        Err(Error::DateInvalid)
    }

    fn serialize_u64(self, _value: u64) -> Result<Self::Ok, Self::Error> {
        Err(Error::DateInvalid)
    }

    fn serialize_f32(self, _value: f32) -> Result<Self::Ok, Self::Error> {
        Err(Error::DateInvalid)
    }

    fn serialize_f64(self, _value: f64) -> Result<Self::Ok, Self::Error> {
        Err(Error::DateInvalid)
    }

    fn serialize_char(self, _value: char) -> Result<Self::Ok, Self::Error> {
        Err(Error::DateInvalid)
    }

    fn serialize_str(self, v: &str) -> Result<Self::Ok, Self::Error> {
        v.parse::<toml_datetime::Datetime>().map_err(Error::custom)
    }

    fn serialize_bytes(self, _value: &[u8]) -> Result<Self::Ok, Self::Error> {
        Err(Error::DateInvalid)
    }

    fn serialize_none(self) -> Result<Self::Ok, Self::Error> {
        Err(Error::DateInvalid)
    }

    fn serialize_some<T: ?Sized>(self, _value: &T) -> Result<Self::Ok, Self::Error>
    where
        T: serde::ser::Serialize,
    {
        Err(Error::DateInvalid)
    }

    fn serialize_unit(self) -> Result<Self::Ok, Self::Error> {
        Err(Error::DateInvalid)
    }

    fn serialize_unit_struct(self, _name: &'static str) -> Result<Self::Ok, Self::Error> {
        Err(Error::DateInvalid)
    }

    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
    ) -> Result<Self::Ok, Self::Error> {
        Err(Error::DateInvalid)
    }

    fn serialize_newtype_struct<T: ?Sized>(
        self,
        _name: &'static str,
        _value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: serde::ser::Serialize,
    {
        Err(Error::DateInvalid)
    }

    fn serialize_newtype_variant<T: ?Sized>(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: serde::ser::Serialize,
    {
        Err(Error::DateInvalid)
    }

    fn serialize_seq(self, _len: Option<usize>) -> Result<Self::SerializeSeq, Self::Error> {
        Err(Error::DateInvalid)
    }

    fn serialize_tuple(self, _len: usize) -> Result<Self::SerializeTuple, Self::Error> {
        Err(Error::DateInvalid)
    }

    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleStruct, Self::Error> {
        Err(Error::DateInvalid)
    }

    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleVariant, Self::Error> {
        Err(Error::DateInvalid)
    }

    fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap, Self::Error> {
        Err(Error::DateInvalid)
    }

    fn serialize_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStruct, Self::Error> {
        Err(Error::DateInvalid)
    }

    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStructVariant, Self::Error> {
        Err(Error::DateInvalid)
    }
}
