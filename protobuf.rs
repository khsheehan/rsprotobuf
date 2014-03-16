#[crate_id = "protobuf#0.0.1"];

#[desc = "protobuf support for rust"];
#[license = "ASL2"];

#[crate_type = "lib"];

pub mod Protobuf {

use std::io::{Reader, MemReader};
use std::iter::Iterator;
use std::option::Option;
use std::str::from_utf8;
use std::vec_ng::Vec;

pub trait Protobuf {
  fn Decode<'a>(&mut self, reader: &'a mut Reader) -> bool;
}

pub struct TagIter<'a> {
  reader: &'a mut Reader
}

static kWireMask: u64 = 0x7;
static kMSBMask: u64 = 0x80;
static kLS7BMask: u64  = 0x7F;

#[deriving(Show)]
enum WireType {
  VarintWireType = 0,
  Fixed64WireType = 1,
  LengthDelimWireType = 2,
  StartGroupWireType = 3,
  EndGroupWireType = 4,
  Fixed32WireType = 5
}

#[deriving(Show,Eq)]
pub enum TaggedValue {
  Varint(u64, u64),
  Fixed64(u64, u64),
  Raw(u64, Vec<u8>),
  StartGroup,
  EndGroup,
  Fixed32(u64, u32)
}

impl<'a> Iterator<TaggedValue> for TagIter<'a> {
  fn next(&mut self) -> Option<TaggedValue> {
    let result_option = DecodeTagged(self.reader);
    if result_option == None {
      return None;
    }
    return Some(result_option.unwrap());
  }
}


fn IntToWireType(i: int) -> Option<WireType> {
  match i {
  0 => {Some(VarintWireType)}
  1 => {Some(Fixed64WireType)}
  2 => {Some(LengthDelimWireType)}
  3 => {Some(StartGroupWireType)}
  4 => {Some(EndGroupWireType)}
  5 => {Some(Fixed32WireType)}
  _ => {None}
  }
}

#[allow(deprecated_owned_vector)]
fn DecodeTagged(reader: &mut Reader) -> Option<TaggedValue> {
  let wire_option = DecodeWire(reader);
  if wire_option.is_none() {
    return None;
  }
  let (wire, tag) = wire_option.unwrap();
  match wire {
    VarintWireType => {
      let varint = DecodeVarint(reader).unwrap();
      return Some(Varint(tag, varint));
    }
    LengthDelimWireType => {
      let length = DecodeVarint(reader).unwrap();
      let bytes = reader.read_bytes(length as uint).unwrap();
      return Some(Raw(tag, Vec::from_slice(bytes.as_slice())));
    }
    Fixed64WireType => {
      return Some(Fixed64(tag, reader.read_le_u64().unwrap()));
    }
    Fixed32WireType => {
      return Some(Fixed32(tag, reader.read_le_u32().unwrap()));
    }
    _ => {
      return None;
    }
  }
}

#[allow(deprecated_owned_vector)]
#[test]
fn test_tag_decode() {
  let mut reader = MemReader::new(~[0x08, 0x96, 0x1]);
  let tagged_val = DecodeTagged(&mut reader).unwrap();
  match tagged_val {
    Varint(1, i) => {
      assert!(i == 150);
    }
    _ => {
      fail!();
    }
  }
}


fn DecodeWire<'a>(reader: &'a mut Reader) -> Option<(WireType, u64)> {
  let readResult = reader.read_byte();
  if readResult.is_err() {
    return None;
  }
  let read = readResult.unwrap() as int;
  let wire_int = read & (kWireMask as int);
  let wire = IntToWireType(wire_int).unwrap();
  let mut tag: u64 = ((read & kLS7BMask as int) >> 3) as u64;
  if (read & kMSBMask as int) != 0x0 {
    match DecodeVarint(reader) {
      Some(integer) => {
        tag = tag | (integer << 4);
      }
      None => {
        return None;
      }
    }
  }
  return Some((wire, tag));
}

fn DecodeVarint<'a>(reader: &'a mut Reader) -> Option<u64> {
  let mut shift = 0;
  let mut n_bytes = 0;
  let mut result: u64 = 0;
  loop {
    let readResult = reader.read_byte();
    if readResult.is_err() {
      return None;
    }
    let read = readResult.unwrap();
    let byte: u64 = (read & 0xFF) as u64;
    let payload = kLS7BMask & byte;
    result |= payload << shift;
    shift = shift + 7;
    n_bytes = n_bytes + 1;
    if (byte & kMSBMask) == 0x0 {
      break;
    }
  }
  return Some(result);
}

#[test]
#[allow(deprecated_owned_vector)]
fn test_tag_iter() {
  let mut reader = MemReader::new(~[
             0x8, 0xf8, 0xac, 0xd1, 0x91,
             0x1, 0x11, 0x78, 0x56, 0x34,
             0x12, 0x0, 0x0, 0x0, 0x0,
             0x1a, 0xc, 0x68, 0x65, 0x6c,
             0x6c, 0x6f, 0x2c, 0x20, 0x77,
             0x6f, 0x72, 0x6c, 0x64, 0x25,
             0x78, 0x56, 0x34, 0x12]);
  let mut iter = TagIter{reader: &mut reader};
  match iter.next().unwrap() {
    Varint(1, 0x12345678) => {}
    _ => { fail!() }
  }
  match iter.next().unwrap() {
    Fixed64(2, 0x12345678) => {}
    _ => { fail!() }
  }
  match iter.next().unwrap() {
    Raw(3, arr) => {
      assert!(from_utf8(arr.as_slice()).get_ref().eq(& &"hello, world"));
    }
    _ => { fail!() }
  }
  match iter.next().unwrap() {
    Fixed32(4, 0x12345678) => {}
    _ => { fail!() }
  }
  assert!(iter.next().is_none());
}

}