use std::io::{file_reader, Reader, ReaderUtil};
use std::option::{Option};
use std::hashmap::HashMap;
use std::at_vec::to_managed;
use std::str::raw::from_utf8;

static kWireMask: u64 = 0x7;
static kMSBMask: u64 = 0x80;
static kLS7BMask: u64  = 0x7F;

struct TagIter {
  reader: @Reader
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
enum TaggedValue {
  Varint(u64, u64),
  Fixed64(u64, u64),
  Raw(u64, @[u8]),
  StartGroup,
  EndGroup,
  Fixed32(u64, u32)
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
  let (wire, tag) = DecodeWire(reader).unwrap();
  println(format!("Wire: {:?}, tag: {:u}", wire, tag));
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
      println(format!("Didn't know how to decode {:?}", wire));
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
    println("Ran out of characters.");
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
        println("Failed to read varint.");
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
      println("Failed to decode varint because ran out of bytes.");
      return None;
    }
    let byte: u64 = (read & 0xFF) as u64;
    println(format!("byte: {:u}", byte));
    let payload = kLS7BMask & byte;
    println(format!("payload: {:u}", payload));
    result |= (payload << shift);
    println(format!("result: {:u}", result));
    shift = shift + 7;
    n_bytes = n_bytes + 1;
    if (byte & kMSBMask) == 0x0 {
      break;
    }
  }
  println(format!("Read {:d} bytes with payload {:u}.", n_bytes, result));
  return Some(result);
}

struct CodeGeneratorRequest {
  file_to_generate: ~[~str],
  parameter: Option<~str>,
  proto_file: ~[u8]
}

fn DecodeCGR(reader: @Reader) -> Option<~CodeGeneratorRequest> {
  let mut file_to_generate: ~[~str] = ~[];
  let file_to_generate_tag = 1;
  unsafe{
  for tag_value in TagIter{reader: reader} {
    match tag_value {
      @Raw(1, bytes) => {println(format!("file_to_generate: {:?}", from_utf8(bytes)));}
      _ => {return None;}
    }
  }
}
  return None;
}

fn main() {
  let reader = file_reader(&PosixPath("in.blah")).unwrap();
  let opt = DecodeCGR(reader);
}
