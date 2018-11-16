// For async/await lark:
#![feature(await_macro, async_await, futures_api)]

use common::program::{Program, StepResult};
use common::error::{Error};
use common::io::IoHandler;
use std::{ io::Read, fs::File };
use clap::{Arg, App};

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
    let io = IoHandler::start(address);

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
                io.send(ascii)?;
            },
            StepResult::InputNeeded{ inputter } => {
                let byte = io.recv()?;
                program.provide_input(inputter, Some(byte));
            },
            StepResult::Continue => {}
        }
    }


    // close out input and output channels now we won't
    // be using them, and wait for the io handler to signal
    // that it's done:
    io.block_until_closed();
    Ok(())
}