use puffin::{
    algebra::{signature::Signature, Matcher},
    error::Error,
    protocol::{
        MessageResult, OpaqueProtocolMessage, ProtocolBehavior, ProtocolMessage,
        ProtocolMessageDeframer,
    },
    put::{PutDescriptor, PutName},
    put_registry::{Factory, PutRegistry},
    trace::Trace,
    variable_data::VariableData,
};

use crate::{
    claims::TlsClaim,
    debug::{debug_message_with_info, debug_opaque_message_with_info},
    protocol::TLSProtocolBehavior,
    query::TlsQueryMatcher,
    tls::{
        rustls::{
            msgs,
            msgs::message::{Message, OpaqueMessage},
        },
        seeds::create_corpus,
        violation::TlsSecurityViolationPolicy,
        TLS_SIGNATURE,
    },
};

pub const OPENSSL111_PUT: PutName = PutName(['O', 'P', 'E', 'N', 'S', 'S', 'L', '1', '1', '1']);
pub const WOLFSSL520_PUT: PutName = PutName(['W', 'O', 'L', 'F', 'S', 'S', 'L', '5', '2', '0']);
pub const TCP_PUT: PutName = PutName(['T', 'C', 'P', '_', '_', '_', '_', '_', '_', '_']);

pub const TLS_PUT_REGISTRY: PutRegistry<TLSProtocolBehavior> = PutRegistry {
    factories: &[
        crate::tcp::new_tcp_factory,
        #[cfg(feature = "openssl-binding")]
        crate::openssl::new_openssl_factory,
        #[cfg(feature = "wolfssl-binding")]
        crate::wolfssl::new_wolfssl_factory,
    ],
    default: DEFAULT_PUT_FACTORY,
};

pub const DEFAULT_PUT_FACTORY: fn() -> Box<dyn Factory<TLSProtocolBehavior>> = {
    cfg_if::cfg_if! {
        if #[cfg(feature = "openssl-binding")] {
            crate::openssl::new_openssl_factory
        } else if #[cfg(feature = "wolfssl-binding")] {
            crate::wolfssl::new_wolfssl_factory
        } else {
             crate::tcp::new_tcp_factory
        }
    }
};
