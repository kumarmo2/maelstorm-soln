use std::io::{StdoutLock, Write};

use anyhow::Context;
use maelstrom_rs::Node;
use maelstrom_rs::{main_loop, Body, InitPayload, Message};
use serde::{Deserialize, Serialize};

struct EchoNode {
    node_id: String,
    msg_counter: usize,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "type")]
#[serde(rename_all = "snake_case")]
enum EchoPayLoad {
    Echo { echo: String },
    EchoOk { echo: String },
}

impl Node for EchoNode {
    type Payload = EchoPayLoad;
    fn from_init(init: InitPayload) -> Self {
        let InitPayload::Init { node_id, ..} = init else {
            panic!("wrong initpayload variant called");
        };
        Self {
            node_id,
            msg_counter: 1,
        }
    }

    fn handle_message(
        &mut self,
        in_msg: Message<EchoPayLoad>,
        std_out: &mut StdoutLock,
    ) -> anyhow::Result<()> {
        let EchoPayLoad::Echo { echo } = in_msg.body.payload else {
            panic!("wrong type of msg payload passed to echo server");
        };

        let out_msg = Message {
            src: in_msg.dst,
            dst: in_msg.src,
            body: Body {
                msg_id: Some(self.msg_counter),
                in_reply_to: in_msg.body.msg_id,
                payload: EchoPayLoad::EchoOk { echo },
            },
        };

        serde_json::to_writer(&mut *std_out, &out_msg)?;

        std_out
            .write_all(b"\n")
            .context("could not write new line")?;

        self.msg_counter += 1;
        Ok(())
    }
}

fn main() -> anyhow::Result<()> {
    main_loop::<EchoNode>()?;
    Ok(())
}
