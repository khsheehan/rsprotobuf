extern crate protobuf;
extern crate extra;
extern crate collections;

use std::to_str::ToStr;
use std::io::{stdin, Reader, MemReader};
use std::str::from_utf8;
use protobuf::{Protobuf, TagIter, Raw, Varint};
use collections::hashmap::{HashSet, HashMap};
use std::iter::FromIterator;

#[deriving(Show)]
struct CodeGeneratorRequest {
  file_to_generate: ~[~str],
  parameter: Option<~str>,
  proto_file: ~[FileDescriptorProto]
}

#[deriving(Show)]
struct FileDescriptorProto {
  name: Option<~str>,
  package: Option<~str>,
  message_type: ~[DescriptorProto]
}

#[deriving(Show)]
struct DescriptorProto {
  name: Option<~str>, // 1
  field: ~[FieldDescriptorProto], // 2
  nested_type: ~[DescriptorProto], // 3
  enum_type: ~[EnumDescriptorProto] // 4
}

static orig_var: &'static str = "encoded_var";

fn WireTypeForField(field: &FieldDescriptorProto) -> ~str {
  match *field.Type.get_ref() {
    Int32Type | Int64Type | UInt32Type | UInt64Type | SInt32Type | SInt64Type | BoolType => format!("@Varint({:s}, {:s})", *field.name.get_ref(), orig_var),
    StringType | BytesType | MessageType => format!("Raw({:s}, {:s})", *field.name.get_ref(), orig_var),
    _ => ~"UNKNOWN"
  }
}

#[deriving(Show,Clone,DeepClone,Eq)]
enum FieldDescriptorProto_Type {
    DoubleType         = 1,
    FloatType          = 2,
    // Not ZigZag encoded.  Negative numbers take 10 bytes.  Use SInt64Type if
    // negative values are likely.
    Int64Type          = 3,
    UInt64Type         = 4,
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
    UInt32Type         = 13,
    EnumType           = 14,
    SFixed32Type       = 15,
    SFixed64Type       = 16,
    SInt32Type         = 17,  // Uses ZigZag encoding.
    SInt64Type         = 18, 
}


impl FieldDescriptorProto_Type {
  fn to_proto_str(&self) -> ~str {
    match self {
      &Int32Type => ~"int32",
      &Int64Type => ~"int64",
      &UInt32Type => ~"uint32",
      &UInt64Type => ~"uint64",
      &SInt32Type => ~"sint32",
      &SInt64Type => ~"sint64",
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
    4 => UInt64Type,
    5 => Int32Type,
    6 => Fixed64Type,
    7 => Fixed32Type,
    8 => BoolType,
    9 => StringType,
    10 => GroupType,
    11 => MessageType,
    12 => BytesType,
    13 => UInt32Type,
    14 => EnumType,
    15 => SFixed32Type,
    16 => SFixed64Type,
    17 => SInt32Type,
    18 => SInt64Type,
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
        _ => fail!()
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
      buf.push_str(format!("pub mod {:s};\n", *self.package.get_ref()));
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
            nested_type: ~[],
            enum_type: ~[]
          };
          assert!(desc_proto.Decode(&mut reader));
          self.message_type.push(desc_proto)
        }
        _ => ()
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
            nested_type: ~[],
            enum_type: ~[]
          };
          assert!(desc_proto.Decode(&mut reader));
          self.nested_type.push(desc_proto)
        }
        Raw(4, enum_type) => {
          let mut reader = MemReader::new(enum_type);
          let mut enum_proto = EnumDescriptorProto{
            name: None,
            value: ~[]
          };
          assert!(enum_proto.Decode(&mut reader));
          self.enum_type.push(enum_proto);
        }
        _ => ()
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
        _ => ()
      }
    }
    return true;
  }
}

struct ProtobufGenerator<'a> {
  request: &'a CodeGeneratorRequest,
  current_package: Option<~str>,
  indent: uint,
  indent_str: ~str,
  buf: std::io::MemWriter
}

impl<'a> ProtobufGenerator<'a> {

  fn translate_package_namespace<'a>(&'a mut self, absolute: &str, current: &str) -> ~str {
    let id = if (absolute.starts_with("." + self.current_package.get_ref().as_slice())) {
      absolute.slice_from(current.len() + 1).to_owned()
    } else {
      absolute.to_owned()
    };
    self.translate_type_name(id)
  }

  fn new<'a>(request: &'a CodeGeneratorRequest) -> ProtobufGenerator<'a> {
    ProtobufGenerator {
      request: request,
      current_package: None,
      indent: 0,
      indent_str: ~"  ",
      buf: std::io::MemWriter::new()
    }
  }

  fn translate_type_name(&mut self, type_name: ~str) -> ~str {
    type_name.trim_left_chars(&'.').replace(".", "::")
  }

  fn translate_identifier(&mut self, identifier: ~str) -> ~str {
    match identifier {
      ~"type" => ~"type__",
      id => id
    }
  }

  fn translate_empty_message(&mut self, identifier: Option<&str>, descriptor: &DescriptorProto) -> std::fmt::Result {
    let prefix = identifier.map(|id| format!("let {:s} = ", id)).unwrap_or(~"");
    self.append_line(format!("{:s}{:s}\\{", prefix, descriptor.name.get_ref().as_slice()));
    self.indent += 1;
    for field in descriptor.field.iter() {
      let name = field.name.get_ref().as_slice();
      let label = field.label.unwrap();
      let field_type = field.Type.unwrap();
      match field_type {
        MessageType => (),/*{
          let child_descriptor = self.lookup_descriptor(name);
          translate_empty_message(name, child_descriptor)
        },*/
        _ => {
          let bare_val = match field_type {
            StringType => "~\"\"",
            BytesType => "~[]",
            BoolType => "false",
            FloatType | DoubleType => "0.0",
            _ => "0"
          };
          let full_val = match label {
            RequiredLabel => bare_val,
            OptionalLabel => "None",
            RepeatedLabel => "~[]",
          };
          self.append_line(format!("{:s}: {:s},", bare_val, full_val));
        }
      }
    }
    self.indent -= 1;
    self.append_line("}")
  }

  fn translate_field_impl(&mut self, field: &FieldDescriptorProto) -> std::fmt::Result {
    let field_name = field.name.get_ref().as_slice();
    let tag = field.number.unwrap();
    let label = field.label.unwrap();
    let field_type = field.Type.unwrap();

    match field_type {
      StringType | BytesType | MessageType => {
        self.append_line(format!("Raw({:d}, {:s}) => \\{", tag, field_name));
      }
      Fixed32Type | SFixed32Type | FloatType => {
        self.append_line(format!("Fixed32({:d}, {:s}) => \\{", tag, field_name));
      }
      Fixed64Type | SFixed64Type | DoubleType => {
        self.append_line(format!("Fixed64({:d}, {:s}) => \\{", tag, field_name));
      }
      Int32Type | Int64Type | SInt32Type | SInt64Type | UInt32Type | UInt64Type | BoolType | EnumType => {
        self.append_line(format!("Varint({:d}, {:s}) => \\{", tag, field_name));
      }
      GroupType => fail!(),
    }
    self.indent += 1;

    let bare_val = match field_type {
      MessageType => {
        let type_name = field.type_name.get_ref().as_slice();
        self.append_line(format!("let reader = std::io::MemReader::new({:s});", field_name));
        //translate_empty_message(Some(field_name), self.lookup_descriptor(type_name));
        self.append_line(format!("assert!({:s}.Decode(&mut reader));", field_name));
        field_name
      }
      StringType => {
        self.append_line(format!("let {:s} = std::str::from_utf8({:s}).unwrap();", field_name, field_name));
        field_name
      }
      _ => field_name
    };

    match label {
      RequiredLabel | OptionalLabel => {
        let val = match label {
          RequiredLabel => bare_val.to_owned(),
          OptionalLabel => format!("Some({:s})", bare_val),
          _ => fail!()
        };
        format!("self.{:s} = {:s};", field_name, val);
      }
      RepeatedLabel => {
        format!("self.{:s}.push({:s});", field_name, bare_val);
      }
    }
    self.indent -= 1;
    self.append_line("}")
  }

  fn translate_field(&mut self, field: &FieldDescriptorProto) -> std::fmt::Result {
    let bare_type = match field.Type.unwrap() {
      DoubleType => ~"f64",
      FloatType => ~"f32",
      Int32Type => ~"i32",
      Int64Type => ~"i64",
      UInt32Type => ~"u32",
      UInt64Type => ~"u64",
      SInt32Type => ~"i32",
      SInt64Type => ~"i64",
      Fixed32Type => ~"u32",
      Fixed64Type => ~"u64",
      SFixed32Type => ~"i32",
      SFixed64Type => ~"i64",
      BoolType => ~"bool",
      StringType => ~"~str",
      BytesType => ~"~[u8]",
      MessageType | EnumType => {
        let current_namespace = self.current_package.get_ref().to_owned();
        self.translate_package_namespace(field.type_name.get_ref().as_slice(), current_namespace)
      },
      _ => format!("UNKNOWN({})", field.Type.unwrap().to_str())
    };

    let full_type = match field.label.unwrap() {
      RepeatedLabel => format!("~[{:s}]", bare_type),
      OptionalLabel => format!("Option<{:s}>", bare_type),
      RequiredLabel => bare_type
    };

    let id = self.translate_identifier(field.name.get_ref().to_owned());
    self.append_line(format!("{}: {},", id, full_type))
  }

  fn translate_enum(&mut self, descriptor: &EnumDescriptorProto) -> std::fmt::Result {
    self.append_line(format!("enum {:s} \\{", descriptor.name.get_ref().to_owned()));
    self.indent += 1;
    for value in descriptor.value.iter() {
      let name = value.name.get_ref().to_owned();
      let number = value.number.unwrap();
      self.append_line(format!("{:s} = {:d},", name, number));
    }
    self.indent -= 1;
    self.append_line("}")
  }

  fn translate_descriptor_impl(&mut self, descriptor: &DescriptorProto) -> std::fmt::Result {
    self.append_line(format!("impl Protobuf for {:s} \\{", descriptor.name.get_ref().to_owned()));
    self.indent += 1;
    /*
  fn Decode<'a>(&mut self, reader: &'a mut Reader) -> bool {
    for tag_option in TagIter{reader: reader} {
      match tag_option {
        Raw(1, name) => {
          assert!(self.name.is_none());
          self.name = from_utf8(name).map(|n| n.to_owned());
        }
    */
    self.append_line("fn Decode<'a>(&mut self, reader: &'a mut Reader) -> bool {");
    self.indent += 1;
    self.append_line("for tag_option in TagIter{reader: reader} {");
    self.indent += 1;
    self.append_line("match tag_option {");
    self.indent += 1;
    for field in descriptor.field.iter() {
      self.translate_field_impl(field);
    }
    self.indent -= 1;
    self.append_line("}");
    self.indent -= 1;
    self.append_line("}");
    self.indent -= 1;
    self.append_line("}");
    self.indent -= 1;
    self.append_line("}")
  }

  fn translate_descriptor(&mut self, descriptor: &DescriptorProto) -> std::fmt::Result {
    self.append_line(format!("struct {:s} \\{", descriptor.name.get_ref().to_owned()));
    self.indent += 1;
    for field in descriptor.field.iter() {
      self.translate_field(field);
    }
    self.indent -= 1;
    self.append_line("}");
    if descriptor.nested_type.len() > 0 || descriptor.enum_type.len() > 0 {
      self.append_line(format!("pub mod {:s} \\{", descriptor.name.get_ref().to_owned()));
      self.indent += 1;
      for ty in descriptor.nested_type.iter() {
        self.translate_descriptor(ty);
      }
      for ty in descriptor.enum_type.iter() {
        self.translate_enum(ty);
      }
      self.indent -= 1;
      self.append_line("}");
    }

    self.translate_descriptor_impl(descriptor)
  }

  fn pad(&self, line: &str) -> ~str {
    let mut buf = ~"";
    for i in std::iter::range(0, self.indent) {
      buf.push_str(self.indent_str);
    }
    buf.push_str(line);
    buf
  }

  fn append_line(&mut self, line: &str) -> std::fmt::Result {
    assert!(!line.ends_with("\n"));
    let complete_line = self.pad(line) + "\n";
    self.buf.write(complete_line.as_bytes())
  }

  fn translate_file(&mut self, proto: &FileDescriptorProto) {
    let mut buf = ~"";
    self.current_package = proto.package.clone();
    self.append_line("extern crate protobuf;");
    self.append_line("");

    let package_path_components = proto.package.get_ref().split('.').map(|p| p.to_owned()).to_owned_vec();
    self.translate_package(proto, package_path_components);
  }

  fn translate_package(&mut self, proto: &FileDescriptorProto, package_path_components: &[~str]) {
    self.append_line(format!("pub mod {:s} \\{", package_path_components[0]));
    self.indent += 1;
    if (package_path_components.len() > 1) {
      self.translate_package(proto, package_path_components.slice_from(1));
    } else {
      for message_type in proto.message_type.iter() {
        self.translate_descriptor(message_type);
      }
    }
    self.indent -= 1;
    self.append_line("}");
  }

  fn translate(&mut self) {
    let files_to_generate: HashSet<&~str> = FromIterator::from_iterator(&mut self.request.file_to_generate.iter());
    for proto_file in self.request.proto_file.iter() {
      if files_to_generate.contains(&proto_file.name.get_ref()) {
        self.translate_file(proto_file);
      }
    }
    let bytes = self.buf.get_ref();
    println!("{}", from_utf8(bytes).unwrap());
  }
}

impl DescriptorProto {
  fn rs_name(&self, prefix: &str) -> ~str {
    format!("{:s}_{:s}", prefix, *self.name.get_ref())
  }
}

impl FileDescriptorProto {
  fn rs_package_name(&self) -> ~str {
    (*self.package.get_ref()).replace(".", "_")
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
  let mut gen = ProtobufGenerator::new(&request);
  gen.translate();
}

#[deriving(Show)]
struct EnumDescriptorProto {
  name: Option<~str>,
  value: ~[EnumValueDescriptorProto]
}

impl Protobuf for EnumDescriptorProto {
  fn Decode<'a>(&mut self, reader: &'a mut Reader) -> bool {
    for tag_option in TagIter{reader: reader} {
      match tag_option {
        Raw(1, name) => {
          assert!(self.name.is_none());
          self.name = from_utf8(name).map(|n| n.to_owned());
        }
        Raw(2, value) => {
          let mut reader = MemReader::new(value);
          let mut enum_value_descriptor_proto = EnumValueDescriptorProto{
            name: None,
            number: None
          };
          assert!(enum_value_descriptor_proto.Decode(&mut reader));
          self.value.push(enum_value_descriptor_proto);
        }
        _ => ()
      }
    }
    true
  }
}

#[deriving(Show)]
struct EnumValueDescriptorProto {
  name: Option<~str>,
  number: Option<i32>
}

impl Protobuf for EnumValueDescriptorProto {
  fn Decode<'a>(&mut self, reader: &'a mut Reader) -> bool {
    for tag_option in TagIter{reader: reader} {
      match tag_option {
        Raw(1, name) => {
          assert!(self.name.is_none());
          self.name = from_utf8(name).map(|n| n.to_owned());
        }
        Varint(2, number) => {
          assert!(self.number.is_none());
          self.number = Some(number as i32);
        }
        _ => ()
      }
    }
    true
  }
}