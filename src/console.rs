use std::io::Error;
use std::net::UdpSocket;

use std::time::Duration;

const PAYLOAD_HEADER: &[u8] = b"\xff\xff\xff\xff";

pub struct Console {
    socket: UdpSocket,
    host: String,
    rcon_password: String,
}

impl Console {
    pub fn new(
        rcon_password: String,
        host_address: &str,
        host_port: u16,
        client_port: u16,
        read_timeout_duration: Duration,
    ) -> Self {
        let socket = UdpSocket::bind(format!("127.0.0.1:{}", client_port))
            .expect(&format!("cannot bind socket to port {}", client_port));
        socket
            .set_read_timeout(Some(read_timeout_duration))
            .expect("can't set read timeout");
        Self {
            socket: socket,
            host: format!("{}:{}", host_address, host_port),
            rcon_password: rcon_password,
        }
    }

    pub fn send(&mut self, payload: &[u8]) -> Result<Receiver, Error> {
        let payload = [PAYLOAD_HEADER, payload].concat();
        match self.socket.send_to(&payload, self.host.as_str()) {
            Ok(_) => Ok(self.receive()),
            Err(error) => Err(error),
        }
    }

    pub fn receive(&mut self) -> Receiver {
        Receiver {
            socket: &mut self.socket,
            buffer: [0; 1024],
        }
    }

    pub fn rcon_send(&mut self, payload: &[u8]) -> Result<Receiver, Error> {
        let payload = [b"rcon ", self.rcon_password.as_bytes(), b" ", payload].concat();
        self.send(&payload)
    }

    pub fn svsay(&mut self, payload: &[u8]) -> Result<Receiver, Error> {
        let payload = [b"svsay ", payload].concat();
        self.rcon_send(&payload)
    }

    pub fn svtell(&mut self, id: &[u8], payload: &[u8]) -> Result<Receiver, Error> {
        let payload = [b"svtell ", id, b" ", payload].concat();
        self.rcon_send(&payload)
    }

    pub fn map(&mut self, map: &[u8]) -> Result<Receiver, Error> {
        let payload = [b"map ", map].concat();
        self.rcon_send(&payload)
    }

    pub fn mbmode(&mut self, mode: &[u8]) -> Result<Receiver, Error> {
        let payload = [b"mbmode ", mode].concat();
        self.rcon_send(&payload)
    }
}

pub struct Receiver<'a> {
    socket: &'a mut UdpSocket,
    buffer: [u8; 1024],
}

impl<'a> Iterator for Receiver<'a> {
    type Item = String;
    fn next(&mut self) -> Option<String> {
        match self.socket.recv(&mut self.buffer) {
            Ok(byte_count) => Some(String::from_utf8_lossy(&self.buffer[..byte_count]).to_string()),
            Err(_) => None,
        }
    }
}
