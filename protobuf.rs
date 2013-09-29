#[ link(name = "protobuf",
        vers = "0.0.1",
        uuid = "706F5A8D-ADCC-4237-92C6-6C1742C73141") ];

#[ desc = "protobuf support for rust" ];
#[ license = "ASL2" ];
#[ author = "Duane R Bailey" ];

#[ crate_type = "lib" ];

use std::libc::{c_void, size_t};

struct ProtobufCIntRange {
  start_value: i32,
  orig_index: u32
}

type ProtobufCMessageInit = extern fn (message: *ProtobufCMessage);

struct ProtobufCMessageDescriptor {
  magic: u32,

  name: *i8,
  short_name: *i8,
  c_name: *i8,
  package_name: *i8,

  sizeof_message: size_t,

  n_fields: uint,
  fields: *ProtobufCFieldDescriptor,
  fields_sorted_by_name: * uint,

  n_field_ranges: uint,
  field_ranges: *ProtobufCIntRange,

  message_init: ProtobufCMessageInit,
  reserved1: *c_void,
  reserved2: *c_void,
  reserved3: *c_void
}

struct ProtobufCMessage {
  descriptor: *ProtobufCMessageDescriptor,
  n_unknown_fields: uint,
  unknown_fields: *ProtobufCMessageUnknownField
}

enum ProtobufCWireType {
  PROTOBUF_C_WIRE_TYPE_VARINT,
  PROTOBUF_C_WIRE_TYPE_64BIT,
  PROTOBUF_C_WIRE_TYPE_LENGTH_PREFIXED,
  PROTOBUF_C_WIRE_TYPE_START_GROUP,     /* unsupported */
  PROTOBUF_C_WIRE_TYPE_END_GROUP,       /* unsupported */
  PROTOBUF_C_WIRE_TYPE_32BIT
}

struct ProtobufCMessageUnknownField {
  tag: u32,
  wire_type: ProtobufCWireType,
  len: size_t,
  data: *u8
}

struct ProtobufCFieldDescriptor {
  name: *u8,
  id: u32,
  label: ProtobufCLabel,
  ty: ProtobufCType,
  quantifier_offset: uint,
  offset: uint,
  descriptor: *c_void, /* for MESSAGE and ENUM types */
  default_value: *c_void, /* or NULL if no default-value */
  packed: bool,

  reserved_flags: uint,
  reserved2: *c_void,
  reserved3: *c_void
}

enum ProtobufCLabel {
  PROTOBUF_C_LABEL_REQUIRED,
  PROTOBUF_C_LABEL_OPTIONAL,
  PROTOBUF_C_LABEL_REPEATED
}

enum ProtobufCType {
  PROTOBUF_C_TYPE_INT32,
  PROTOBUF_C_TYPE_SINT32,
  PROTOBUF_C_TYPE_SFIXED32,
  PROTOBUF_C_TYPE_INT64,
  PROTOBUF_C_TYPE_SINT64,
  PROTOBUF_C_TYPE_SFIXED64,
  PROTOBUF_C_TYPE_UINT32,
  PROTOBUF_C_TYPE_FIXED32,
  PROTOBUF_C_TYPE_UINT64,
  PROTOBUF_C_TYPE_FIXED64,
  PROTOBUF_C_TYPE_FLOAT,
  PROTOBUF_C_TYPE_DOUBLE,
  PROTOBUF_C_TYPE_BOOL,
  PROTOBUF_C_TYPE_ENUM,
  PROTOBUF_C_TYPE_STRING,
  PROTOBUF_C_TYPE_BYTES,
  //PROTOBUF_C_TYPE_GROUP,          // NOT SUPPORTED
  PROTOBUF_C_TYPE_MESSAGE,
}

struct ProtobufCAllocator {
  alloc: extern fn(allocator_data: *c_void, size: size_t) -> *c_void,
  free: extern fn(allocator_data: *c_void, pointer: *c_void),
  tmp_alloc: extern fn(allocator_data: *c_void, size: size_t) -> *c_void,
  max_alloca: uint,
  allocator_data: *c_void
}

struct CCodeGeneratorRequest {
  base: ProtobufCMessage,
  n_files_to_generate: size_t,
  files_to_generate: **i8,
  parameter: *i8,
  n_proto_file: size_t,
  proto_file: **c_void
}

extern {

fn google__protobuf__compiler__code_generator_request__get_packed_size(message: *CCodeGeneratorRequest) -> size_t;

fn google__protobuf__compiler__code_generator_request__unpack(allocator: *ProtobufCAllocator, len: size_t, data: *u8) -> *CCodeGeneratorRequest;

fn google__protobuf__compiler__code_generator_request__free_unpacked(message: *CCodeGeneratorRequest, allocator: *ProtobufCAllocator);

}

#[cfg(test)]
#[link_args="-lproto -lprotobuf-c"]
#[fixed_stack_segment]
#[inline(never)]
#[test]
fn test_unpack() {
  use std::io::file_reader;
  use std::path::PosixPath;
  use std::ptr::null;
  use std::vec::raw::{to_ptr, from_buf_raw};
  use std::str::raw::from_c_str;

  let path = &PosixPath("./testdata/CodeGenRequest.request");
  let reader = file_reader(path).unwrap();
  let packed = reader.read_whole_stream();
  unsafe {
    let packed_len = packed.len() as size_t;
    let packed_ptr = to_ptr(packed);
    let code_gen_ptr = google__protobuf__compiler__code_generator_request__unpack(null(), packed_len, packed_ptr);
    let bytes_vec = from_buf_raw((*code_gen_ptr).files_to_generate, (*code_gen_ptr).n_files_to_generate as uint);
    for c_string in bytes_vec.iter() {
      let string = from_c_str(*c_string);
      println(string);
    }
  }
}