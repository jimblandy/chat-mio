extern crate mio;

use mio::{Events, Poll, PollOpt, Ready, Token};
use mio::tcp::{TcpListener, TcpStream};
use mio::unix::EventedFd;
use std::io::{Read, Write};
use std::net::{SocketAddr, ToSocketAddrs};

const USAGE: &'static str = "Usage: chat-mio [--serve] HOST:PORT";

const TTY:        Token = Token(0);
const LISTENER:   Token = Token(1);
const CONNECTION: Token = Token(2);

fn to_addr(addr: &str) -> SocketAddr {
    addr.to_socket_addrs()
        .expect("couldn't resolve network address")
        .next().expect("network address didn't resolve to any IPs")
}

fn main() {
    let mut args = std::env::args().skip(1);

    let poll = Poll::new().expect("couldn't create mio::Poll");

    let mut listener = None;
    let mut connection = None;

    let arg = args.next().expect(USAGE);
    if arg == "--serve" {
        let addr = args.next().expect(USAGE);
        listener = Some(TcpListener::bind(&to_addr(&addr[..]))
                        .expect("failed to listen"));
        poll.register(listener.as_ref().unwrap(), LISTENER, Ready::readable(), PollOpt::level())
            .expect("failed to register network listener with mio poll");
    } else {
        connection = Some(TcpStream::connect(&to_addr(&arg[..]))
                          .expect("failed to connect to"));
        poll.register(connection.as_ref().unwrap(), CONNECTION,
                      Ready::readable(),
                      PollOpt::level())
            .expect("failed to register outbound connection with mio poll");
    }

    poll.register(&EventedFd(&0), TTY, Ready::readable(), PollOpt::level())
        .expect("failed to register stdin");

    let mut stdin = std::io::stdin();
    let mut stdout = std::io::stdout();
    let mut tty_eof = false;

    let mut events = Events::with_capacity(8);
    while !tty_eof && (listener.is_some() || connection.is_some()) {
        poll.poll(&mut events, None).expect("mio poll call failed");

        for event in &events {
            match event.token() {
                LISTENER => {
                    let listener = listener.take().unwrap();
                    let (stream, peer) = listener.accept().expect("accepting incoming connection");
                    poll.deregister(&listener).expect("deregistering listening socket");
                    println!("Accepting connection from {:?}\x07", peer);
                    poll.register(&stream, CONNECTION, Ready::readable(), PollOpt::level())
                        .expect("failed to register inbound connection with mio poll");
                    connection = Some(stream);
                }
                TTY => {
                    // stdin isn't necessary in non-blocking mode, so just do a
                    // single read.
                    let mut buf = [0_u8; 512];
                    let len = stdin.read(&mut buf).expect("error reading from stdin");
                    if len == 0 {
                        println!("eof on stdin");
                        tty_eof = true;
                    } else {
                        match connection {
                            Some(ref mut connection) => {
                                connection.write_all(&buf[0..len])
                                    .expect("error writing to connection");
                            }
                            None => {
                                println!("no connection yet, input dropped");
                            }
                        }
                    }
                }
                CONNECTION => {
                    let mut buf = Vec::new();
                    match connection.as_mut().unwrap().read_to_end(&mut buf) {
                        Ok(0) => {
                            println!("got eof");
                            connection = None;
                        }
                        Ok(_) => (),
                        Err(e) => {
                            if e.kind() != std::io::ErrorKind::WouldBlock {
                                panic!("error reading from connection: {:?}", e);
                            }
                        }
                    }

                    print!("\x07");
                    stdout.write_all(&buf)
                        .and_then(|()| stdout.flush())
                        .expect("failed to write received string");
                }
                Token(_) => panic!("unexpected token")
            }
        }
    }
}
