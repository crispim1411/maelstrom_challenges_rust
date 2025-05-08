use anyhow::bail;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::sync::mpsc;
use std::thread;
use std::io::{StdoutLock, Write, self, StdinLock, Lines};

const LIN_KV: &str = "lin-kv";
const KEY: &str = "counter";
const ERROR_MSG: &str = r"current value (?P<current>\d+) is not (?P<expected>\d+)";

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
enum Payload {
    Read { key: Option<String> },
    ReadOk { value: usize },
    Add { delta: usize },
    AddOk,
    Cas { 
        key: String,
        from: usize, 
        to:usize,
        create_if_not_exists: bool,
    },
    CasOk,
    Error {
        code: usize,
        text: String,
    }
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
    Read,
    Cas(usize),
    Eof,
}

struct Node {
    id: usize, 
    node_id: String,
    event_sender: mpsc::Sender<Event>,
    counter: usize,
    last_value: usize,
}

impl Node {
    fn from_init(message: Init, tx: mpsc::Sender<Event>) -> Self {
        Node {
            id: 1,
            node_id: message.node_id,
            event_sender: tx,
            counter: 0,
            last_value: 0,
        }
    }

    fn process(&mut self, msg: Message<Payload>, output: &mut StdoutLock) -> anyhow::Result<()> {
        let mut reply = msg.into_reply(Some(self.id));
        match reply.body.payload {
            Payload::Read { .. } => {
                reply.body.payload = Payload::ReadOk { 
                    value: self.counter
                };
                reply.send(output)?;
                self.event_sender.send(Event::Read)?;
            }
            Payload::Add { delta } => {
                if delta > 0 {
                    self.last_value = delta;
                    self.event_sender.send(Event::Cas(delta))?;    
                }
                reply.body.payload = Payload::AddOk;
                reply.send(output)?;
            }
            Payload::ReadOk { value } => {
                self.counter = value;
            }
            Payload::CasOk => {
                self.event_sender.send(Event::Read)?;
            }
            Payload::Error { code, text } => {
                if code == 20 {
                    self.event_sender.send(Event::Cas(0))?;
                }
                else if code == 22 {
                    let re = Regex::new(ERROR_MSG).unwrap();
                    if let Some(matches) = re.captures(&text) {
                        if let Ok(current_value) = matches["current"].parse::<usize>() {
                            self.counter = current_value;
                        }
                        self.event_sender.send(Event::Cas(self.last_value))?;
                    } 
                }
            }
            Payload::AddOk | Payload::Cas { .. } => {},
        }
        self.id += 1;
        Ok(())
    }

    fn handle_event(&mut self, event: Event, output: &mut StdoutLock) -> anyhow::Result<()> {
        match event { 
            Event::Eof => {}
            Event::Cas(delta) => {
                let message = Message {
                    src: self.node_id.clone(),
                    dst: "lin-kv".to_string(),
                    body: Body { 
                        msg_id: None, 
                        in_reply_to: None, 
                        payload: Payload::Cas { 
                            key: KEY.to_string(),
                            from: self.counter, 
                            to: self.counter + delta,
                            create_if_not_exists: true
                        }
                    }
                };
                message.send(output)?;
            }
            Event::Read => {
                let message = Message {
                    src: self.node_id.clone(),
                    dst: LIN_KV.to_string(),
                    body: Body { 
                        msg_id: None, 
                        in_reply_to: None, 
                        payload: Payload::Read {
                            key: Some(KEY.to_string())
                        }
                    }
                };
                message.send(output)?;
            }
            Event::Message(msg) => {
                self.process(msg, output)?;
            }
        }
        Ok(())
    }
}

fn main() -> anyhow::Result<()> {
    let stdin = io::stdin();
    let stdout = io::stdout();
    let (tx, rx) = mpsc::channel();

    let init_msg = wait_for_initialization(&mut stdin.lines(), &mut stdout.lock())
        .expect("Expected init message");
    let mut node = Node::from_init(init_msg, tx.clone());

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
        let _ = tx.send(Event::Eof);
        Ok(())
    });

    for event in rx {
        node
            .handle_event(event, &mut stdout.lock())
            .expect("node failed processing message");
    }

    main_thread.join().expect("Error running main thread")?;

    Ok(())
}

fn wait_for_initialization(input: &mut Lines<StdinLock>, output: &mut StdoutLock) -> anyhow::Result<Init> {
    let msg: Message<InitPayload> = serde_json::from_str(
        &input
        .next()
        .expect("no message received")?)?;
    let InitPayload::Init(init_msg) = msg.body.payload.clone() else {
        bail!("Expected init message")
    };
    let mut reply = msg.into_reply(Some(0));
    reply.body.payload = InitPayload::InitOk;
    reply.send(output)?;
    Ok(init_msg)
}
