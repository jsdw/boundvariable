// For async/await lark:
#![feature(await_macro, async_await, futures_api)]

use common::program::{Program, StepResult};
use common::error::{Error};
use common::io_extra;
use common::broadcaster::Broadcaster;
use std::{ thread, io::Read, fs::File };
use clap::{Arg, App};
use crossbeam::{ channel };
use tokio::prelude::*;
use tokio::net::TcpListener;
use futures::sync::mpsc;

#[global_allocator]
static ALLOC: jemallocator::Jemalloc = jemallocator::Jemalloc;

fn main() -> Result<(), Error> {

    // Parse args and provide program help/info on load:
    let opts = App::new("interpreter")
        .version("0.2")
        .author("James Wilson <me@jsdw.me>")
        .about("UM interpreter for ICFP 06 Boundvariable coding challenge")
        .arg(Arg::with_name("address")
            .short("a")
            .long("address")
            .value_name("ADDRESS")
            .help("Provide an address to listen on to allow TCP connections to take hold of input/output"))
        .arg(Arg::with_name("FILE")
            .help("Set the UM/UMZ program to interpret")
            .required(true)
            .index(1))
        .get_matches();

    let filename = opts.value_of("FILE").unwrap();
    let address = if let Some(addr) = opts.value_of("address") {
        Some(addr.parse::<std::net::SocketAddr>()?)
    } else {
        None
    };

    // handle in/out via separate thread.
    let (output, input) = handle_io(address);

    // we block on each attempt to send to futures channel. Unblocking happens
    // when the byte is fully flushed to output. This ensures that this thread
    // won't finish until all output from the program is handed back, whereas
    // if we use unbounded channels the thread can finish before we've processed
    // and flushed all of the output.
    let mut output = output.wait();

    // Create new interpreter and read data into it:
    let mut program = Program::new();
    let mut file_data = vec![];
    let mut file = File::open(filename)?;
    file.read_to_end(&mut file_data)?;
    program.load_program(&file_data);

    // Run instructions and handle the result:
    loop {
        match program.step()? {
            StepResult::Halted => {
                break;
            },
            StepResult::Output{ ascii } => {
                output.send(ascii)?;
            },
            StepResult::InputNeeded{ inputter } => {
                let byte = input.recv()?;
                program.provide_input(inputter, Some(byte));
            },
            StepResult::Continue => {}
        }
    }

    Ok(())
}

/// This provides a way of sending to and receiving input to/from the interpreter. If
/// a socket address is provided, it will also allow an arbitrary number of network connections
/// to send/receive input. Allows things like `nc localhost 8080 > output` to save output:
fn handle_io(addr: Option<std::net::SocketAddr>)  -> (mpsc::Sender<u8>, channel::Receiver<u8>) {

    let (send_input, recv_input) = channel::unbounded::<u8>();
    let (send_output, mut recv_output) = mpsc::channel::<u8>(0);

    thread::spawn(move || {
        tokio::run_async(async move {

            // This guy sends off any input he receives to all interested parties:
            let mut broadcaster = Broadcaster::new();

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

        });
    });

    (send_output, recv_input)
}
