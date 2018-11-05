use common::program::{Program, StepResult};
use common::error::{Error};
use std::io::{Read, Write};
use std::fs::{File};
use clap::{Arg, App};
use crossbeam::{ channel };
use std::thread;

fn main() -> Result<(), Error> {

    // Parse args and provide program help/info on load:
    let opts = App::new("interpreter")
        .version("0.2")
        .author("James Wilson <me@jsdw.me>")
        .about("UM interpreter for ICFP 06 Boundvariable coding challenge")
        .arg(Arg::with_name("port")
            .short("p")
            .long("port")
            .value_name("PORT")
            .help("Provide a port to allow TCP connections to take hold of input/output"))
        .arg(Arg::with_name("FILE")
            .help("Set the UM/UMZ program to interpret")
            .required(true)
            .index(1))
        .get_matches();

    let filename = opts.value_of("FILE").unwrap();

    // If a port was provided, we'll allow TCP connections
    // to provide input/output:
    let port: Option<u16> = opts
        .value_of("port")
        .and_then(|p| p.parse().ok());

    // Spin up channels to get input/output from.
    let (_in, input) = channel::unbounded();
    let (output, _out) = channel::unbounded();

    // handle in/out via separate thread.
    thread::spawn(move || handle_io(_in, _out, port));

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
                output.send(ascii);
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

fn handle_io(input: channel::Sender<u8>, output: channel::Receiver<u8>, _tcp_port: Option<u16>) {

    // Reading from stdin:
    thread::spawn(move || {
        let mut stdin = std::io::stdin();
        loop {
            let mut buf = [0;1];
            if let Ok(()) = stdin.read_exact(&mut buf) {
                input.send(buf[0]);
            }
        }
    });

    // Writing to stdout:
    loop {
        if let Some(byte) = output.recv() {
            let mut stdout = std::io::stdout();
            let _ = stdout.write(&[byte]);
            let _ = stdout.flush();
        }
    }
}