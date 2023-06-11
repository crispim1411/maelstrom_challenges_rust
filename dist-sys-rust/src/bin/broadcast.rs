use anyhow::bail;
use serde::{Deserialize, Serialize};
use std::sync::mpsc;
use std::collections::{HashMap, HashSet};
use std::thread;
use std::time::Duration;
use std::io::{StdoutLock, Write, self};

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
enum Payload  {
    Read,
    ReadOk { messages: Vec<usize> },
    Broadcast { message: usize },
    BroadcastOk,
    Topology { topology: HashMap<String, Vec<String>> },
    TopologyOk,
    Gossip { seen: Vec<usize> }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag="type")]
#[serde(rename_all = "snake_case")]
enum InitPayload {
    Init(Init),
    InitOk,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Init {
    node_id: String,
    node_ids: Vec<String>,
}

#[derive(Debug, Clone)]
enum Event {
    Message(Message<Payload>),
    Gossip,
    EOF,
}

#[derive(Default)]
struct Node {
    id: usize, 
    node_id: String,
    seen: HashSet<usize>,
    known: HashMap<String, HashSet<usize>>,
    neighborhood: Vec<String>, 
}

impl Node {
    fn from_init(message: Init, tx: mpsc::Sender<Event>) -> Self {
        thread::spawn(move || {
            loop { 
                thread::sleep(Duration::from_millis(300));
                if tx.send(Event::Gossip).is_err() {
                    break;
                }
            }
        });
        Node {
            id: 1,
            node_id: message.node_id,
            seen:HashSet::new(),
            known: message
                .node_ids
                .into_iter()
                .map(|node_id| (node_id, HashSet::new()))
                .collect(),
            neighborhood: Vec::new(),
        }
    }

    fn process(&mut self, msg: Message<Payload>, output: &mut StdoutLock) -> anyhow::Result<()> {
        let mut reply = msg.into_reply(Some(self.id));
        match reply.body.payload {
            Payload::Read => {
                reply.body.payload = Payload::ReadOk { 
                    messages: self.seen
                        .iter()
                        .copied()
                        .collect(), 
                };
                reply.send(output)?;
            }
            Payload::Broadcast { message } => {
                self.seen.insert(message);
                reply.body.payload = Payload::BroadcastOk;
                reply.send(output)?;
            }
            Payload::Topology { mut topology } => {
                self.neighborhood = topology
                    .remove(&self.node_id)
                    .unwrap_or_else(|| panic!("No topology sent to Node {}", self.node_id));
                reply.body.payload = Payload::TopologyOk; 
                reply.send(output)?;
            }
            Payload::Gossip { seen } => {
                self.seen.extend(seen);
            }
            Payload::ReadOk { .. }| Payload::BroadcastOk | Payload::TopologyOk => {}
        }
        self.id += 1;
        Ok(())
    }

    fn handle_event(&mut self, event: Event, output: &mut StdoutLock) -> anyhow::Result<()> {
        match event { 
            Event::EOF => {}
            Event::Message(msg) => {
                self.process(msg, output)?;
            }
            Event::Gossip => {
                for node in &self.neighborhood {
                    let reply = self.create_gossip(node);
                    reply.send(output)?;
                }
            }
        }
        Ok(())
    }

    fn create_gossip(&self, node: &String) -> Message<Payload> {
        let known_to_n = &self.known[node];
        Message {
            src: self.node_id.clone(),
            dst: node.clone(),
            body: Body {
                msg_id: None,
                in_reply_to: None,
                payload: Payload::Gossip { 
                    seen: self.seen
                        .iter()
                        .copied()
                        .filter(|x| !known_to_n.contains(x))
                        .collect()
                }
            }
        }
    }
}

fn main() -> anyhow::Result<()> {
    let stdin = io::stdin();
    let mut stdout = io::stdout().lock();
    let (tx, rx) = mpsc::channel();

    let mut stdin = stdin.lines();
    let msg: Message<InitPayload> = serde_json::from_str(
        &stdin
        .next()
        .expect("no message received")?)?;
    let InitPayload::Init(init_msg) = &msg.body.payload else {
        panic!("Expected init message");
    };

    let mut node = Node::from_init(init_msg.clone(), tx.clone());

    let mut reply = msg.into_reply(Some(0));
    reply.body.payload = InitPayload::InitOk;
    reply.send(&mut stdout)?;
    
    drop(stdin);
    let main_thread = thread::spawn(move || {
        let stdin = io::stdin();
        for line in stdin.lines() {
            let input = line.expect("no message received");
            let msg: Message<Payload> = serde_json::from_str(&input)
                .expect("Error deserializing message");
            if tx.send(Event::Message(msg)).is_err() {
                bail!("Error tx send");
            };
        }
        let _ = tx.send(Event::EOF);
        Ok(())
    });

    for event in rx  {
        node
            .handle_event(event, &mut stdout)
            .expect("node failed processing message");
    }

    main_thread.join().expect("Error running main thread")?;

    Ok(())
}
