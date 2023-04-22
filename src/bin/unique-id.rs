use std::io::Write;

use maelstrom_rs::{main_loop, InitPayload, Message, Node};
use serde::{Deserialize, Serialize};

struct UniqueNode {
    node_id: String,
    msg_counter: usize,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "type")]
#[serde(rename_all = "snake_case")]
pub enum UnqiuePayload {
    Generate,
    GenerateOk {
        #[serde(rename = "id")]
        guid: String,
    },
}

impl Node for UniqueNode {
    type Payload = UnqiuePayload;

    fn from_init(init: maelstrom_rs::InitPayload) -> Self {
        let InitPayload::Init { node_id, ..}= init else {
            panic!("init must be passed");
        };
        Self {
            node_id,
            msg_counter: 1,
        }
    }

    fn handle_message(
        &mut self,
        in_msg: maelstrom_rs::Message<Self::Payload>,
        std_out: &mut std::io::StdoutLock,
    ) -> anyhow::Result<()> {
        match in_msg.body.payload {
            UnqiuePayload::Generate => {
                let output = Message {
                    src: in_msg.dst,
                    dst: in_msg.src,
                    body: maelstrom_rs::Body {
                        msg_id: Some(self.msg_counter),
                        in_reply_to: in_msg.body.msg_id,
                        payload: UnqiuePayload::GenerateOk {
                            guid: format!("{}-{}", self.node_id, self.msg_counter),
                        },
                    },
                };

                serde_json::to_writer(&mut *std_out, &output)?;
                std_out.write_all(b"\n")?;
            }
            UnqiuePayload::GenerateOk { .. } => {}
        };
        self.msg_counter += 1;
        Ok(())
    }
}

fn main() -> anyhow::Result<()> {
    main_loop::<UniqueNode>()?;
    Ok(())
}
