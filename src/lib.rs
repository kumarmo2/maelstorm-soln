#![allow(dead_code, unused_imports)]
use std::io::StdinLock;
use std::io::StdoutLock;
use std::io::Write;

use anyhow::bail;
use anyhow::Context;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Message<P> {
    pub src: String,
    #[serde(rename = "dest")]
    pub dst: String,
    pub body: Body<P>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Body<P> {
    pub msg_id: Option<usize>,
    pub in_reply_to: Option<usize>,

    #[serde(flatten)]
    pub payload: P,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "snake_case")]
#[serde(tag = "type")]
pub enum InitPayload {
    Init {
        node_id: String,
        node_ids: Vec<String>,
    },

    InitOk {
        in_reply_to: usize,
    },
}

pub trait Node {
    type Payload;
    fn from_init(init: InitPayload) -> Self;
    fn handle_message(
        &mut self,
        in_msg: Message<Self::Payload>,
        std_out: &mut StdoutLock,
    ) -> anyhow::Result<()>;
}

pub fn main_loop<N>() -> anyhow::Result<()>
where
    N: Node,
    <N as Node>::Payload: DeserializeOwned,
{
    let mut std_in = std::io::stdin().lock();
    let mut std_out = std::io::stdout().lock();

    let init_msg: Message<InitPayload> = serde_json::de::Deserializer::from_reader(&mut std_in)
        .into_iter()
        .next()
        .expect("no init message")?;

    let InitPayload::Init { node_id, node_ids } = init_msg.body.payload  else {
                    bail!("first message must be init message");
    };
    let mut node = N::from_init(InitPayload::Init {
        node_id: node_id.clone(),
        node_ids,
    });

    let body = Body {
        msg_id: Some(0),
        in_reply_to: init_msg.body.msg_id,
        payload: InitPayload::InitOk {
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

    let inputs =
        serde_json::de::Deserializer::from_reader(std_in).into_iter::<Message<N::Payload>>();

    for input in inputs {
        let input = input.context("input msg could not be retrieved.")?;
        let _ = node.handle_message(input, &mut std_out);
    }
    Ok(())
}
