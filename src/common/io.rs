use crate::error::{Error};
use crate::io_extra;
use crate::broadcaster::Broadcaster;
use std::{ thread };
use crossbeam::{ channel };
use tokio::prelude::*;
use tokio::net::TcpListener;
use futures::sync::mpsc;

/// This provides a way of sending to and receiving input to/from the interpreter. If
/// a socket address is provided, it will also allow an arbitrary number of network connections
/// to send/receive input. Allows things like `nc localhost 8080 > output` to save output:
pub struct IoHandler {
    sender: mpsc::UnboundedSender<u8>,
    receiver: channel::Receiver<u8>,
    on_closed: channel::Receiver<()>
}

impl IoHandler {

    pub fn start(addr: Option<std::net::SocketAddr>) -> IoHandler {

        let (finished_input, finished_output) = channel::bounded::<()>(0);
        let (send_input, recv_input) = channel::unbounded::<u8>();
        let (send_output, mut recv_output) = mpsc::unbounded::<u8>();

        thread::spawn(move || {

            let mut rt = tokio::runtime::Runtime::new().unwrap();
            block_on_async(&mut rt, async move {

                // This guy sends off any input he receives to all interested parties:
                let (mut broadcaster, mut broadcaster_done) = Broadcaster::new();

                // if a network addy is provided, spin up a TCP listener to connect to
                // stdin and stdout from the program:
                if let Some(addr) = addr {
                    let broadcaster = broadcaster.clone();
                    let send_input = send_input.clone();
                    tokio::spawn_async(async move {

                        let mut tcp_connections = TcpListener::bind(&addr)
                            .expect("listener cant bind to address")
                            .incoming();

                        while let Some(sock) = await!(tcp_connections.next()) {

                            let sock = match sock {
                                Err(e) => {
                                    eprintln!("Error opening socket: {:?}", e);
                                    continue;
                                },
                                Ok(s) => s
                            };

                            let (reader, writer) = sock.split();
                            let send_input = send_input.clone();

                            // listen for input and send to the main thread:
                            tokio::spawn_async(async move {
                                let mut input = io_extra::stream_bytes(reader);
                                while let Some(byte) = await!(input.next()) {
                                    if let Ok(byte) = byte {
                                        send_input.send(byte);
                                    }
                                }
                            });

                            // subscribe to output:
                            let mut broadcaster = broadcaster.clone();
                            tokio::spawn_async(async move {
                                let output = io_extra::sink_bytes(writer).sink_map_err(|_| ());
                                await!(broadcaster.subscribe(output));
                            })

                        }
                    });
                }

                // Stream input from stdin to the program:
                tokio::spawn_async(async move {
                    let mut stdin_future = io_extra::stream_bytes(tokio::io::stdin());
                    while let Some(Ok(byte)) = await!(stdin_future.next()) {
                        send_input.send(byte);
                    }
                });

                // Stream output from stdout to our broadcaster, once we've subscribed to it:
                let stdout_sink = io_extra::sink_bytes(tokio::io::stdout()).sink_map_err(|_| ());
                await!(broadcaster.subscribe(stdout_sink));
                while let Some(Ok(byte)) = await!(recv_output.next()) {
                    if let Err(e) = await!(broadcaster.send_async(byte)) {
                        eprintln!("Error sending byte to outputters: {:?}", e);
                    }
                }

                // Tell the broadcaster it won't be receiving any more,
                // and then wait for it to signal that it's finished
                await!(broadcaster.close());
                await!(broadcaster_done.next());

            });

            // Tokio has finished; send "done":
            finished_input.send(());
        });

        // Return a struct which provides access to these things:
        IoHandler {
            sender: send_output,
            receiver: recv_input,
            on_closed: finished_output
        }

    }

    /// Send a byte to the output(s):
    pub fn send(&self, byte: u8) -> Result<(), Error> {
        self.sender.unbounded_send(byte)?;
        Ok(())
    }

    /// Block until we receive a byte from an input source:
    pub fn recv(&self) -> Result<u8, Error> {
        let byte = self.receiver.recv()?;
        Ok(byte)
    }

    /// Shutdown, and block until all output has been flushed:
    pub fn block_until_closed(self) -> () {
        drop(self.sender);
        drop(self.receiver);
        let _ = self.on_closed.recv();
    }

}

// A shim borrowed from how run_async is implemented to allow us to
// tell a reactor to run only until its async block resolved, not
// worrying about spawned things:
fn block_on_async<F>(runtime: &mut tokio::runtime::Runtime, future: F) where F: std::future::Future<Output = ()> + Send + 'static {
    use tokio_async_await::compat::backward;
    let future = backward::Compat::new(async move {
        await!(future);
        let r: Result<(),()> = Ok(());
        r
    });
    let _ = runtime.block_on(future);
}