#[link_args = "-lprotobuf-c -lproto"]

extern mod protobuf;

use std::io::stdin;
use std::ptr::null;
use std::vec::raw::{to_ptr, from_buf_raw};
use std::str::raw::from_c_str;
use std::libc::size_t;

#[fixed_stack_segment]
fn main() {
  let packed = stdin().read_whole_stream();
  unsafe {
    let packed_len = packed.len() as size_t;
    let packed_ptr = to_ptr(packed);
    let code_gen_ptr = protobuf::google__protobuf__compiler__code_generator_request__unpack(null(), packed_len, packed_ptr);
    let bytes_vec = from_buf_raw((*code_gen_ptr).files_to_generate, (*code_gen_ptr).n_files_to_generate as uint);
    
    let proto_file_vec = from_buf_raw((*code_gen_ptr).proto_file, (*code_gen_ptr).n_proto_file as uint);
    for proto_file in proto_file_vec.iter() {
      println(from_c_str((*(*proto_file)).name));
    }
  }
}