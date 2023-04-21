// #![allow(dead_code, unused_imports)]
use std::io::StdoutLock;
use std::io::Write;

use anyhow::Context;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
struct Message {
    src: String,
    #[serde(rename = "dest")]
    dst: String,
    body: Body,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct Body {
    msg_id: Option<usize>,
    in_reply_to: Option<usize>,

    #[serde(flatten)]
    payload: Payload,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "snake_case")]
#[serde(tag = "type")]
enum Payload {
    Init {
        node_id: String,
        node_ids: Vec<String>,
    },

    InitOk {
        in_reply_to: usize,
    },
    Echo {
        echo: String,
    },
    EchoOk {
        echo: String,
    },
}

struct EchoNode {
    node_id: String,
    msg_counter: usize,
}

impl EchoNode {
    fn handle_message(&mut self, in_msg: Message, std_out: &mut StdoutLock) -> anyhow::Result<()> {
        let Payload::Echo { echo } = in_msg.body.payload else {
            panic!("wrong type of msg payload passed to echo server");
        };

        let out_msg = Message {
            src: self.node_id.clone(),
            dst: in_msg.src,
            body: Body {
                msg_id: Some(self.msg_counter),
                in_reply_to: in_msg.body.msg_id,
                payload: Payload::EchoOk { echo },
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
    let std_in = std::io::stdin().lock();
    let mut std_out = std::io::stdout().lock();

    let mut inputs = serde_json::de::Deserializer::from_reader(std_in).into_iter::<Message>();

    let init_msg = inputs.next().context("could not get the init message")?;
    let Ok(init_msg) = init_msg else {
    panic!("could not get the init message deser working");
    };

    let Payload::Init { node_id, .. } = init_msg.body.payload  else {
    panic!("first message must be init message");
    };

    let body = Body {
        msg_id: Some(0),
        in_reply_to: init_msg.body.msg_id,
        payload: Payload::InitOk {
            in_reply_to: init_msg.body.msg_id.unwrap(),
        },
    };

    let init_ok_msg = Message {
        src: node_id.clone(),
        dst: init_msg.src,
        body,
    };

    serde_json::to_writer(&mut std_out, &init_ok_msg).context("could not write")?;
    std_out
        .write_all(b"\n")
        .context("could not write new line")?;

    let mut node = EchoNode {
        node_id,
        msg_counter: 1,
    };

    for input in inputs {
        let input: Message = input.context("input msg could not be retrieved.")?;
        let _ = node.handle_message(input, &mut std_out);
    }
    Ok(())
}
