use serde::{Deserialize, Serialize};
use std::io::{self, StdoutLock, Write};
use uuid::Uuid;

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
    InitOk {
        in_reply_to: usize,
    },
    Echo {
        msg_id: usize,
        echo: String,
    },
    EchoOk {
        msg_id: usize,
        in_reply_to: usize,
        echo: String,
    },
    Generate {
        msg_id: usize,
    },
    GenerateOk {
        id: String,
        in_reply_to: usize,
    }
}

struct Node {
    id: usize,
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
            Payload::Echo { msg_id, echo, .. } => {
                let response = Message {
                    src: msg.dst,
                    dst: msg.src,
                    body: Payload::EchoOk {
                        msg_id,
                        in_reply_to: msg_id,
                        echo,
                    },
                };
                serde_json::to_writer(&mut *output, &response)?;
                output.write_all(b"\n")?;
            }
            Payload::Generate { msg_id } => {
                let response = Message {
                    src: msg.dst,
                    dst: msg.src,
                    body: Payload::GenerateOk { 
                        id: Uuid::new_v4().to_string(),
                        in_reply_to: msg_id, 
                    }
                };
                serde_json::to_writer(&mut *output, &response)?;
                output.write_all(b"\n")?;
            }
            Payload::InitOk { .. } => {
                panic!("Should not receive init_ok message");
            }
            Payload::EchoOk { .. } => {
                panic!("Should not receive echo_ok message");
            }
            Payload::GenerateOk { .. } => {
                panic!("Should not receive generate_ok message");
            }
        };
        Ok(())
    }
}

fn main() -> io::Result<()> {
    // initialization
    let stdin = io::stdin().lock();
    let mut stdout = io::stdout().lock();

    let mut node = Node { id: 0 };

    let inputs = serde_json::Deserializer::from_reader(stdin).into_iter::<Message>();

    for input in inputs {
        let input = input.expect("Error deserializing message");
        node.process(input, &mut stdout)
            .expect("Error processing message");
        node.id += 1;
    }
    Ok(())
}
