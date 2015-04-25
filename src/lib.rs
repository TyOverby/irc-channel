extern crate irc_message;

use std::io::Result as IoResult;
use std::io::{LineWriter, Write, BufStream, BufRead};
use std::thread;
use std::net::{ToSocketAddrs, TcpStream};
use std::sync::mpsc::{channel, Receiver};
use std::hash::Hash;

use irc_message::IrcMessage;

pub struct Sender {
    stream: LineWriter<TcpStream>
}

impl Sender {
    pub fn send<T: Hash + Eq + AsRef<str>>(&mut self, message: &IrcMessage<T>) -> IoResult<()> {
        let string = format!("{}\n", message);
        self.stream.write_all(string.as_bytes())
    }

    /// Kills both the reading and writing portions of the channel.
    pub fn kill_channel(mut self) {
        use std::net::Shutdown;
        let _ = self.stream.get_mut().shutdown(Shutdown::Read);
    }
}

impl Drop for Sender {
    fn drop(&mut self) {
        use std::net::Shutdown;
        let _ = self.stream.get_mut().shutdown(Shutdown::Write);
    }
}

pub fn irc_channel<A: ToSocketAddrs>(address: A, auto_pong: bool)
-> IoResult<(Sender, Receiver<IrcMessage<String>>)> {
    let conn1 = try!(TcpStream::connect(address));
    let conn2 = try!(conn1.try_clone());
    let conn3 = try!(conn1.try_clone());

    let (sx, rx) = channel();

    thread::spawn(move || {
        let buf_stream = BufStream::new(conn2);
        let mut out_buf_stream = BufStream::new(conn3);
        for line in buf_stream.lines() {
            match line {
                Ok(line) => {
                    if let Some(mut parsed) = IrcMessage::parse_own(&line) {
                        if auto_pong
                           && parsed.command.is_some()
                           && parsed.command.as_ref().unwrap() == "PING" {
                            parsed.command = Some("PONG".to_string());
                            let string = format!("{}\n", parsed);
                            let _ = out_buf_stream.write_all(string.as_bytes());
                            continue;
                        }

                        if let Err(_) = sx.send(parsed) {
                            break;
                        }
                    }
                }
                Err(_) => break
            }
        }
    });

    Ok((Sender {
        stream: LineWriter::new(conn1)
    }, rx))
}

