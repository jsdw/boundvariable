use common::program::{Program, StepResult};
use common::error::{Error, err};
use std::io::{Read, Write, ErrorKind};
use std::fs::{File,OpenOptions};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

fn main() -> Result<(), Error> {

    // Install signal handling so that we can catch a signal and
    // toggle between sending output to stdout vs a file:
    let signal_received = Arc::new(AtomicBool::new(false));
    signal_hook::flag::register(signal_hook::SIGUSR1, Arc::clone(&signal_received))?;

    let mut program = Program::new();

    let args: Vec<String> = std::env::args().take(2).collect();

    let mut file_data = vec![];
    let filename = args.get(1).ok_or("Need to provide argument for program (.um or .umz) to read")?;
    let mut file = File::open(&filename)?;
    file.read_to_end(&mut file_data)?;

    program.load_program(&file_data);
    let mut send_to_stdout = true;
    let mut file: Option<File> = None;

    let mut interval_steps = 0;

    loop {

        // check our signal handler every now and then so that we can print
        // a message about it more promptly if it changes:
        if interval_steps == 10000 {
            check_redirect(&signal_received, &mut send_to_stdout);
            interval_steps = 0;
        }
        interval_steps += 1;

        // Run one instruction and handle the result:
        match program.step()? {
            StepResult::Halted => {
                break;
            },
            StepResult::Output{ ascii } => {
                // check our signal handler before every output to ensure we
                // put the output in the right place:
                check_redirect(&signal_received, &mut send_to_stdout);

                // send the output to either stdout or file based on our toggle:
                if send_to_stdout {
                    let mut stdout = std::io::stdout();
                    stdout.write(&[ascii])?;
                    stdout.flush()?;
                } else {
                    if file.is_none() {
                        file = Some(OpenOptions::new()
                            .create(true)
                            .append(true)
                            .open("boundvariable.out")?);
                    }
                    let handle = file.as_mut()?;
                    handle.write(&[ascii])?;
                    handle.flush()?;
                }
            },
            StepResult::InputNeeded{ inputter } => {
                let mut buf = [0; 1];
                match std::io::stdin().read_exact(&mut buf) {
                    Ok(_) => {
                        program.provide_input(inputter, Some(buf[0]));
                    },
                    Err(e) => {
                        // If we have run out of input, send None to the
                        // program to signal that input has finished..
                        if e.kind() != ErrorKind::UnexpectedEof {
                            return Err(err("stdin expected but not given"));
                        }
                        program.provide_input(inputter, None);
                    }
                }
            },
            StepResult::Continue => {}
        }
    }

    Ok(())
}

fn check_redirect(toggle: &Arc<AtomicBool>, state: &mut bool) {
    if toggle.swap(false, Ordering::Relaxed) {
        *state = !*state;
        if *state {
            eprintln!("<<Redirecting output to stdout>>");
        } else {
            eprintln!("<<Redirecting output to file>>");
        }
    }
}