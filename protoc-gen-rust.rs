extern mod protobuf;

use std::io::{stdin, Reader, with_bytes_reader};
use std::str::from_utf8;
use protobuf::{Protobuf, TagIter, Raw};

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
  nested_type: ~[DescriptorProto], // 3
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
        @Raw(3, nested_type) => {
          let reader = do with_bytes_reader(nested_type) |reader| { reader };
          let mut desc_proto = DescriptorProto{
            name: None,
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

fn main() {
  let stdin_reader = stdin();
  let mut request = CodeGeneratorRequest{
    file_to_generate: ~[],
    parameter: None,
    proto_file: ~[],
  };
  assert!(request.Decode(stdin_reader));
  println(request.to_str());
}