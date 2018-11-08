use common::program::{Program, StepResult};
use common::error::{Error};
use std::io::{Read, Write};
use std::fs::{File};
use clap::{Arg, App};
use crossbeam::{ channel };
use std::thread;
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
                output.unbounded_send(ascii);
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

    use tokio::runtime::current_thread::Runtime;
    use tokio::{ io, io::{ AsyncWrite, AsyncRead } };
    use futures::future::Future;
    use futures::stream::Stream;
    use futures::sync::mpsc;

    let (send_input, recv_input) = channel::unbounded::<u8>();
    let (send_output, recv_output) = mpsc::unbounded::<u8>();

    thread::spawn(move || {

        let mut runtime = Runtime::new().unwrap();
        let handle = runtime.handle();

        // Create a stream that plucks bytes from stdin:
        let stdin_bytes = futures::stream::poll_fn(move || {
            let mut buf: [u8; 1] = [0; 1];
            let res = io::stdin().poll_read(&mut buf);
            res.map(|res| res.map(|n| Some(buf[0]))).map_err(|_| ())
        });

        // Iterate over that stream, sending bytes out to the program:
        let stdin_stream = stdin_bytes.for_each(move |b| {
            let _ = send_input.send(b);
            Ok(())
        });

        // Put bytes received into stdout:
        let stdout_stream = recv_output.for_each(|b| {
            let mut stdout = io::stdout();
            futures::future::poll_fn(move || {
                let res = stdout.poll_write(&[b]); println!("RES: {:?}", res);
                res.map(|res| res.map(|n| ())).map_err(|_| ())
            })
        });

        handle.spawn(stdin_stream);
        handle.spawn(stdout_stream);
        runtime.run().expect("failed to run tokio runtime");

    });


    // thread::spawn(move || {
    //     // Reading from stdin:
    //     thread::spawn(move || {
    //         let mut stdin = std::io::stdin();
    //         loop {
    //             let mut buf = [0;1];
    //             if let Ok(()) = stdin.read_exact(&mut buf) {
    //                 send_input.send(buf[0]);
    //             }
    //         }
    //     });
    //     // Writing to stdout:
    //     loop {
    //         if let Some(byte) = recv_output.recv() {
    //             let mut stdout = std::io::stdout();
    //             let _ = stdout.write(&[byte]);
    //             let _ = stdout.flush();
    //         }
    //     }
    // });

    (send_output, recv_input)
}