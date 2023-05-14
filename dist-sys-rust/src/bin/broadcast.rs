use serde::{Deserialize, Serialize};
use std::{io::{self, StdoutLock, Write}, collections::HashMap};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag="type")]
#[serde(rename_all = "snake_case")]
enum Payload {
    // Requests
    Init {
        msg_id: usize,
        node_id: String,
        node_ids: Vec<String>,
    },
    Read {
        msg_id: usize,
    },
    Broadcast {
        msg_id: usize,
        message: usize 
    },
    Topology { 
        msg_id: usize,
        topology: HashMap<String, Vec<String>> 
    },
    // Responses
    InitOk {
        in_reply_to: usize,
    },
    BroadcastOk {
        in_reply_to: usize,
    },
    ReadOk { 
        in_reply_to: usize,
        messages: Vec<usize> 
    },
    TopologyOk {
        in_reply_to: usize,
    },
}


#[derive(Debug, Clone, Serialize, Deserialize)]
struct Message {
    src: String,
    #[serde(rename = "dest")]
    dst: String,
    body: Payload,
}


#[derive(Default)]
struct Node {
    id: usize,
    values: Vec<usize>,
}


impl Node {
    fn new(id: usize) -> Self {
        Node { id, values: vec![] }
    }

    fn process(&mut self, msg: Message, output: &mut StdoutLock) -> io::Result<()> {
        let response_body = match msg.body {
            Payload::Init { msg_id, .. } => {
                Payload::InitOk { in_reply_to: msg_id } 
            }
            Payload::Read { msg_id } => {
                Payload::ReadOk { 
                    in_reply_to: msg_id,
                    messages: self.values.clone(), 
                }
            }
            Payload::Broadcast { msg_id, message } => {
                self.values.push(message);
                Payload::BroadcastOk { in_reply_to: msg_id }
            }
            Payload::Topology { msg_id, .. } => {
                Payload::TopologyOk { in_reply_to: msg_id }
            }
            _m => panic!("Should not receive {:?} message", _m),
        };

        let response = Message {
            src: msg.dst,
            dst: msg.src,
            body: response_body, 
        };
        serde_json::to_writer(&mut *output, &response)?;
        output.write_all(b"\n")?;

        Ok(())
    }
}

fn main() -> io::Result<()> {
    // initialization
    let stdin = io::stdin().lock();
    let mut stdout = io::stdout().lock();

    let mut node = Node::new(0);

    let inputs = serde_json::Deserializer::from_reader(stdin).into_iter::<Message>();

    for input in inputs {
        let input = input.expect("Error deserializing message");
        node.process(input, &mut stdout)
            .expect("Error processing message");
        node.id += 1;
    }
    Ok(())
}
