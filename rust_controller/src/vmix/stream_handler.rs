use std::collections::VecDeque;

#[cfg(not(target_arch = "wasm32"))]
use {std::io::{Read, Write}, std::sync::Mutex, std::net::{IpAddr,TcpStream, SocketAddr, }};

use std::sync::Arc;
#[cfg(target_arch = "wasm32")]
use tokio::sync::Mutex;

use wasm_bindgen::prelude::*;


#[cfg(target_arch = "wasm32")]
pub struct Queue {
    #[cfg(target_arch = "wasm32")]
    functions: VecDeque<String>,
    #[cfg(target_arch = "wasm32")]
    ip: String,
    #[cfg(target_arch = "wasm32")]
    client: reqwest::Client,
}

#[cfg(not(target_arch = "wasm32"))]
pub struct Queue { 
    stream: Arc<Mutex<TcpStream>>,
    functions: Arc<Mutex<VecDeque<String>>>
}

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = globalThis)]
    fn setTimeout(closure: &Closure<dyn FnMut()>, millis: i32) -> i32;
}

// A function to simulate sleep
/*#[wasm_bindgen]
pub fn sleep(millis: i32) -> js_sys::Promise {
    js_sys::Promise::new(&mut |resolve, _| {
        let closure = Closure::wrap(Box::new(move || {
            resolve.call0(&JsValue::NULL).unwrap();
        }) as Box<dyn FnMut()>);
        setTimeout(&closure, millis);
        closure.forget(); // Prevents the closure from being cleaned up
    })
}

pub async fn sleep_rust(millis: i32) {
    JsFuture::from(sleep(millis)).await;
}*/

use crate::log;
use crate::vmix::functions::{VMixFunction, VMixSelectionTrait};
use futures::task::SpawnExt;
use std::str::FromStr;
use wasm_bindgen::prelude::wasm_bindgen;

impl Queue {
    #[cfg(not(target_arch = "wasm32"))]
    pub fn new(ip: String) -> Self {
        let me = Self {
            functions: Default::default(),
            stream: Arc::new(Mutex::new(Self::make_tcp_stream(&ip))),
        };
        me
    }
    #[cfg(target_arch = "wasm32")]
    pub fn new(ip: String) -> Self {
        let me = Self {
            functions: Default::default(),
            ip: ip.clone(),
            client: reqwest::Client::new(),
        };
        let funcs = me.functions.clone();
        let stream = me.stream.clone();
        #[cfg(not(target_arch = "wasm32"))]
        std::thread::spawn(move || Self::start_queue_thread(funcs,stream));        
        me
    }

    #[cfg(target_arch = "wasm32")]
    pub async fn clear_queue(&mut self) {
        while let Some(f) = self.functions.pop_front() {
            self.send(f).await.unwrap();
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn start_queue_thread(funcs: Arc<Mutex<VecDeque<String>>>, stream: Arc<Mutex<TcpStream>>) {
        loop {
            if let Ok(mut functions) = funcs.lock() {
                while let Some(f) = functions.pop_front() {
                    Queue::send(&f.into_bytes(), stream.clone());
                }
            }
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn make_tcp_stream(ip: &str) -> TcpStream {
        TcpStream::connect(SocketAddr::new(IpAddr::from_str(ip).unwrap(), 8099)).unwrap()
    }

    #[cfg(target_arch = "wasm32")]
    async fn send(&self, body: String) -> Result<(), String> {
        let response = self
            .client
            .post(format!("http://{}:8088/API/?{body}", self.ip))
            .send()
            .await
            .expect("failed to send request");
        response.text().await.expect("failed to parse response");
        Ok(())
    }

    #[cfg(not(target_arch = "wasm32"))]
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

    #[cfg(target_arch = "wasm32")]
    pub fn add<T: VMixSelectionTrait>(&mut self, functions: &[VMixFunction<T>])
    {
        self.functions.extend(
            functions
                .iter()
                .map(VMixFunction::to_cmd)
                .collect::<Vec<_>>(),
        )
    }
    #[cfg(not(target_arch = "wasm32"))]
    pub fn add<T: VMixSelectionTrait>(&mut self, functions: &[VMixFunction<T>]) {
        loop {
            if let Ok(mut funcs) = self.functions.lock() {
                funcs.extend(functions.iter().map(VMixFunction::to_cmd).collect::<Vec<_>>());
                break;
            }
        }
    }
    
}

#[cfg(test)]
mod test {
    use wasm_bindgen_test::wasm_bindgen_test;

    use super::*;
    use crate::utils;
    use crate::vmix::conversions::{BogeyType, ReadableScore, Score};
    use crate::vmix::functions::{VMixFunction, VMixProperty, VMixSelection};

    use rand::Rng;

    fn random_score_type(hole: usize) -> Score {
        let mut rng = rand::thread_rng();
        let throws = rng.gen_range(1..=6);
        Score::new(throws, 5, hole)
    }

    fn connect() -> Queue {
        Queue::new("10.170.120.134".to_string())
    }

    #[cfg(target_arch = "wasm32")]
    #[wasm_bindgen_test]
    async fn set_all_colours() {
        utils::set_panic_hook();

        let mut q = connect();
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
}
