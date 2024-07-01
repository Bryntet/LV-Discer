use std::collections::VecDeque;

#[cfg(not(target_arch = "wasm32"))]
use {
    std::io::{Read, Write},
    std::net::{IpAddr, SocketAddr, TcpStream},
    std::str::FromStr,
    std::sync::Mutex,
};

use std::sync::Arc;
use crate::api::MyError;

#[cfg(target_arch = "wasm32")]
#[derive(Clone)]
pub struct Queue {
    functions: Arc<Mutex<VecDeque<String>>>,
    ip: String,
    client: reqwest::Client,
}

#[cfg(not(target_arch = "wasm32"))]
#[derive(Clone, Debug)]
pub struct Queue {
    stream: Arc<Mutex<TcpStream>>,
    functions: Arc<Mutex<VecDeque<String>>>,
}

use crate::vmix::functions::{VMixFunction, VMixSelectionTrait};

impl Queue {
    pub fn new(ip: String) -> Result<Self, MyError> {
        let me = Self {
            functions: Default::default(),
            stream: Mutex::new(Self::make_tcp_stream(&ip).ok_or(MyError::IpNotFound("Ip not found"))?).into(),
        };
        let funcs = me.functions.clone();
        let stream = me.stream.clone();
        // Here is the actual thread that clears the queue:
        std::thread::spawn(move || loop {
            if let Ok(mut functions) = funcs.lock() {
                while let Some(f) = functions.pop_front() {
                    Queue::send(&f.into_bytes(), stream.clone());
                }
            }
        });
        Ok(me)
    }

    fn make_tcp_stream(ip: &str) -> Option<TcpStream> {
        TcpStream::connect(SocketAddr::new(IpAddr::from_str(ip).unwrap(), 8099))
            .map_err(|err| {
                println!("TCP STREAM BUGGED OUT: {err}");
            })
            .ok()
    }

    fn send(bytes: &[u8], stream: Arc<Mutex<TcpStream>>) -> Result<(), String> {
        let mut stream = loop {
            if let Ok(stream) = stream.lock() {
                break stream;
            }
        };

        match stream.write_all(bytes) {
            Ok(()) => (),
            Err(e) => Err(e.to_string())?,
        }

        let mut response = Vec::new();
        let mut buff = [0; 256];
        loop {
            let number_of_bytes = match stream.read(&mut buff) {
                Ok(n) => n,
                Err(e) => Err(dbg!(e).to_string())?,
            };
            response.extend_from_slice(&buff[..number_of_bytes]);
            if response.ends_with(b"\r\n") {
                break;
            }
        }
        Self::parse_buffer(String::from_utf8(response).unwrap())
    }

    fn parse_buffer(buf: String) -> Result<(), String> {
        if buf.contains("OK") {
            Ok(())
        } else {
            Err(buf.split("ER ").collect::<Vec<_>>()[1].trim().to_string())
        }
    }

    pub fn add<T: VMixSelectionTrait>(&self, functions: &[VMixFunction<T>]) {
        loop {
            if let Ok(mut funcs) = self.functions.lock() {
                funcs.extend(
                    functions
                        .iter()
                        .map(VMixFunction::to_cmd)
                        .collect::<Vec<_>>(),
                );
                break;
            }
        }
    }
}

#[cfg(test)]
mod test {

    use super::*;
    use crate::flipup_vmix_controls::Score;
    use crate::vmix::functions::{VMixFunction, VMixProperty, VMixSelection};

    use crate::get_data::{
        DEFAULT_BACKGROUND_COL, DEFAULT_FOREGROUND_COL, DEFAULT_FOREGROUND_COL_ALPHA,
    };
    use rand::Rng;

    fn random_score_type(hole: usize) -> Score {
        let mut rng = rand::thread_rng();
        let throws = rng.gen_range(1..=6);
        Score::new(throws, 5, hole)
    }

    fn connect() -> Queue {
        Queue::new("10.170.120.134".to_string()).unwrap()
    }

    #[cfg(target_arch = "wasm32")]
    #[wasm_bindgen_test]
    async fn set_all_colours() {
        utils::set_panic_hook();

        let q = connect();
        let funcs = (0..=3)
            .flat_map(|player| {
                (1..=9).flat_map(move |hole| random_score_type(hole).update_score(player))
            })
            .collect::<Vec<VMixFunction<_>>>();
        q.add(&funcs);

        /*loop {
            let lock =  q.functions.lock().await;
            //log("hi1");
            if lock.len() == 0 {
                break;
            }
        }*/
        q.clear_queue().await;
    }

    #[cfg(target_arch = "wasm32")]
    #[wasm_bindgen_test]
    async fn clear_all_scores() {
        utils::set_panic_hook();
        let q = connect();
        for player in 0..=3 {
            for hole in 1..=9 {
                q.add(&[
                    VMixFunction::SetText {
                        input: VMixProperty::Score { player, hole }.into(),
                        value: "".to_string(),
                    },
                    VMixFunction::SetColor {
                        input: VMixProperty::ScoreColor { player, hole }.into(),
                        color: DEFAULT_FOREGROUND_COL,
                    },
                    VMixFunction::SetTextVisibleOff {
                        input: VMixProperty::Score { player, hole }.into(),
                    },
                ]);
            }
        }
        q.clear_queue().await;
    }
}
