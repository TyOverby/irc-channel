#![feature(std_misc)]

extern crate irc_channel;
extern crate irc_message;
use irc_channel::irc_channel;
use irc_message::IrcMessage;

use std::io::stdin;
use std::sync::mpsc::{channel, Receiver};
use std::thread;

fn stdio_channel() -> Receiver<String> {
    use std::io::BufRead;

    let (sx, rx) = channel();
    thread::spawn(move || {
        let i = stdin();
        let lock = i.lock();
        for line in lock.lines() {
            match line {
                Ok(line) => {
                    if let Err(_) = sx.send(line) {
                        break;
                    }
                }
                Err(_) => break
            }
        }
    });

    rx
}

fn main() {
    let (mut irc_send, irc_recv) = irc_channel("irc.mozilla.org:6666", true).unwrap();
    let stdin_recv = stdio_channel();

    loop {
        select!{
            message = irc_recv.recv() => {
                if let Ok(m) = message {
                    println!("{}", m.raw);
                } else {
                    break;
                }
            },

            line = stdin_recv.recv() => {
                if let Ok(line) = line {
                    if let Some(parsed) = IrcMessage::parse_ref(&line) {
                        irc_send.send(&parsed);
                    } else {
                        println!("COULD NOT PARSE");
                    }
                }
            }
        }
    }
}
