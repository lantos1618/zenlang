// Network module for Zen standard library
// Provides TCP/UDP networking capabilities

use crate::ast::{Declaration, Function, ExternalFunction, Statement, Expression, AstType};
use crate::ast::{StructDefinition, StructField, VariableDeclarationType};
use crate::error::Result;

/// Create the net module with TCP and UDP support
pub fn create_net_module() -> Vec<Declaration> {
    let mut declarations = Vec::new();
    
    // External C functions for networking
    declarations.extend(create_socket_externals());
    
    // Socket type definitions
    declarations.extend(create_socket_types());
    
    // TCP functions
    declarations.extend(create_tcp_functions());
    
    // UDP functions
    declarations.extend(create_udp_functions());
    
    // Helper functions
    declarations.extend(create_helper_functions());
    
    declarations
}

fn create_socket_externals() -> Vec<Declaration> {
    vec![
        // Socket creation and management
        Declaration::ExternalFunction(ExternalFunction {
            name: "socket".to_string(),
            args: vec![AstType::I32, AstType::I32, AstType::I32], // domain, type, protocol
            return_type: AstType::I32, // file descriptor
            is_varargs: false,
        }),
        
        Declaration::ExternalFunction(ExternalFunction {
            name: "bind".to_string(),
            args: vec![
                AstType::I32, // socket fd
                AstType::Pointer(Box::new(AstType::U8)), // sockaddr
                AstType::U32, // addrlen
            ],
            return_type: AstType::I32,
            is_varargs: false,
        }),
        
        Declaration::ExternalFunction(ExternalFunction {
            name: "listen".to_string(),
            args: vec![AstType::I32, AstType::I32], // socket fd, backlog
            return_type: AstType::I32,
            is_varargs: false,
        }),
        
        Declaration::ExternalFunction(ExternalFunction {
            name: "accept".to_string(),
            args: vec![
                AstType::I32, // socket fd
                AstType::Pointer(Box::new(AstType::U8)), // sockaddr
                AstType::Pointer(Box::new(AstType::U32)), // addrlen
            ],
            return_type: AstType::I32, // new socket fd
            is_varargs: false,
        }),
        
        Declaration::ExternalFunction(ExternalFunction {
            name: "connect".to_string(),
            args: vec![
                AstType::I32, // socket fd
                AstType::Pointer(Box::new(AstType::U8)), // sockaddr
                AstType::U32, // addrlen
            ],
            return_type: AstType::I32,
            is_varargs: false,
        }),
        
        Declaration::ExternalFunction(ExternalFunction {
            name: "send".to_string(),
            args: vec![
                AstType::I32, // socket fd
                AstType::Pointer(Box::new(AstType::U8)), // buffer
                AstType::U64, // length
                AstType::I32, // flags
            ],
            return_type: AstType::I64, // bytes sent
            is_varargs: false,
        }),
        
        Declaration::ExternalFunction(ExternalFunction {
            name: "recv".to_string(),
            args: vec![
                AstType::I32, // socket fd
                AstType::Pointer(Box::new(AstType::U8)), // buffer
                AstType::U64, // length
                AstType::I32, // flags
            ],
            return_type: AstType::I64, // bytes received
            is_varargs: false,
        }),
        
        Declaration::ExternalFunction(ExternalFunction {
            name: "close".to_string(),
            args: vec![AstType::I32], // file descriptor
            return_type: AstType::I32,
            is_varargs: false,
        }),
        
        // UDP specific
        Declaration::ExternalFunction(ExternalFunction {
            name: "sendto".to_string(),
            args: vec![
                AstType::I32, // socket fd
                AstType::Pointer(Box::new(AstType::U8)), // buffer
                AstType::U64, // length
                AstType::I32, // flags
                AstType::Pointer(Box::new(AstType::U8)), // dest_addr
                AstType::U32, // addrlen
            ],
            return_type: AstType::I64, // bytes sent
            is_varargs: false,
        }),
        
        Declaration::ExternalFunction(ExternalFunction {
            name: "recvfrom".to_string(),
            args: vec![
                AstType::I32, // socket fd
                AstType::Pointer(Box::new(AstType::U8)), // buffer
                AstType::U64, // length
                AstType::I32, // flags
                AstType::Pointer(Box::new(AstType::U8)), // src_addr
                AstType::Pointer(Box::new(AstType::U32)), // addrlen
            ],
            return_type: AstType::I64, // bytes received
            is_varargs: false,
        }),
        
        // Helper functions
        Declaration::ExternalFunction(ExternalFunction {
            name: "htons".to_string(),
            args: vec![AstType::U16], // host to network short
            return_type: AstType::U16,
            is_varargs: false,
        }),
        
        Declaration::ExternalFunction(ExternalFunction {
            name: "htonl".to_string(),
            args: vec![AstType::U32], // host to network long
            return_type: AstType::U32,
            is_varargs: false,
        }),
        
        Declaration::ExternalFunction(ExternalFunction {
            name: "inet_addr".to_string(),
            args: vec![AstType::Pointer(Box::new(AstType::I8))], // IP string
            return_type: AstType::U32, // network byte order address
            is_varargs: false,
        }),
    ]
}

fn create_socket_types() -> Vec<Declaration> {
    vec![
        // Socket address structure
        Declaration::Struct(StructDefinition {
            name: "SockAddr".to_string(),
            type_params: vec![],
            fields: vec![
                StructField {
                    name: "family".to_string(),
                    type_: AstType::U16,
                    is_mutable: false,
                    default_value: None,
                },
                StructField {
                    name: "port".to_string(),
                    type_: AstType::U16,
                    is_mutable: false,
                    default_value: None,
                },
                StructField {
                    name: "addr".to_string(),
                    type_: AstType::U32,
                    is_mutable: false,
                    default_value: None,
                },
                StructField {
                    name: "zero".to_string(),
                    type_: AstType::FixedArray {
                        element_type: Box::new(AstType::U8),
                        size: 8,
                    },
                    is_mutable: false,
                    default_value: None,
                },
            ],
            methods: vec![],
        }),
        
        // TCP Socket wrapper
        Declaration::Struct(StructDefinition {
            name: "TcpSocket".to_string(),
            type_params: vec![],
            fields: vec![
                StructField {
                    name: "fd".to_string(),
                    type_: AstType::I32,
                    is_mutable: false,
                    default_value: None,
                },
            ],
            methods: vec![],
        }),
        
        // UDP Socket wrapper
        Declaration::Struct(StructDefinition {
            name: "UdpSocket".to_string(),
            type_params: vec![],
            fields: vec![
                StructField {
                    name: "fd".to_string(),
                    type_: AstType::I32,
                    is_mutable: false,
                    default_value: None,
                },
            ],
            methods: vec![],
        }),
        
        // Connection result type
        Declaration::Struct(StructDefinition {
            name: "Connection".to_string(),
            type_params: vec![],
            fields: vec![
                StructField {
                    name: "socket".to_string(),
                    type_: AstType::Struct {
                        name: "TcpSocket".to_string(),
                        fields: vec![],
                    },
                    is_mutable: false,
                    default_value: None,
                },
                StructField {
                    name: "addr".to_string(),
                    type_: AstType::Struct {
                        name: "SockAddr".to_string(),
                        fields: vec![],
                    },
                    is_mutable: false,
                    default_value: None,
                },
            ],
            methods: vec![],
        }),
    ]
}

fn create_tcp_functions() -> Vec<Declaration> {
    vec![
        // Create TCP listener
        Declaration::Function(Function {
            type_params: vec![],
            name: "tcp_listen".to_string(),
            args: vec![
                ("port".to_string(), AstType::U16),
            ],
            return_type: AstType::Struct {
                name: "Result".to_string(),
                fields: vec![], // Would be Result<TcpSocket, Error>
            },
            body: vec![
                // Implementation would create socket, bind, and listen
                Statement::VariableDeclaration {
                    name: "sock_fd".to_string(),
                    type_: Some(AstType::I32),
                    initializer: Some(Expression::FunctionCall {
                        name: "socket".to_string(),
                        args: vec![
                            Expression::Integer32(2), // AF_INET
                            Expression::Integer32(1), // SOCK_STREAM
                            Expression::Integer32(0), // protocol
                        ],
                    }),
                    is_mutable: false,
                    declaration_type: VariableDeclarationType::ExplicitImmutable,
                },
                // More implementation...
                Statement::Return(Expression::StructLiteral {
                    name: "TcpSocket".to_string(),
                    fields: vec![
                        ("fd".to_string(), Expression::Identifier("sock_fd".to_string())),
                    ],
                }),
            ],
            is_async: false,
        }),
        
        // Accept connection
        Declaration::Function(Function {
            type_params: vec![],
            name: "tcp_accept".to_string(),
            args: vec![
                ("listener".to_string(), AstType::Pointer(Box::new(AstType::Struct {
                    name: "TcpSocket".to_string(),
                    fields: vec![],
                }))),
            ],
            return_type: AstType::Struct {
                name: "Result".to_string(),
                fields: vec![], // Result<Connection, Error>
            },
            body: vec![
                // Implementation would call accept
                Statement::Return(Expression::StructLiteral {
                    name: "Connection".to_string(),
                    fields: vec![],
                }),
            ],
            is_async: false,
        }),
        
        // Connect to TCP server
        Declaration::Function(Function {
            type_params: vec![],
            name: "tcp_connect".to_string(),
            args: vec![
                ("host".to_string(), AstType::String),
                ("port".to_string(), AstType::U16),
            ],
            return_type: AstType::Struct {
                name: "Result".to_string(),
                fields: vec![], // Result<TcpSocket, Error>
            },
            body: vec![
                // Implementation would create socket and connect
                Statement::Return(Expression::StructLiteral {
                    name: "TcpSocket".to_string(),
                    fields: vec![],
                }),
            ],
            is_async: false,
        }),
        
        // Send data over TCP
        Declaration::Function(Function {
            type_params: vec![],
            name: "tcp_send".to_string(),
            args: vec![
                ("socket".to_string(), AstType::Pointer(Box::new(AstType::Struct {
                    name: "TcpSocket".to_string(),
                    fields: vec![],
                }))),
                ("data".to_string(), AstType::Pointer(Box::new(AstType::U8))),
                ("len".to_string(), AstType::U64),
            ],
            return_type: AstType::I64, // bytes sent
            body: vec![
                Statement::Return(Expression::FunctionCall {
                    name: "send".to_string(),
                    args: vec![
                        Expression::StructField {
                            struct_: Box::new(Expression::Dereference(
                                Box::new(Expression::Identifier("socket".to_string()))
                            )),
                            field: "fd".to_string(),
                        },
                        Expression::Identifier("data".to_string()),
                        Expression::Identifier("len".to_string()),
                        Expression::Integer32(0), // flags
                    ],
                }),
            ],
            is_async: false,
        }),
        
        // Receive data over TCP
        Declaration::Function(Function {
            type_params: vec![],
            name: "tcp_recv".to_string(),
            args: vec![
                ("socket".to_string(), AstType::Pointer(Box::new(AstType::Struct {
                    name: "TcpSocket".to_string(),
                    fields: vec![],
                }))),
                ("buffer".to_string(), AstType::Pointer(Box::new(AstType::U8))),
                ("len".to_string(), AstType::U64),
            ],
            return_type: AstType::I64, // bytes received
            body: vec![
                Statement::Return(Expression::FunctionCall {
                    name: "recv".to_string(),
                    args: vec![
                        Expression::StructField {
                            struct_: Box::new(Expression::Dereference(
                                Box::new(Expression::Identifier("socket".to_string()))
                            )),
                            field: "fd".to_string(),
                        },
                        Expression::Identifier("buffer".to_string()),
                        Expression::Identifier("len".to_string()),
                        Expression::Integer32(0), // flags
                    ],
                }),
            ],
            is_async: false,
        }),
    ]
}

fn create_udp_functions() -> Vec<Declaration> {
    vec![
        // Create UDP socket
        Declaration::Function(Function {
            type_params: vec![],
            name: "udp_socket".to_string(),
            args: vec![],
            return_type: AstType::Struct {
                name: "Result".to_string(),
                fields: vec![], // Result<UdpSocket, Error>
            },
            body: vec![
                Statement::VariableDeclaration {
                    name: "sock_fd".to_string(),
                    type_: Some(AstType::I32),
                    initializer: Some(Expression::FunctionCall {
                        name: "socket".to_string(),
                        args: vec![
                            Expression::Integer32(2), // AF_INET
                            Expression::Integer32(2), // SOCK_DGRAM
                            Expression::Integer32(0), // protocol
                        ],
                    }),
                    is_mutable: false,
                    declaration_type: VariableDeclarationType::ExplicitImmutable,
                },
                Statement::Return(Expression::StructLiteral {
                    name: "UdpSocket".to_string(),
                    fields: vec![
                        ("fd".to_string(), Expression::Identifier("sock_fd".to_string())),
                    ],
                }),
            ],
            is_async: false,
        }),
        
        // Bind UDP socket
        Declaration::Function(Function {
            type_params: vec![],
            name: "udp_bind".to_string(),
            args: vec![
                ("socket".to_string(), AstType::Pointer(Box::new(AstType::Struct {
                    name: "UdpSocket".to_string(),
                    fields: vec![],
                }))),
                ("port".to_string(), AstType::U16),
            ],
            return_type: AstType::I32,
            body: vec![
                // Implementation would create sockaddr and bind
                Statement::Return(Expression::Integer32(0)),
            ],
            is_async: false,
        }),
        
        // Send UDP datagram
        Declaration::Function(Function {
            type_params: vec![],
            name: "udp_sendto".to_string(),
            args: vec![
                ("socket".to_string(), AstType::Pointer(Box::new(AstType::Struct {
                    name: "UdpSocket".to_string(),
                    fields: vec![],
                }))),
                ("data".to_string(), AstType::Pointer(Box::new(AstType::U8))),
                ("len".to_string(), AstType::U64),
                ("host".to_string(), AstType::String),
                ("port".to_string(), AstType::U16),
            ],
            return_type: AstType::I64,
            body: vec![
                // Implementation would create sockaddr and call sendto
                Statement::Return(Expression::Integer64(0)),
            ],
            is_async: false,
        }),
        
        // Receive UDP datagram
        Declaration::Function(Function {
            type_params: vec![],
            name: "udp_recvfrom".to_string(),
            args: vec![
                ("socket".to_string(), AstType::Pointer(Box::new(AstType::Struct {
                    name: "UdpSocket".to_string(),
                    fields: vec![],
                }))),
                ("buffer".to_string(), AstType::Pointer(Box::new(AstType::U8))),
                ("len".to_string(), AstType::U64),
            ],
            return_type: AstType::Struct {
                name: "RecvResult".to_string(),
                fields: vec![], // (bytes, addr, port)
            },
            body: vec![
                // Implementation would call recvfrom
                Statement::Return(Expression::StructLiteral {
                    name: "RecvResult".to_string(),
                    fields: vec![],
                }),
            ],
            is_async: false,
        }),
    ]
}

fn create_helper_functions() -> Vec<Declaration> {
    vec![
        // Close socket
        Declaration::Function(Function {
            type_params: vec![],
            name: "socket_close".to_string(),
            args: vec![
                ("fd".to_string(), AstType::I32),
            ],
            return_type: AstType::I32,
            body: vec![
                Statement::Return(Expression::FunctionCall {
                    name: "close".to_string(),
                    args: vec![Expression::Identifier("fd".to_string())],
                }),
            ],
            is_async: false,
        }),
        
        // Create socket address
        Declaration::Function(Function {
            type_params: vec![],
            name: "make_sockaddr".to_string(),
            args: vec![
                ("host".to_string(), AstType::String),
                ("port".to_string(), AstType::U16),
            ],
            return_type: AstType::Struct {
                name: "SockAddr".to_string(),
                fields: vec![],
            },
            body: vec![
                Statement::Return(Expression::StructLiteral {
                    name: "SockAddr".to_string(),
                    fields: vec![
                        ("family".to_string(), Expression::Integer32(2)), // AF_INET
                        ("port".to_string(), Expression::FunctionCall {
                            name: "htons".to_string(),
                            args: vec![Expression::Identifier("port".to_string())],
                        }),
                        ("addr".to_string(), Expression::FunctionCall {
                            name: "inet_addr".to_string(),
                            args: vec![Expression::Identifier("host".to_string())],
                        }),
                    ],
                }),
            ],
            is_async: false,
        }),
    ]
}