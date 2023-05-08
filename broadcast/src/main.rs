use serde::{Deserialize, Serialize};
use std::{io::{self, StdoutLock, Write}, collections::HashMap};

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Message {
    src: String,
    #[serde(rename = "dest")]
    dst: String,
    body: Payload,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
#[serde(rename_all = "snake_case")]
enum Payload {
    Init {
        msg_id: usize,
        node_id: String,
        node_ids: Vec<String>,
    },
    Broadcast {
        msg_id: usize,
        message: usize,
    },
    Read {
        msg_id: usize,
    },
    Topology {
        msg_id: usize,
        topology: HashMap<String, Vec<String>>
    },
    InitOk {
        in_reply_to: usize,
    },
    BroadcastOk {
        in_reply_to: usize,
    },
    ReadOk {
       in_reply_to: usize,
       messages: Vec<usize>,
    },
    TopologyOk {
        in_reply_to: usize,
    }
}

#[derive(Default)]
struct Node {
    id: usize,
    values: Vec<usize>,
    topology: Vec<String>,
}

impl Node {
    fn process(&mut self, msg: Message, output: &mut StdoutLock) -> io::Result<()> {
        match msg.body {
            Payload::Init { msg_id, .. } => {
                let response = Message {
                    src: msg.dst,
                    dst: msg.src,
                    body: Payload::InitOk {
                        in_reply_to: msg_id,
                    },
                };
                serde_json::to_writer(&mut *output, &response)?;
                output.write_all(b"\n")?;
            }
            Payload::Broadcast { msg_id, message } => {
                let response = Message {
                    src: msg.dst,
                    dst: msg.src,
                    body: Payload::BroadcastOk { in_reply_to: msg_id },
                };
                self.values.push(message);
                serde_json::to_writer(&mut *output, &response)?;
                output.write_all(b"\n")?;
            }
            Payload::Read { msg_id } => {
                let response = Message {
                    src: msg.dst,
                    dst: msg.src,
                    body: Payload::ReadOk { 
                        in_reply_to: msg_id,  
                        messages: self.values.clone(), 
                    },
                };
                serde_json::to_writer(&mut *output, &response)?;
                output.write_all(b"\n")?;
            }
            Payload::Topology { msg_id, .. } => {                
                let response = Message {
                    src: msg.dst,
                    dst: msg.src,
                    body: Payload::TopologyOk { in_reply_to: msg_id },
                };
                serde_json::to_writer(&mut *output, &response)?;
                output.write_all(b"\n")?;
            }
            Payload::ReadOk { .. } => panic!("Should not receive read_ok message"),
            Payload::BroadcastOk { .. } => panic!("Should not receive broadcast_ok message"),
            Payload::TopologyOk { .. } => panic!("Should not receive topology_ok message"),
            Payload::InitOk { .. } => panic!("Should not receive init_ok message"),
        };
        Ok(())
    }
}

fn main() -> io::Result<()> {
    // initialization
    let stdin = io::stdin().lock();
    let mut stdout = io::stdout().lock();

    let mut node = Node { id: 0, values: vec![1,2,3], topology: vec![] };

    let inputs = serde_json::Deserializer::from_reader(stdin).into_iter::<Message>();

    for input in inputs {
        let input = input.expect("Error deserializing message");
        node.process(input, &mut stdout)
            .expect("Error processing message");
        node.id += 1;
    }
    Ok(())
}
