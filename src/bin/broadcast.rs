#![allow(dead_code, unused_variables)]

use std::{collections::HashMap, io::Write, mem};

use anyhow::{bail, Context};
use maelstrom_rs::{main_loop, Body, InitPayload, Message, Node};
use serde::{Deserialize, Serialize};
struct BroadcastNode {
    node_id: String,
    msg_counter: usize,
    messages: Vec<i32>,
    topology: HashMap<String, Vec<String>>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "type")]
#[serde(rename_all = "snake_case")]
enum BroadcastPayload {
    Broadcast {
        message: i32,
    },
    BroadcastOk,
    Read,
    // TODO: check if any "borrowed" version of messages could be used here.
    ReadOk {
        messages: Vec<i32>,
    },
    Topology {
        topology: HashMap<String, Vec<String>>,
    },
    TopologyOk,
}

impl Node for BroadcastNode {
    type Payload = BroadcastPayload;

    fn from_init(init: maelstrom_rs::InitPayload) -> Self {
        let InitPayload::Init {
        node_id,
        node_ids,

    } = init else {
            panic!("wrong init payload");

        };
        Self {
            node_id,
            msg_counter: 1,
            messages: Vec::new(),
            topology: HashMap::new(),
        }
    }

    fn handle_message(
        &mut self,
        in_msg: maelstrom_rs::Message<Self::Payload>,
        std_out: &mut std::io::StdoutLock,
    ) -> anyhow::Result<()> {
        match in_msg.body.payload {
            BroadcastPayload::Broadcast { message } => {
                self.messages.push(message);
                let output = Message {
                    src: in_msg.dst,
                    dst: in_msg.src,
                    body: Body {
                        msg_id: Some(self.msg_counter),
                        in_reply_to: in_msg.body.msg_id,
                        payload: BroadcastPayload::BroadcastOk,
                    },
                };
                serde_json::to_writer(&mut *std_out, &output)?;

                std_out
                    .write_all(b"\n")
                    .context("could not write new line")?;
            }
            BroadcastPayload::BroadcastOk => {
                self.msg_counter += 1;
                bail!("BroadcastOk should not come as input");
            }
            BroadcastPayload::Read => {
                let empty = Vec::new();
                /* NOTE:
                 * since the ReadOk needs the ownership of "messages", we temporarily
                 * basically replace self.messages with an empty vector and "take out" messages,
                 * created response, send the response stream, and the replace self.messages again
                 * with the original vector.
                 * */
                let messages = std::mem::replace(&mut self.messages, empty);

                let payload = BroadcastPayload::ReadOk { messages };
                let output = Message {
                    src: in_msg.dst,
                    dst: in_msg.src,
                    body: Body {
                        msg_id: Some(self.msg_counter),
                        in_reply_to: in_msg.body.msg_id,
                        payload,
                    },
                };
                serde_json::to_writer(&mut *std_out, &output)?;

                std_out
                    .write_all(b"\n")
                    .context("could not write new line")?;

                let messages = match output.body.payload {
                    BroadcastPayload::ReadOk { messages } => messages,
                    _ => unreachable!(),
                };
                let _ = mem::replace(&mut self.messages, messages);
            }
            BroadcastPayload::ReadOk { messages } => {
                self.msg_counter += 1;
                bail!("BroadcastOk should not come as input");
            }
            BroadcastPayload::Topology { topology } => {
                self.topology = topology;
                let payload = BroadcastPayload::TopologyOk;
                let output = Message {
                    src: in_msg.dst,
                    dst: in_msg.src,
                    body: Body {
                        msg_id: Some(self.msg_counter),
                        in_reply_to: in_msg.body.msg_id,
                        payload,
                    },
                };
                serde_json::to_writer(&mut *std_out, &output)?;

                std_out
                    .write_all(b"\n")
                    .context("could not write new line")?;
            }
            BroadcastPayload::TopologyOk => {
                self.msg_counter += 1;
                bail!("TopologyOk should not come as input");
            }
        };

        self.msg_counter += 1;
        Ok(())
    }
}

fn main() -> anyhow::Result<()> {
    main_loop::<BroadcastNode>()?;
    Ok(())
}
