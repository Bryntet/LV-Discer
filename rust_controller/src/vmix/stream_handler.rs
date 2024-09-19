use tokio::sync::broadcast::{channel, Receiver, Sender};

#[cfg(not(target_arch = "wasm32"))]
use {
    std::io::{Read, Write},
    std::net::{IpAddr, SocketAddr, TcpStream},
    std::str::FromStr,
};

use crate::api::Error;
use crate::vmix::functions::{VMixInterfacer, VMixSelectionTrait};

#[cfg(target_arch = "wasm32")]
#[derive(Clone)]
pub struct Queue {
    functions: Arc<Mutex<VecDeque<String>>>,
    ip: String,
    client: reqwest::Client,
}

#[cfg(not(target_arch = "wasm32"))]
#[derive(Clone, Debug)]
pub struct VMixQueue {
    functions_sender: tokio::sync::broadcast::Sender<String>,
}

impl VMixQueue {
    pub fn new(ip: String) -> Result<Self, Error> {
        let (tx, mut rx): (Sender<String>, Receiver<String>) = channel(2048);
        let mut stream = Self::make_tcp_stream(&ip).ok_or(Error::IpNotFound(ip))?;

        let me = Self {
            functions_sender: tx,
        };
        // Here is the actual thread that clears the queue:
        tokio::spawn(async move {
            loop {
                if let Ok(f) = rx.recv().await {
                    dbg!(&f);
                    match VMixQueue::send(&f.into_bytes(), &mut stream) {
                        Ok(()) => (),
                        Err(e) => {
                            warn!("{}", e);
                        }
                    };
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

    fn send(bytes: &[u8], stream: &mut TcpStream) -> Result<(), String> {
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

    pub fn add_ref<'a, T: VMixSelectionTrait + 'a>(
        &self,
        functions: impl Iterator<Item = &'a VMixInterfacer<T>>,
    ) {
        for func in functions {
            match self.functions_sender.send(func.to_cmd()) {
                Ok(_) => (),
                Err(e) => {
                    warn!("Failed to send command to queue: {e}");
                }
            };
        }
    }

    pub fn add<T: VMixSelectionTrait>(&self, functions: impl Iterator<Item = VMixInterfacer<T>>) {
        for func in functions {
            match self.functions_sender.send(func.to_cmd()) {
                Ok(_) => (),
                Err(e) => {
                    warn!("Failed to send command to queue: {e}");
                }
            };
        }
    }
}
