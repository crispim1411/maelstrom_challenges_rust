use anyhow::bail;
use serde::{Deserialize, Serialize};
use std::io::{self, StdoutLock, Write, BufRead};

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Message<T> {
    src: String,
    #[serde(rename = "dest")]
    dst: String,
    body: Body<T>,
}

impl<T> Message<T> where T: Serialize {
    fn into_reply(self, id: Option<usize>) -> Self {
        Message {
            src: self.dst,
            dst: self.src,
            body: Body {
                msg_id: id,
                in_reply_to: self.body.msg_id,
                payload: self.body.payload, 
            }
        }
    }

    fn send(&self, output: &mut StdoutLock) -> anyhow::Result<()> {
        serde_json::to_writer(&mut *output, &self)?;
        output.write_all(b"\n")?;
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Body<T> {
    msg_id: Option<usize>,
    in_reply_to: Option<usize>,
    #[serde(flatten)]
    payload: T,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
#[serde(rename_all = "snake_case")]
enum InitPayload {
    Init(Init),
    InitOk,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
#[serde(rename_all = "snake_case")]
enum Payload {
    Echo { echo: String },
    EchoOk { echo: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Init {
    node_id: String,
    node_ids: Vec<String>,
}

struct Node {
    id: usize,
    node_id: String,
}

impl Node {
    fn from_init(msg: Init) -> Self {
        Node { id: 0, node_id: msg.node_id }
    }

    fn process(&mut self, msg: Message<Payload>, output: &mut StdoutLock) -> anyhow::Result<()> {
        let mut reply = msg.into_reply(Some(self.id));
        match reply.body.payload {
            Payload::Echo { echo } => {
                reply.body.payload = Payload::EchoOk { echo };
                reply.send(output)?;
            }
            Payload::EchoOk { .. } => { bail!("Should not receive echo_ok message") },
        };
        self.id += 1;
        Ok(())
    }
}

fn main() -> anyhow::Result<()> {
    let stdin = io::stdin().lock();
    let mut stdout = io::stdout().lock();

    let mut stdin = stdin.lines();

    let msg: Message<InitPayload> = serde_json::from_str(
        &stdin.next().expect("no message received")?)?;

    let InitPayload::Init(init_msg) = msg.body.payload.clone() else {
        panic!("Expected init message");
    };

    let mut node = Node::from_init(init_msg);
    let mut reply = msg.into_reply(Some(0));
    reply.body.payload = InitPayload::InitOk;
    reply.send(&mut stdout)?;

    for line in stdin {
            let input: Message<Payload> = serde_json::from_str(&line?)?;
        node.process(input, &mut stdout)
            .expect("Error processing message");
        node.id += 1;
    }
    Ok(())
}
