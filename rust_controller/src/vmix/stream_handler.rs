use std::collections::VecDeque;
use std::io::{Read, Write};
use std::net::{IpAddr, SocketAddr, TcpStream};
use std::sync::{Arc};
use tokio::sync::Mutex;
use wasm_bindgen::prelude::*;

pub struct Queue {
    functions: Arc<Mutex<VecDeque<String>>>,
    #[cfg(not(target_arch = "wasm32"))]
    stream: Arc<Mutex<TcpStream>>,
    #[cfg(target_arch = "wasm32")]
    ip: String
}

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = globalThis)]
    fn setTimeout(closure: &Closure<dyn FnMut()>, millis: i32) -> i32;
}


// A function to simulate sleep
#[wasm_bindgen]
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
}

use wasm_bindgen_futures::{JsFuture, spawn_local};
use std::str::FromStr;
use cynic::GraphQlResponse;
use wasm_bindgen::JsValue;
use wasm_bindgen::prelude::wasm_bindgen;
use crate::{log, queries};
use crate::vmix::functions::{VMixFunction, VMixSelectionTrait};


impl Queue {
    pub fn new(ip: String) -> Self {
        let me = Self {
            functions: Default::default(),
            #[cfg(not(target_arch = "wasm32"))]
            stream: Arc::new(Mutex::new(Self::make_tcp_stream(&ip))),
            #[cfg(target_arch = "wasm32")]
            ip: ip.clone(),
        };

        #[cfg(not(target_arch = "wasm32"))]
            let stream = me.stream.clone();
        let funcs = me.functions.clone();


        #[cfg(not(target_arch = "wasm32"))]
        std::thread::spawn(move || Self::start_queue_thread(funcs,stream));

        #[cfg(target_arch = "wasm32")]
        spawn_local(async move {
            log("hello from local spawned");
            loop {
                Self::clear_queue(funcs.clone(),&ip).await;
            }});
        me
    }
    #[cfg(target_arch = "wasm32")]
    async fn clear_queue(funcs: Arc<Mutex<VecDeque<String>>>, ip: &str) {
        let mut functions = funcs.lock().await;
        while let Some(f) = functions.pop_front() {
            log("cleared one");
            Queue::send(f, ip).await.unwrap();
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
    async fn send(body: String, ip: &str) -> Result<(), String> {
        log("send");
        log(&format!("{body}"));
        let response = reqwest::Client::new()
            .post(format!("http://{ip}:8088/API/?{body}"))
            .send()
            .await
            .expect("failed to send request");
        let res = response
            .text()
            .await
            .expect("failed to parse response");
        //log(&res);
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

    pub fn add<T>(&self, functions: &[VMixFunction<T>])
        where
            T: VMixSelectionTrait + std::marker::Send + 'static + std::marker::Sync,
    {
        log("add");
        let funcs = self.functions.clone();
        let mut funcs = funcs.blocking_lock();
        funcs.extend(functions.iter().map(|f| {
            
            let cmd = f.to_cmd();
            log(&cmd);
            cmd
        }).collect::<Vec<_>>())
    }
}



#[cfg(test)]
#[cfg(target_arch = "wasm32")]
mod test {
    use wasm_bindgen::{JsValue, UnwrapThrowExt};
    use wasm_bindgen_test::wasm_bindgen_test;

    
    
    use crate::vmix::conversions::{BogeyType, ReadableScore};
    use crate::vmix::functions::{VMixFunction, VMixProperty, VMixSelection};
    use super::*;
    use std::time::Duration;
    use wasm_bindgen::__rt::Start;
    use crate::utils;

    use rand::Rng;

    fn random_number() -> ReadableScore {
        let mut rng = rand::thread_rng();
        let num = rng.gen_range(0..=6);
        match num  {
            0 => ReadableScore::Ace,
            1 => ReadableScore::Albatross,
            2 => ReadableScore::Birdie,
            3 => ReadableScore::Eagle,
            4 => ReadableScore::Par,
            _ => ReadableScore::Bogey(BogeyType::Ouch)
        }
    }

    fn connect() -> Queue {
        Queue::new("10.170.120.134".to_string())
    }

    #[wasm_bindgen_test]
    async fn set_all_colours() {
        utils::set_panic_hook();

        let mut q = connect();
        //let time = std::time::Instant::now();
        let funcs = (0..=3)
            .flat_map(|player| {
                (1..=9).map(move |hole| VMixFunction::SetColor {
                    color: ReadableScore::Birdie.to_colour(),
                    input: VMixSelection(VMixProperty::ScoreColor { hole, player }),
                })
            })
            .collect::<Vec<_>>();
        q.add(&funcs);
        //Queue::clear_queue(q.functions.clone(),&q.ip).await;
        //dbg!(std::time::Instant::now().duration_since(time));


        log("hi");
        dbg!("hi");
        /*loop {
            let lock =  q.functions.lock().await;
            //log("hi1");
            if lock.len() == 0 {
                break;
            }
        }*/
    }
}
