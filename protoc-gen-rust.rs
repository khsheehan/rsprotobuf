extern crate protobuf;
extern crate extra;
extern crate collections;

use std::to_str::ToStr;
use std::io::{stdin, Reader, MemReader};
use std::str::from_utf8;
use protobuf::{Protobuf, TagIter, Raw, Varint};
use collections::hashmap::HashSet;
use std::iter::FromIterator;

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

static orig_var: &'static str = "encoded_var";

fn WireTypeForField(field: &FieldDescriptorProto) -> ~str {
  match *field.Type.get_ref() {
    Int32Type | Int64Type | Uint32Type | TYPE_UINT64 | SInt32Type | SIntType64 | BoolType => format!("@Varint({:s}, {:s})", *field.name.get_ref(), orig_var),
    StringType | BytesType | MessageType => format!("Raw({:s}, {:s})", *field.name.get_ref(), orig_var),
    _ => ~"UNKNOWN"
  }
}

fn DerivingString(field: &FieldDescriptorProto) -> ~str {
  let cast_var = "decoded_var";
  let struct_var = "struct_var";
  let field_name: ~str = (*field.name.get_ref()).clone();
  match *field.Type.get_ref() {
    Int32Type | Int64Type | Uint32Type | TYPE_UINT64 | SInt32Type | SIntType64 | BoolType | BytesType => {
      format!(
        "{:s} => \\{
          let {:s} = {:s} as {:s};
          {:s}.{:s} = {:s};
        \\}",
        WireTypeForField(field),
        cast_var.clone(),
        orig_var,
        field.Type.get_ref().translate(),
        struct_var,
        field_name,
        cast_var.clone(),
      )
    }
    StringType => {
      format!(
        "{:s} => \\{
          let {:s}: {:s} = from_utf8({:s});
          {:s}.{:s} = {:s};
        \\}",
        WireTypeForField(field),
        cast_var.clone(),
        field.Type.get_ref().translate(),
        orig_var,
        struct_var,
        field_name,
        cast_var.clone(),
      )
    }
    MessageType => {
      format!("{:s} => \\{
          let mut reader = MemReader::new({:s})
          let mut {:s} = {:s}();
          assert!({:s}.Decode(reader));
          {:s}.{:s}.push({:s});
        \\}",
        WireTypeForField(field),
        orig_var,
        cast_var,
        (*field.type_name.get_ref()),
        cast_var,
        struct_var,
        field_name,
        cast_var)
    }
    _ => ~"UNKNOWN"
  }
}

#[deriving(Show,Clone,DeepClone,Eq)]
enum FieldDescriptorProto_Type {
    DoubleType         = 1,
    FloatType          = 2,
    // Not ZigZag encoded.  Negative numbers take 10 bytes.  Use SIntType64 if
    // negative values are likely.
    Int64Type          = 3,
    TYPE_UINT64         = 4,
    // Not ZigZag encoded.  Negative numbers take 10 bytes.  Use SInt32Type if
    // negative values are likely.
    Int32Type          = 5,
    Fixed64Type        = 6,
    Fixed32Type        = 7,
    BoolType           = 8,
    StringType         = 9,
    GroupType          = 10,  // Tag-delimited aggregate.
    MessageType        = 11,  // Length-delimited aggregate.

    // New in version 2.
    BytesType          = 12,
    Uint32Type         = 13,
    EnumType           = 14,
    SFixed32Type       = 15,
    SFixed64Type       = 16,
    SInt32Type         = 17,  // Uses ZigZag encoding.
    SIntType64         = 18, 
}


impl FieldDescriptorProto_Type {
  fn translate(&self) -> ~str {
    match self {
      &Int32Type => ~"i32",
      &Int64Type => ~"i64",
      &Uint32Type => ~"u32",
      &TYPE_UINT64 => ~"u64",
      &SInt32Type => ~"i32",
      &SIntType64 => ~"i64",
      &BoolType => ~"bool",
      &StringType => ~"~str",
      &BytesType => ~"~[u8]",
      _ => fail!(self.to_str())
    }
  }

  fn to_proto_str(&self) -> ~str {
    match self {
      &Int32Type => ~"int32",
      &Int64Type => ~"int64",
      &Uint32Type => ~"uint32",
      &TYPE_UINT64 => ~"uint64",
      &SInt32Type => ~"sint32",
      &SIntType64 => ~"sint64",
      &BoolType => ~"bool",
      &StringType => ~"string",
      &BytesType => ~"bytes",
      &MessageType => ~"message",
      &DoubleType => ~"double",
      _ => fail!(self.to_str())
    }
  }
}

fn type_from_u64(u: u64) -> FieldDescriptorProto_Type {
  match u {
    1 => DoubleType,
    2 => FloatType,
    3 => Int64Type,
    4 => TYPE_UINT64,
    5 => Int32Type,
    6 => Fixed64Type,
    7 => Fixed32Type,
    8 => BoolType,
    9 => StringType,
    10 => GroupType,
    11 => MessageType,
    12 => BytesType,
    13 => Uint32Type,
    14 => EnumType,
    15 => SFixed32Type,
    16 => SFixed64Type,
    17 => SInt32Type,
    18 => SIntType64,
    _ => {fail!()},
  }
}

#[deriving(Show)]
enum FieldDescriptorProto_Label {
  OptionalLabel      = 1,
  RequiredLabel      = 2,
  RepeatedLabel      = 3,
}

impl FieldDescriptorProto_Label {
  fn to_proto_str(&self) -> ~str {
    match self {
      &OptionalLabel => ~"optional",
      &RequiredLabel => ~"required",
      &RepeatedLabel => ~"repeated"
    }
  }
}

fn label_from_u64(u: u64) -> FieldDescriptorProto_Label {
  match u {
    1 => OptionalLabel,
    2 => RequiredLabel,
    3 => RepeatedLabel,
    _ => fail!()
  }
}

#[deriving(Show)]
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
    let ty_proto_str = match self.Type.unwrap() {
      EnumType => {
        (*self.type_name.get_ref()).clone()
      }
      id => id.to_proto_str()
    };
    let default = if self.default_value.is_some() { format!(" [default = \"{:s}\"]", *self.default_value.get_ref()) } else { ~"" };
    return format!("{:s}{:s} {:s} {:s} = {:d}{:s};",
                   padding,
                   (*self.label.get_ref()).to_proto_str(),
                   ty_proto_str,
                   (*self.name.get_ref()),
                   *self.number.get_ref(),
                   default);
  }

  fn fmt(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
      try!(write!(formatter.buf, "{}", self.BuildTreeLines(0)));
      Ok(())
  }
}

impl Protobuf for CodeGeneratorRequest {
  fn Decode<'a>(&mut self, reader: &'a mut Reader) -> bool {
    for tag_option in TagIter{reader: reader} {
      match tag_option {
        Raw(1, data) => {
          self.file_to_generate.push(from_utf8(data).unwrap().to_owned());
        }
        Raw(2, parameter) => {
          assert!(self.parameter.is_none());
          self.parameter = Some(from_utf8(parameter).unwrap().to_owned());
        }
        Raw(15, proto_file) => {
          let mut reader = MemReader::new(proto_file);
          let mut fd_proto = FileDescriptorProto{
            name: None,
            package: None,
            message_type: ~[]
          };
          assert!(fd_proto.Decode(&mut reader));
          self.proto_file.push(fd_proto);
        }
        _ => {
          //println(format!("Unknown tag: {:?}", tag_option));
        }
      }
    }
    true
  }
}

impl CodeGeneratorRequest {
  fn to_proto_str(&self) -> ~str {
    let mut buf = ~"";
    if self.file_to_generate.len() > 0 {
      buf.push_str(format!("Files to generate:\n{:s}", self.file_to_generate.connect("\n\t")));
    }
    if self.proto_file.len() > 0 {
      buf.push_str(format!("\n\nFile descriptor protos:\n{:s}", self.proto_file.map(|proto_file| {proto_file.to_proto_str()}).connect("\n\n")));
    }
    buf
  }
}

impl FileDescriptorProto {
  fn to_proto_str(&self) -> ~str {
    let mut buf = format!("File \"{:s}\":\n\n", *self.name.get_ref());

    if !self.package.is_none() {
      buf.push_str(format!("package {:s};\n", *self.package.get_ref()));
    }

    if self.message_type.len() > 0 {
      buf.push_str(format!("\n\n{:s}\n", self.message_type.map(|message_type|{message_type.to_proto_str()}).connect("\n\n")));
    }
    buf
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

impl DescriptorProto {
  fn to_proto_str(&self) -> ~str {
    self.BuildTreeLines(0)
  }
}

impl Protobuf for FileDescriptorProto {
  fn Decode<'a>(&mut self, reader: &'a mut Reader) -> bool {
    for tag_option in TagIter{reader: reader} {
      match tag_option {
        Raw(1, name) => {
          assert!(self.name.is_none());
          self.name = Some(from_utf8(name).unwrap().to_owned())
        }
        Raw(2, package) => {
          assert!(self.package.is_none());
          self.package = Some(from_utf8(package).unwrap().to_owned());
        }
        Raw(4, message_type) => {
          let mut reader = MemReader::new(message_type);
          let mut desc_proto = DescriptorProto{
            name: None,
            field: ~[],
            nested_type: ~[]
          };
          assert!(desc_proto.Decode(&mut reader));
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
  fn Decode<'a>(&mut self, reader: &'a mut Reader) -> bool {
    for tag_option in TagIter{reader: reader} {
      match tag_option {
        Raw(1, name) => {
          assert!(self.name.is_none());
          self.name = Some(from_utf8(name).unwrap().to_owned())
        }
        Raw(2, field) => {
          let mut reader = MemReader::new(field);
          let mut field_proto = FieldDescriptorProto{
            name: None,
            number: None,
            label: None,
            Type: None,
            type_name: None,
            default_value: None,
          };
          assert!(field_proto.Decode(&mut reader));
          self.field.push(field_proto)
        }
        Raw(3, nested_type) => {
          let mut reader = MemReader::new(nested_type);
          let mut desc_proto = DescriptorProto{
            name: None,
            field: ~[],
            nested_type: ~[]
          };
          assert!(desc_proto.Decode(&mut reader));
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
  fn Decode<'a>(&mut self, reader: &'a mut Reader) -> bool {
    for tag_option in TagIter{reader: reader} {
      match tag_option {
        Raw(1, name) => {
          assert!(self.name.is_none());
          self.name = Some(from_utf8(name).unwrap().to_owned())
        }
        Varint(3, number) => {
          assert!(self.number.is_none());
          self.number = Some(number as i32);
        }
        Varint(4, label) => {
          assert!(self.label.is_none());
          self.label = Some(label_from_u64(label));
        }
        Varint(5, Type) => {
          assert!(self.Type.is_none());
          self.Type = Some(type_from_u64(Type));
        }
        Raw(6, type_name) => {
          assert!(self.type_name.is_none());
          self.type_name = Some(from_utf8(type_name).unwrap().to_owned());
        }
        Raw(7, default_value) => {
          assert!(self.default_value.is_none());
          self.default_value = Some(from_utf8(default_value).unwrap().to_owned());
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
  fn translate(&self, package: &str) -> ~str {
    let pkg = package;
    if self.Type.unwrap() == EnumType {
      return ~"";
    }
    format!("  {:s}: {:s},", *self.name.get_ref(), self.translate_label(pkg))
  }

  fn translate_type_path(&self, package: &str) -> ~str {
    let ref pkg = package;
    let ty_name = self.type_name.get_ref().clone();
    ty_name.trim_left_chars(~'.').slice_from(pkg.char_len() + 1).replace(".", "_")
  }

  fn translate_type(&self, package: &str) -> ~str {
    let pkg = package.clone();
    match self.Type.unwrap() {
      DoubleType => ~"f64",
      FloatType => ~"f32",
      Int32Type => ~"i32",
      Int64Type => ~"i64",
      Uint32Type => ~"u32",
      TYPE_UINT64 => ~"u64",
      SInt32Type => ~"i32",
      SIntType64 => ~"i64",
      Fixed32Type => ~"u32",
      Fixed64Type => ~"u64",
      SFixed32Type => ~"i32",
      SFixed64Type => ~"i64",
      BoolType => ~"bool",
      StringType => ~"~str",
      BytesType => ~"~[u8]",
      MessageType => self.translate_type_path(pkg.clone()),
      _ => {fail!()}
    }
  }

  fn translate_label(&self, package: &str) -> ~str {
    let pkg = package;
    let ty = self.translate_type(pkg);

    match self.label.unwrap() {
      RepeatedLabel => {format!("~[{:s}]", ty)}
      OptionalLabel => {format!("Option<{:s}>", ty)}
      RequiredLabel => {ty}
    }
  }
}

struct ProtobufGenerator {
  request: ~CodeGeneratorRequest,
  current_package: Option<~str>,
}

impl ProtobufGenerator {
  fn new(request: ~CodeGeneratorRequest) -> ProtobufGenerator {
    ProtobufGenerator {
      request: request,
      current_package: None,
    }
  }

  fn translate_descriptor(&mut self, descriptor: &DescriptorProto, name: ~str) -> ~str {
    let pkg = self.current_package.get_ref().to_owned();
    let fields = descriptor.field.map(|field|{field.translate(pkg)}).connect("\n");
    let others = descriptor.nested_type.map(|nested_type| {
      let child_name = format!("{:s}_{:s}", name, *nested_type.name.get_ref());
      self.translate_descriptor(nested_type, child_name)
    }).connect("\n\n");
    let arms = descriptor.field.map(|field| {
      DerivingString(field)
    }).connect("\n");
    let implementation = format!("impl Protobuf for {:s} \\{
  fn Decode<'a>(&mut self, reader: &'a mut Reader) -> bool \\{
    for tag_option in TagIter\\{reader: reader\\} \\{
      match tag_option \\{
{:s}
        _ => \\{
          //println(format!(\"Unknown tag: \\{:?\\}\", tag_option));
        \\}
      \\}
    \\}
    return true;
  \\}
\\}",
name,
arms);
    format!("struct {:s} \\{\n{:s}\n\\}\n{:s}\n\n{:s}", name, fields, implementation, others)
  }

  fn translate_file(&mut self, proto: &FileDescriptorProto) -> ~str {
    let mut buf = ~"";
    self.current_package = Some((*proto.package.get_ref()).clone());
    for message_type in proto.message_type.iter() {
      let name = (*message_type.name.get_ref()).clone();
      buf.push_str(format!("{:s}\n", self.translate_descriptor(message_type, name)));
    }
    buf
  }

  fn translate(&mut self) {
    let files_to_generate: HashSet<&~str> = FromIterator::from_iterator(&mut self.request.file_to_generate.iter());
    for proto_file in self.request.proto_file.iter() {
      if files_to_generate.contains(&proto_file.name.get_ref()) {
        print!("{}", proto_file.name.get_ref());
      }
    }
  }
}

impl DescriptorProto {
  fn translate<'a>(&self, package: &'a str) -> ~str {
    let pkg = package;
    let name = self.name.get_ref().clone();
    let fields = self.field.map(|field|{field.translate(pkg.clone())}).connect("\n");
    let others = self.nested_type.map(|nested_type|{nested_type.translate(pkg.clone())}).connect("\n\n");
    format!("struct {:s} \\{\n{:s}\n\\}\n\n{:s}", name, fields, others)
  }

  fn rs_name(&self, prefix: &str) -> ~str {
    format!("{:s}_{:s}", prefix, *self.name.get_ref())
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
  let mut stdin_reader = stdin();
  let mut request = CodeGeneratorRequest{
    file_to_generate: ~[],
    parameter: None,
    proto_file: ~[],
  };
  assert!(request.Decode(&mut stdin_reader));
  for proto_file in request.proto_file.iter() {
    println!("{}", proto_file.to_proto_str());
  }
  let mut gen = ProtobufGenerator::new(~request);
}