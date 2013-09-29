#[ link(name = "protobuf",
        vers = "0.0.1",
        uuid = "706F5A8D-ADCC-4237-92C6-6C1742C73141") ];

#[ desc = "protobuf support for rust" ];
#[ license = "ASL2" ];
#[ author = "Duane R Bailey" ];

#[ crate_type = "lib" ];
#[ link_args = "-lprotobuf-c" ]

pub mod protobuf;
