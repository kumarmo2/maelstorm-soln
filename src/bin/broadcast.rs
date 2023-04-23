#![allow(dead_code, unused_variables)]

use std::{
    collections::{HashMap, HashSet},
    io::{StdoutLock, Write},
    mem,
};

use anyhow::{bail, Context};
use maelstrom_rs::{main_loop, Body, InitPayload, Message, Node};
use serde::{Deserialize, Serialize};
struct BroadcastNode {
    node_id: String,
    msg_counter: usize,
    messages: Vec<i32>,
    messages_set: HashSet<i32>,
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

impl BroadcastNode {
    pub(crate) fn broadcast_to_neighbours(
        &self,
        message: i32,
        in_msg_id: Option<usize>,
        std_out: &mut StdoutLock,
    ) -> anyhow::Result<()> {
        let Some(neighbours) =  self.topology.get(&self.node_id) else {
            bail!("no neighbour found");
        };

        for neighbour in neighbours {
            let payload = BroadcastPayload::Broadcast { message };
            let out = Message {
                src: self.node_id.to_owned(),
                dst: neighbour.to_string(),
                body: Body {
                    msg_id: None,
                    in_reply_to: in_msg_id,
                    payload,
                },
            };
            serde_json::to_writer(&mut *std_out, &out)?;

            std_out
                .write_all(b"\n")
                .context("could not write new line")?;
        }

        Ok(())
    }

    fn handle_broadcast(
        &mut self,
        in_msg: Message<BroadcastPayload>,
        message: i32,
        std_out: &mut StdoutLock,
    ) -> anyhow::Result<()> {
        if self.messages_set.contains(&message) {
            return Ok(());
        }

        self.messages.push(message);
        self.messages_set.insert(message);
        // handlebro
        self.broadcast_to_neighbours(message, in_msg.body.msg_id, std_out)?;

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
        Ok(())
    }
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
            messages_set: HashSet::new(),
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
                self.handle_broadcast(in_msg, message, std_out)?;
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
                // let vec_message: Vec<_> = messages.iter().collect();

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
