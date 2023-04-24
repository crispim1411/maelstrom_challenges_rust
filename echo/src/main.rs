use serde::{Deserialize, Serialize};
use std::io::{self, Write};

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
}

fn main() -> io::Result<()> {
    // initialization
    let mut stdin = io::stdin().lock();
    let mut stdout = io::stdout().lock();

    // reading from stdin
    let mut counter = 0;

    // todo: separate into Node struct
    let mut unique_id = String::new();

    let inputs = serde_json::Deserializer::from_reader(stdin).into_iter::<Message>();

    for input in inputs {
        let input = input.expect("Error deserializing message");
        match input.body {
            Payload::Init { msg_id, .. } => {
                let response = Message {
                    src: input.dst,
                    dst: input.src,
                    body: Payload::InitOk {
                        in_reply_to: msg_id,
                    },
                };
                serde_json::to_writer(&mut stdout, &response)?;
                stdout.write_all(b"\n")?;
            }
            Payload::Echo { msg_id, echo, .. } => {
                let response = Message {
                    src: input.dst,
                    dst: input.src,
                    body: Payload::EchoOk {
                        msg_id,
                        in_reply_to: msg_id,
                        echo,
                    },
                };
                serde_json::to_writer(&mut stdout, &response)?;
                stdout.write_all(b"\n")?;
            }
            Payload::InitOk { .. } => {
                panic!("Should not receive init ok message");
            }
            Payload::EchoOk { .. } => {
                panic!("Should not receive echo ok message");
            }
        };
    }
    Ok(())
}
