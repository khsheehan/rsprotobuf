extern mod protobuf;

use std::io::stdin;
use std::str::from_utf8;
use std::io::Reader;
use protobuf::{Protobuf, TagIter, Raw};

struct CodeGeneratorRequest {
  file_to_generate: ~[~str],
  parameter: Option<~str>,
  proto_file: ~[u8]
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
        _ => {
          println(format!("Unknown tag: {:?}", tag_option));
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
}