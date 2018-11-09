use common::program::{Program, StepResult};
use common::error::{Error};
use std::io::{Read};
use std::fs::{File};
use clap::{Arg, App};
use crossbeam::{ channel };
use std::thread;

use tokio::prelude::*;
use futures::try_ready;
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

fn handle_io(_addr: Option<std::net::SocketAddr>)  -> (mpsc::UnboundedSender<u8>, channel::Receiver<u8>) {

    let (send_input, recv_input) = channel::unbounded::<u8>();
    let (send_output, recv_output) = mpsc::unbounded::<u8>();

    thread::spawn(move || {
        tokio::run(future::lazy(|| {

            // poll stdin and send any bytes we receive out to our
            // program. always return NotReady because we never want
            // the future to resolve.
            let stdin_future = futures::future::poll_fn(move || {
                let mut buf: [u8; 1] = [0; 1];
                loop {
                    // if n != 1, we should return with NotReady anyway:
                    let _ = try_ready!(tokio::io::stdin().poll_read(&mut buf).map_err(|_| ()));
                    send_input.send(buf[0]);
                }
            });

            // stream output from the program to stdout. Each byte streamed
            // successfully results in Async::Ready so that we can progress
            // to the next byte. Flush on every byte for immediate output.
            let stdout_future = recv_output.for_each(|b| {
                let mut stdout = tokio::io::stdout();
                futures::future::poll_fn(move || {
                    let _ = try_ready!(stdout.poll_write(&[b]).map_err(|_| ()));
                    let _ = try_ready!(stdout.poll_flush().map_err(|_| ()));
                    Ok(Async::Ready(()))
                })
            });

            // spawn our futures onto the threadpool to be executed:
            tokio::spawn(stdin_future);
            tokio::spawn(stdout_future);

            Ok(())

        }));
    });

    (send_output, recv_input)
}
