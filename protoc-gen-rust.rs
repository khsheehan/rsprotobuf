extern mod protobuf;
extern mod extra;

use std::io::{stdin, Reader, with_bytes_reader};
use std::str::from_utf8;
use protobuf::{Protobuf, TagIter, Raw, Varint};

struct CodeGeneratorRequest {
  file_to_generate: ~[~str],
  parameter: Option<~str>,
  proto_file: ~[FileDescriptorProto]
}

struct FileDescriptorProto {
  name: Option<~str>,
  package: Option<~str>,
  message_type: ~[DescriptorProto]
}

struct DescriptorProto {
  name: Option<~str>, // 1
  field: ~[FieldDescriptorProto], // 2
  nested_type: ~[DescriptorProto], // 3
}

#[deriving(ToStr)]
enum FieldDescriptorProto_Type {
    TYPE_DOUBLE         = 1,
    TYPE_FLOAT          = 2,
    // Not ZigZag encoded.  Negative numbers take 10 bytes.  Use TYPE_SINT64 if
    // negative values are likely.
    TYPE_INT64          = 3,
    TYPE_UINT64         = 4,
    // Not ZigZag encoded.  Negative numbers take 10 bytes.  Use TYPE_SINT32 if
    // negative values are likely.
    TYPE_INT32          = 5,
    TYPE_FIXED64        = 6,
    TYPE_FIXED32        = 7,
    TYPE_BOOL           = 8,
    TYPE_STRING         = 9,
    TYPE_GROUP          = 10,  // Tag-delimited aggregate.
    TYPE_MESSAGE        = 11,  // Length-delimited aggregate.

    // New in version 2.
    TYPE_BYTES          = 12,
    TYPE_UINT32         = 13,
    TYPE_ENUM           = 14,
    TYPE_SFIXED32       = 15,
    TYPE_SFIXED64       = 16,
    TYPE_SINT32         = 17,  // Uses ZigZag encoding.
    TYPE_SINT64         = 18, 
}

fn type_from_u64(u: u64) -> FieldDescriptorProto_Type {
  match u {
    1 => TYPE_DOUBLE,
    2 => TYPE_FLOAT,
    3 => TYPE_INT64,
    4 => TYPE_UINT64,
    5 => TYPE_INT32,
    6 => TYPE_FIXED64,
    7 => TYPE_FIXED32,
    8 => TYPE_BOOL,
    9 => TYPE_STRING,
    10 => TYPE_GROUP,
    11 => TYPE_MESSAGE,
    12 => TYPE_BYTES,
    13 => TYPE_UINT32,
    14 => TYPE_ENUM,
    15 => TYPE_SFIXED32,
    16 => TYPE_SFIXED64,
    17 => TYPE_SINT32,
    18 => TYPE_SINT64,
    _ => {fail!()},
  }
}

#[deriving(ToStr)]
enum FieldDescriptorProto_Label {
  LABEL_OPTIONAL      = 1,
  LABEL_REQUIRED      = 2,
  LABEL_REPEATED      = 3,
}

fn label_from_u64(u: u64) -> FieldDescriptorProto_Label {
  match u {
    1 => LABEL_OPTIONAL,
    2 => LABEL_REQUIRED,
    3 => LABEL_REPEATED,
    _ => {fail!(u.to_str())},
  }
}

struct FieldDescriptorProto {
  name: Option<~str>,
  number: Option<i32>,
  label: Option<FieldDescriptorProto_Label>,
  Type: Option<FieldDescriptorProto_Type>,
  type_name: Option<~str>,
  default_value: Option<~str>,
}

impl FieldDescriptorProto {
  fn BuildTreeLines(&self, depth: uint) -> ~str {
    let padding = "\t".repeat(depth);
    let Type = match self.Type.unwrap() {
      TYPE_ENUM => {
        (*self.type_name.get_ref()).clone()
      }
      id => id.to_str()
    };
    let default = if self.default_value.is_some() { format!(" [default = \"{:s}\"]", *self.default_value.get_ref()) } else { ~"" };
    return format!("{:s}{:s} {:s} {:s} = {:d}{:s};",
                   padding,
                   (*self.label.get_ref()).to_str(),
                   Type,
                   (*self.name.get_ref()),
                   *self.number.get_ref(),
                   default);
  }
}

impl ToStr for FieldDescriptorProto {
  fn to_str(&self) -> ~str {
    return self.BuildTreeLines(0);
  }
}

impl Protobuf for CodeGeneratorRequest {
  fn Decode(&mut self, reader: @Reader) -> bool {
    for tag_option in TagIter{reader: reader} {
      match tag_option {
        @Raw(1, data) => {
          self.file_to_generate.push(from_utf8(data));
        }
        @Raw(2, parameter) => {
          assert!(self.parameter.is_none());
          self.parameter = Some(from_utf8(parameter));
        }
        @Raw(15, proto_file) => {
          let reader = do with_bytes_reader(proto_file) |reader| { reader };
          let mut fd_proto = FileDescriptorProto{
            name: None,
            package: None,
            message_type: ~[]
          };
          assert!(fd_proto.Decode(reader));
          self.proto_file.push(fd_proto);
        }
        _ => {
          //println(format!("Unknown tag: {:?}", tag_option));
        }
      }
    }
    return true;
  }
}

impl ToStr for CodeGeneratorRequest {
  fn to_str(&self) -> ~str {
    let mut buf = ~"";
    if self.file_to_generate.len() > 0 {
      buf.push_str(format!("Files to generate:\n{:s}", self.file_to_generate.connect("\n\t")));
    }
    if self.proto_file.len() > 0 {
      buf.push_str(format!("\n\nFile descriptor protos:\n{:s}", self.proto_file.map(|proto_file| {proto_file.to_str()}).connect("\n\n")));
    }
    return buf;
  }
}

impl ToStr for FileDescriptorProto {
  fn to_str(&self) -> ~str {
    let mut buf = format!("File \"{:s}\":\n\n", *self.name.get_ref());

    if !self.package.is_none() {
      buf.push_str(format!("package {:s};\n", *self.package.get_ref()));
    }

    if self.message_type.len() > 0 {
      buf.push_str(format!("\n\n{:s}\n", self.message_type.map(|message_type|{message_type.to_str()}).connect("\n\n")));
    }

    return buf;
  }
}

impl DescriptorProto {
  fn BuildTreeLines(&self, depth: uint) -> ~str {
    let padding = "\t".repeat(depth);

    let mut buf = format!("{:s}message {:s} \\{", padding, *self.name.get_ref());

    for field in self.field.iter() {
      buf.push_str(format!("\n{:s}", field.BuildTreeLines(depth + 1)));
    }

    for nested_type in self.nested_type.iter() {
      buf.push_str(format!("\n{:s}", nested_type.BuildTreeLines(depth + 1)));
    }

    buf.push_str(format!("\n{:s}\\}", padding));
    return buf;
  }
}

impl ToStr for DescriptorProto {
  fn to_str(&self) -> ~str {
    return self.BuildTreeLines(0);
  }
}

impl Protobuf for FileDescriptorProto {
  fn Decode(&mut self, reader: @Reader) -> bool {
    for tag_option in TagIter{reader: reader} {
      match tag_option {
        @Raw(1, name) => {
          assert!(self.name.is_none());
          self.name = Some(from_utf8(name))
        }
        @Raw(2, package) => {
          assert!(self.package.is_none());
          self.package = Some(from_utf8(package));
        }
        @Raw(4, message_type) => {
          let reader = do with_bytes_reader(message_type) |reader| { reader };
          let mut desc_proto = DescriptorProto{
            name: None,
            field: ~[],
            nested_type: ~[]
          };
          assert!(desc_proto.Decode(reader));
          self.message_type.push(desc_proto)
        }
        _ => {
          //println(format!("Unknown tag: {:?}", tag_option));
        }
      }
    }
    return true;
  }
}

impl Protobuf for DescriptorProto {
  fn Decode(&mut self, reader: @Reader) -> bool {
    for tag_option in TagIter{reader: reader} {
      match tag_option {
        @Raw(1, name) => {
          assert!(self.name.is_none());
          self.name = Some(from_utf8(name))
        }
        @Raw(2, field) => {
          let reader = do with_bytes_reader(field) |reader| { reader };
          let mut field_proto = FieldDescriptorProto{
            name: None,
            number: None,
            label: None,
            Type: None,
            type_name: None,
            default_value: None,
          };
          assert!(field_proto.Decode(reader));
          self.field.push(field_proto)
        }
        @Raw(3, nested_type) => {
          let reader = do with_bytes_reader(nested_type) |reader| { reader };
          let mut desc_proto = DescriptorProto{
            name: None,
            field: ~[],
            nested_type: ~[]
          };
          assert!(desc_proto.Decode(reader));
          self.nested_type.push(desc_proto)
        }
        _ => {
          //println(format!("Unknown tag: {:?}", tag_option));
        }
      }
    }
    return true;
  }
}

impl Protobuf for FieldDescriptorProto {
  fn Decode(&mut self, reader: @Reader) -> bool {
    for tag_option in TagIter{reader: reader} {
      match tag_option {
        @Raw(1, name) => {
          assert!(self.name.is_none());
          self.name = Some(from_utf8(name))
        }
        @Varint(3, number) => {
          assert!(self.number.is_none());
          self.number = Some(number as i32);
        }
        @Varint(4, label) => {
          assert!(self.label.is_none());
          self.label = Some(label_from_u64(label));
        }
        @Varint(5, Type) => {
          assert!(self.Type.is_none());
          self.Type = Some(type_from_u64(Type));
        }
        @Raw(6, type_name) => {
          assert!(self.type_name.is_none());
          self.type_name = Some(from_utf8(type_name));
        }
        @Raw(7, default_value) => {
          assert!(self.default_value.is_none());
          self.default_value = Some(from_utf8(default_value));
        }
        _ => {
          //println(format!("Unknown tag: {:?}", tag_option));
        }
      }
    }
    return true;
  }
}

impl FieldDescriptorProto {
  fn translate(&self, package: ~str) -> ~str {
    let pkg = package;
    format!("  {:s}: {:s},", *self.name.get_ref(), self.translate_label(pkg))
  }

  fn translate_type_path(&self, package: ~str) -> ~str {
    let ref pkg = package;
    let ty_name = self.type_name.get_ref().clone();
    ty_name.trim_left_chars(~'.').slice_from(pkg.char_len() + 1).replace(".", "_")
  }

  fn translate_type(&self, package: ~str) -> ~str {
    let pkg = package;
    match self.Type.unwrap() {
      TYPE_DOUBLE => ~"f64",
      TYPE_FLOAT => ~"f32",
      TYPE_INT32 => ~"i32",
      TYPE_INT64 => ~"i64",
      TYPE_UINT32 => ~"u32",
      TYPE_UINT64 => ~"u64",
      TYPE_SINT32 => ~"i32",
      TYPE_SINT64 => ~"i64",
      TYPE_FIXED32 => ~"u32",
      TYPE_FIXED64 => ~"u64",
      TYPE_SFIXED32 => ~"i32",
      TYPE_SFIXED64 => ~"i64",
      TYPE_BOOL => ~"bool",
      TYPE_STRING => ~"~str",
      TYPE_BYTES => ~"~[u8]",
      TYPE_MESSAGE => self.translate_type_path(pkg),
      TYPE_ENUM => ~"ENUM",
      TYPE_GROUP => {fail!()}
    }
  }

  fn translate_label(&self, package: ~str) -> ~str {
    let pkg = package;
    let ty = self.translate_type(pkg);

    match self.label.unwrap() {
      LABEL_REPEATED => {format!("~[{:s}]", ty)}
      LABEL_OPTIONAL => {format!("Option<{:s}>", ty)}
      LABEL_REQUIRED => {ty}
    }
  }
}

impl DescriptorProto {
  fn translate(&self, package: ~str) -> ~str {
    let pkg = package;
    let name = self.name.get_ref().clone();
    let fields = self.field.map(|field|{field.translate(pkg.clone())}).connect("\n");
    format!("struct {:s} \\{\n{:s}\n\\}\n", name, fields)
  }
}

impl FileDescriptorProto {
  fn rs_package_name(&self) -> ~str {
    (*self.package.get_ref()).replace(".", "_")
  }

  fn translate(&self) -> ~str {
    let mut buf = ~"";
    for message_type in self.message_type.iter() {
      buf.push_str(format!("{:s}\n", message_type.translate(self.package.get_ref().clone())));
    }
    buf
  }
}

fn main() {
  let stdin_reader = stdin();
  let mut request = CodeGeneratorRequest{
    file_to_generate: ~[],
    parameter: None,
    proto_file: ~[],
  };
  assert!(request.Decode(stdin_reader));
  for proto_file in request.proto_file.iter() {
    println(proto_file.translate());
  }
}