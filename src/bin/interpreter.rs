use common::program::{Program, StepResult};
use common::error::{Error};
use common::io_extra;
use std::io::{Read};
use std::fs::{File};
use std::thread;
use clap::{Arg, App};
use crossbeam::{ channel };
use tokio::prelude::*;
use tokio::net::TcpListener;
use futures::sync::mpsc;

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
                output.unbounded_send(ascii)?;
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

fn handle_io(addr: Option<std::net::SocketAddr>)  -> (mpsc::UnboundedSender<u8>, channel::Receiver<u8>) {

    let (send_input, recv_input) = channel::unbounded::<u8>();
    let (send_output, recv_output) = mpsc::unbounded::<u8>();

    thread::spawn(move || {
        tokio::run(future::lazy(move || {

            // if a network addy is provided, spin up a TCP listener to connect to
            // stdin and stdout from the program:
            if let Some(addr) = addr {

                let send_input = send_input.clone();
                let tcp_connections = TcpListener::bind(&addr)
                    .expect("listener cant bind to address")
                    .incoming()
                    .map_err(|e| eprintln!("accept failed: {:?}", e))
                    .for_each(move |sock| {

                        let (reader, writer) = sock.split();

                        // Stream input from the new TCP connection
                        // to our program:
                        let send_input = send_input.clone();
                        let input = io_extra::stream_bytes(reader)
                            .map_err(|e| {
                                eprintln!("Error reading from TCP socket: {:?}", e);
                            })
                            .for_each(move |byte| {
                                send_input.send(byte);
                                Ok(())
                            });

                        tokio::spawn(input);

                        // Subscribe this connection to receive output from
                        // the program
                        //
                        // @todo: actually do something with this:
                        let _output = io_extra::sink_bytes(writer)
                            .sink_map_err(|e| {
                                eprintln!("Error writing to TCP socker: {:?}", e);
                            });

                        Ok(())

                    });

                tokio::spawn(tcp_connections);

            }

            // stream bytes from stdin, complaining if there is an error:
            let stdin_future = io_extra::stream_bytes(tokio::io::stdin())
                .map_err(|e| {
                    eprintln!("Error reading from stdin: {:?}", e);
                })
                .for_each(move |byte| {
                    send_input.send(byte);
                    Ok(())
                })
                .and_then(|_| {
                    eprintln!("Stdin has been closed");
                    Ok(())
                });

            tokio::spawn(stdin_future);

            // stream output from the program to stdout. This flushes after
            // every byte to ensure output is seen:
            let stdout_sink = io_extra::sink_bytes(tokio::io::stdout())
                .sink_map_err(|e| {
                    eprintln!("Error writing to stdout: {:?}", e);
                });

            let stdout_future = recv_output
                .map_err(|e| {
                    eprintln!("Error receiving bytes from output chan: {:?}", e);
                })
                .forward(stdout_sink)
                .map(|_| ());

            tokio::spawn(stdout_future);

            Ok(())

        }));
    });

    (send_output, recv_input)
}
