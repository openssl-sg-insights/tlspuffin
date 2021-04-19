use std::any::Any;

use rustls::internal::msgs::codec::Codec;
use rustls::internal::msgs::enums::ContentType::Handshake as RecordHandshake;
use rustls::internal::msgs::enums::{AlertLevel, HandshakeType};
use rustls::internal::msgs::handshake::{
    ClientHelloPayload, HandshakeMessagePayload, HandshakePayload,
};
use rustls::internal::msgs::message::Message;
use rustls::internal::msgs::message::MessagePayload::Handshake;
use rustls::ProtocolVersion;

use crate::agent::{Agent, AgentName};
use crate::debug::debug_message;
use crate::io::MemoryStream;
use crate::openssl_server;
use crate::openssl_server::openssl_version;
use crate::variable::{
    CipherSuiteData, ClientVersionData, CompressionData, ExtensionData, RandomData, SessionIDData,
    VariableData,
};

pub struct TraceContext {
    variables: Vec<Box<dyn VariableData>>,
    agents: Vec<Agent>,
}

impl TraceContext {
    pub fn new() -> TraceContext {
        TraceContext {
            variables: vec![],
            agents: vec![],
        }
    }

    pub fn add_variable(&mut self, data: Box<dyn VariableData>) {
        self.variables.push(data)
    }

    // Why do we need to extend Any here? do we need to make sure that the types T are known during
    // compile time?
    fn downcast<T: Any>(variable: &dyn VariableData) -> Option<&T> {
        variable.as_any().downcast_ref::<T>()
    }

    fn get_variable<T: Any>(&self) -> Option<&T> {
        for variable in &self.variables {
            if let Some(derived) = TraceContext::downcast(variable.as_ref()) {
                return Some(derived);
            }
        }
        None
    }

    fn get_variable_set<T: Any>(&self) -> Vec<&T> {
        let mut variables: Vec<&T> = Vec::new();
        for variable in &self.variables {
            if let Some(derived) = TraceContext::downcast(variable.as_ref()) {
                variables.push(derived);
            }
        }
        variables
    }

    pub fn publish(&mut self, sending_agent_name: AgentName, data: &dyn AsRef<[u8]>) {
        for agent in self.agents.iter_mut() {
            if agent.name != sending_agent_name {
                agent.stream.extend_incoming(data.as_ref());
            }
        }
    }

    pub fn new_agent(&mut self) -> AgentName {
        let agent = Agent::new();
        let name = agent.name;
        self.agents.push(agent);
        return name;
    }
}

pub struct Trace<'a> {
    pub steps: Vec<Step<'a>>,
}

impl<'a> Trace<'a> {
    pub fn execute(&mut self, ctx: &mut TraceContext) {
        for step in self.steps.iter_mut() {
            step.action.execute(ctx);
        }
    }
}

pub struct Step<'a> {
    pub from: AgentName,
    pub to: AgentName,
    pub action : &'a (dyn Action + 'static),
}

pub trait Action {
    fn execute(&self, ctx: &mut TraceContext);
}

pub trait SendAction: Action {
    fn craft(&self, ctx: &TraceContext) -> Result<Vec<u8>, ()>;
}

pub trait ExpectAction: Action {
    fn get_concrete_variables(&self) -> Vec<String>; // Variables and the actual values
}

// ServerHello

pub struct ServerHelloExpectAction {}

impl Action for ServerHelloExpectAction {
    fn execute(&self, ctx: &mut TraceContext) {
        // TODO
        // let buffer = ctx.receive_from_previous();
        // openssl_server::process(ssl_stream)
    }
}

impl ServerHelloExpectAction {
    pub fn new() -> ServerHelloExpectAction {
        ServerHelloExpectAction {}
    }
}

impl ExpectAction for ClientHelloSendAction {
    fn get_concrete_variables(&self) -> Vec<String> {
        todo!()
    }
}

// ClientHello

pub struct ClientHelloSendAction {
}

impl Action for ClientHelloSendAction {
    fn execute(&self, ctx: &mut TraceContext) {
        let result = self.craft(ctx);

        match result {
            Ok(buffer) => {
                debug_message(&buffer);
                //ctx.publish(self.agent, &buffer);
            }
            _ => {
                println!("Error");
            }
        }
    }
}

impl ClientHelloSendAction {
    pub fn new() -> ClientHelloSendAction {
        ClientHelloSendAction { }
    }
}

impl SendAction for ClientHelloSendAction {
    fn craft(&self, ctx: &TraceContext) -> Result<Vec<u8>, ()> {
        return if let (
            Some(client_version),
            Some(random),
            Some(session_id),
            ciphersuits,
            compression_methods,
            extensions,
        ) = (
            ctx.get_variable::<ClientVersionData>(),
            ctx.get_variable::<RandomData>(),
            ctx.get_variable::<SessionIDData>(),
            ctx.get_variable_set::<CipherSuiteData>(),
            ctx.get_variable_set::<CompressionData>(),
            ctx.get_variable_set::<ExtensionData>(),
        ) {
            let payload = Handshake(HandshakeMessagePayload {
                typ: HandshakeType::ClientHello,
                payload: HandshakePayload::ClientHello(ClientHelloPayload {
                    client_version: client_version.data,
                    random: random.data.clone(),
                    session_id: session_id.data,
                    cipher_suites: ciphersuits.into_iter().map(|c| c.data).collect(),
                    compression_methods: compression_methods.into_iter().map(|c| c.data).collect(),
                    extensions: extensions.into_iter().map(|c| c.data.clone()).collect(),
                }),
            });
            let message = Message {
                typ: RecordHandshake,
                version: ProtocolVersion::TLSv1_3,
                payload,
            };

            let mut out: Vec<u8> = Vec::new();
            message.encode(&mut out);
            Ok(out)
        } else {
            Err(())
        };
    }
}
