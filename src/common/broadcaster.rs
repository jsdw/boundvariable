use std::{ mem, sync::{ Mutex, Arc } };
use tokio::prelude::*;
use futures::sync::mpsc;

#[derive(Clone)]
pub struct Broadcaster {
    subs: Arc<Mutex<Vec<BoxedSink>>>,
    sender: mpsc::Sender<u8>
}

/// The type of sink we can broadcast to:
type BoxedSink = Box<dyn Sink<SinkItem=u8, SinkError=()> + Send>;

/// This structure adds a convenient interface which you to
/// subscribe and send messages to the broadcaster:
impl Broadcaster {

    /// Create a new byte broadcaster (this will panic if it does not execute in the context
    /// of a tokio runtime). You can subscribe new Sinks and broadcast bytes to them. If a sink
    /// errors (eg it is no longer possible to send to it) it is no longer broadcasted to.
    ///
    /// @todo: this is really slow due to the mutex-per-byte and Sink API. We shouldn't need
    /// locks if we use a message passing API to subscribe and keep the map internal. Revisit
    /// at some point!
    pub fn new() -> Broadcaster {

        let outputters = Arc::new(Mutex::new(vec![]));
        let outputters2 = outputters.clone();
        let (send_broadcaster, recv_broadcaster) = mpsc::channel(0);

        tokio::spawn(future::lazy(move || {
            recv_broadcaster
                .map_err(|e| {
                    eprintln!("Error receiving msg to broadcast: {:?}", e);
                })
                .for_each(move |byte| {

                    // Swap outputters out of the shared reference and map into
                    // an iterator of send promises:
                    let this_outputters = mem::replace(&mut *outputters.lock().unwrap(), Vec::new());
                    let sending = this_outputters.into_iter().map(move |sink: BoxedSink| {
                        sink.send(byte).then(|res| {
                            match res {
                                // return the sink if it sent OK:
                                Ok(sink) => Ok(Some(sink)),
                                // throw the sink away if it errors:
                                Err(_) => Ok(None)
                            }
                        })
                    });

                    // resolve all of the send promises and then put the new
                    // sinks back into the outputters ref ready for next send:
                    let outputters2 = outputters.clone();
                    let sent = future::join_all(sending)
                        .and_then(move |sinks| {
                            *outputters2.lock().unwrap() = sinks.into_iter().filter_map(|s| s).collect();
                            Ok(())
                        });

                    sent

                })
        }));

        // return our interface:
        Broadcaster {
            subs: outputters2,
            sender: send_broadcaster
        }

    }

    /// Subscribe a new sink to receive output. It'll be dropped once it errors.
    pub fn subscribe(&self, sink: impl Sink<SinkItem=u8, SinkError=()> + Send + 'static) {
        self.subs.lock().unwrap().push(Box::new(sink))
    }
}

/// Broadcaster is also a valid Sink, to avoid needing to consume the inner sink
/// on every attempt to send a byte into it, and allow us to use `.forward` to
/// stream bytes into it.
impl Sink for Broadcaster {
    type SinkItem = u8;
    type SinkError = ();

    fn start_send(&mut self, byte: u8) -> Result<AsyncSink<u8>, Self::SinkError> {
        self.sender.start_send(byte).map_err(|_| ())
    }
    fn poll_complete(&mut self) -> Poll<(), Self::SinkError> {
        self.sender.poll_complete().map_err(|_| ())
    }
}
