use crate::vmix::functions::{LeaderBoardProperty, VMixFunction, VMixSelectionTrait};
use std::collections::VecDeque;
use std::io::{Read, Write};
use std::net::{IpAddr, Ipv4Addr, SocketAddr, TcpStream};
use std::str::FromStr;
use std::sync::{Arc, Mutex};
use std::time::Duration;


pub struct Queue {
    functions: Arc<Mutex<VecDeque<String>>>,
    stream: Arc<Mutex<TcpStream>>,
}

impl Queue {
    pub fn new(ip: String) -> Self {
        let mut stream = TcpStream::connect(SocketAddr::new(IpAddr::V4(Ipv4Addr::from_str(&ip).unwrap()), 8099)).unwrap();
        /*stream
            .set_read_timeout(Some(Duration::from_millis(10)))
            .expect("unable to set read timeout");*/
        let mut buff = String::new();
        loop {
            stream.read_to_string(&mut buff).ok();
            if buff.contains("\r\n") {
                break;
            }
        }
        let thing = format!("Connected to VMix TCP API. Received response:\n{buff}");
        //super::super::log(&thing);
        println!("{thing}");

        let me = Self {
            functions: Mutex::new(VecDeque::from(vec![])).into(),
            stream: Mutex::new(stream).into(),
        };
        let stream = me.stream.clone();
        let funcs = me.functions.clone();
        std::thread::spawn(move || Self::start_queue_thread(funcs, stream));
        me
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

    fn start_queue_thread(funcs: Arc<Mutex<VecDeque<String>>>, stream: Arc<Mutex<TcpStream>>) {
        
        
        loop {
            if let Ok(mut functions) = funcs.lock() {
                while let Some(f) = functions.pop_front() {
                    Queue::send(&f.into_bytes(), stream.clone());
                }
            }
        }
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
        let funcs = self.functions.clone();

        let mut funcs = loop {
            if let Ok(funcs) = funcs.lock() {
                break funcs;
            }
        };
        funcs.extend(functions.iter().map(|f| f.to_cmd()).collect::<Vec<_>>())
    }
}

#[cfg(test)]
mod test {
    use crate::vmix::conversions::ReadableScore;
    use crate::vmix::functions::{VMixFunction, VMixProperty, VMixSelection};
    use crate::vmix::stream_handler::Queue;
    use std::time::Duration;

    fn connect() -> Queue {
        Queue::new("10.170.120.134".to_string())
    }

    #[test]
    fn set_all_colours() {
        let mut q = connect();
        let time = std::time::Instant::now();
        let funcs = (0..=4)
            .flat_map(|player| {
                (1..=9).map(move |hole| VMixFunction::SetColor {
                    color: ReadableScore::Ace.to_colour(),
                    input: VMixSelection(VMixProperty::ScoreColor { hole, player }),
                })
            })
            .collect::<Vec<_>>();
        q.add(&funcs);
        dbg!(std::time::Instant::now().duration_since(time));
        std::thread::sleep(Duration::from_millis(1000))
    }
}
