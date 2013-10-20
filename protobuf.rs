#[ link(name = "protobuf",
        vers = "0.0.1",
        uuid = "706F5A8D-ADCC-4237-92C6-6C1742C73141") ];

#[ desc = "protobuf support for rust" ];
#[ license = "ASL2" ];
#[ author = "Duane R Bailey" ];

#[ crate_type = "lib" ];

use std::io::{Reader, ReaderUtil};
use std::option::{Option};
use std::at_vec::to_managed;

#[cfg(test)]
use std::str;

trait Protobuf {
  fn Decode(&mut self, reader: @Reader) -> bool;
}

pub struct TagIter {
  reader: @Reader
}

static kWireMask: u64 = 0x7;
static kMSBMask: u64 = 0x80;
static kLS7BMask: u64  = 0x7F;

#[deriving(ToStr)]
enum WireType {
  kVarint = 0,
  k64Bit = 1,
  kLengthDelim = 2,
  kStartGroup = 3,
  kEndGroup = 4,
  k32Bit = 5
}

#[deriving(ToStr,Eq)]
pub enum TaggedValue {
  Varint(u64, u64),
  Fixed64(u64, u64),
  Raw(u64, @[u8]),
  StartGroup,
  EndGroup,
  Fixed32(u64, u32)
}

impl std::iter::Iterator<@TaggedValue> for TagIter {
  fn next(&mut self) -> Option<@TaggedValue> {
    let result_option = DecodeTagged(self.reader);
    if result_option == None {
      return None;
    }
    let result = result_option.unwrap();
    return Some(@result);
  }
}


fn IntToWireType(i: int) -> Option<WireType> {
  match i {
  0 => {Some(kVarint)}
  1 => {Some(k64Bit)}
  2 => {Some(kLengthDelim)}
  3 => {Some(kStartGroup)}
  4 => {Some(kEndGroup)}
  5 => {Some(k32Bit)}
  _ => {None}
  }
}

fn DecodeTagged(reader: @Reader) -> Option<TaggedValue> {
  let reader_util = @reader as @ReaderUtil;
  let wire_option = DecodeWire(reader);
  if wire_option.is_none() {
    return None;
  }
  let (wire, tag) = wire_option.unwrap();
  match wire {
    kVarint => {
      let varint = DecodeVarint(reader).unwrap();
      return Some(Varint(tag, varint));
    }
    kLengthDelim => {
      let length = DecodeVarint(reader).unwrap();
      return Some(Raw(tag, to_managed(reader_util.read_bytes(length as uint))));
    }
    k64Bit => {
      return Some(Fixed64(tag, reader_util.read_le_u64()));
    }
    k32Bit => {
      return Some(Fixed32(tag, reader_util.read_le_u32()));
    }
    _ => {
      return None;
    }
  }
}

#[test]
fn test_tag_decode() {
  let reader = std::io::BytesReader {
    bytes: &[0x08, 0x96, 0x1],
    pos: @mut 0
  };
  let tagged_val = DecodeTagged(@reader as @Reader).unwrap();
  match tagged_val {
    Varint(tag, i) => {
      assert!(i == 150);
    }
    _ => {
      fail!();
    }
  }
}

fn DecodeWire(reader: @Reader) -> Option<(WireType, u64)> {
  let read = reader.read_byte();
  if read < 0 {
    // EOF
    return None;
  }
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

fn DecodeVarint(reader: @Reader) -> Option<u64> {
  let mut shift = 0;
  let mut n_bytes = 0;
  let mut result: u64 = 0;
  loop {
    let read = reader.read_byte();
    if read < 0 {
      return None;
    }
    let byte: u64 = (read & 0xFF) as u64;
    let payload = kLS7BMask & byte;
    result |= (payload << shift);
    shift = shift + 7;
    n_bytes = n_bytes + 1;
    if (byte & kMSBMask) == 0x0 {
      break;
    }
  }
  return Some(result);
}

#[test]
fn test_tag_iter() {
  let reader = std::io::BytesReader {
    bytes: &[0x8, 0xf8, 0xac, 0xd1, 0x91,
             0x1, 0x11, 0x78, 0x56, 0x34,
             0x12, 0x0, 0x0, 0x0, 0x0,
             0x1a, 0xc, 0x68, 0x65, 0x6c,
             0x6c, 0x6f, 0x2c, 0x20, 0x77,
             0x6f, 0x72, 0x6c, 0x64, 0x25,
             0x78, 0x56, 0x34, 0x12],
    pos: @mut 0
  };
  let mut iter = TagIter{reader: @reader as @Reader};
  match iter.next().unwrap() {
    @Varint(1, 0x12345678) => {}
    _ => { fail!() }
  }
  match iter.next().unwrap() {
    @Fixed64(2, 0x12345678) => {}
    _ => { fail!() }
  }
  match iter.next().unwrap() {
    @Raw(3, arr) => {
      assert!(str::eq(&from_utf8(arr), &~"hello, world"));
    }
    _ => { fail!() }
  }
  match iter.next().unwrap() {
    @Fixed32(4, 0x12345678) => {}
    _ => { fail!() }
  }
  assert!(iter.next().is_none());
}

/*#[test]
fn blah_container() {
  // nary
  let reader = std::io::BytesReader {
    bytes: &{0x8, 0xf8, 0xac, 0xd1, 0x91,
             0x1, 0x18, 0xf8, 0xac, 0xd1,
             0x91, 0x1, 0x20, 0xf8, 0xac,
             0xd1, 0x91, 0x1, 0x20, 0xf8,
             0xac, 0xd1, 0x91, 0x1, 0x20,
             0xf8, 0xac, 0xd1, 0x91, 0x1,
             0x20, 0xf8, 0xac, 0xd1, 0x91,
             0x1, 0x30, 0xf8, 0xac, 0xd1,
             0x91, 0x1};
    pos: @mut 0
  };
}*/
