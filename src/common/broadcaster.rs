use std::{ mem, sync::{ Mutex, Arc } };
use tokio::prelude::*;
use futures::sync::mpsc;

#[derive(Clone)]
pub struct Broadcaster {
    sender: mpsc::Sender<Msg>
}

/// This structure adds a convenient interface which you to
/// subscribe and send messages to the broadcaster:
impl Broadcaster {
    pub fn subscribe(&self, sink: Box<dyn Sink<SinkItem=u8, SinkError=()> + Send>) -> impl Future<Item=Broadcaster, Error=()> {
        // Clone to avoid giving sender away; subscriptions are not a hot path:
        self.sender.clone().send(Msg::Subscribe(sink))
            .and_then(|s| Ok(Broadcaster{ sender: s }))
            .map_err(|_| ())
    }
}

/// Broadcaster is also a valid Sink, to avoid needing to consume the inner sink
/// on every attempt to send a byte into it, and allow us to use `.forward` to
/// stream bytes into it.
impl Sink for Broadcaster {
    type SinkItem = u8;
    type SinkError = ();

    fn start_send(&mut self, byte: u8) -> Result<AsyncSink<u8>, Self::SinkError> {
        match self.sender.start_send(Msg::Broadcast(byte)) {
            Err(_) => Err(()),
            Ok(inner) => Ok(inner.map(|_| byte))
        }
    }
    fn poll_complete(&mut self) -> Poll<(), Self::SinkError> {
        self.sender
            .poll_complete()
            .map_err(|_| ())
    }
}

/// The types of message that can be sent to the broadcaster:
enum Msg {
    Subscribe(Box<dyn Sink<SinkItem=u8, SinkError=()> + Send>),
    Broadcast(u8)
}

/// Create a new byte broadcaster (this will panic if it does not execute in the context
/// of a tokio runtime). You can subscribe new Sinks and broadcast bytes to them. If a sink
/// errors (eg it is no longer possible to send to it) it is no longer broadcasted to.
pub fn new() -> Broadcaster {

    let (send_broadcaster, recv_broadcaster) = mpsc::channel(0);
    tokio::spawn(future::lazy(move || {

        // This should not really need to lock, since it is only ever accessed
        // once at a time, but to satisfy the type system for now we wrap it up:
        let outputters = Arc::new(Mutex::new(vec![]));

        recv_broadcaster
            .map_err(|e| {
                eprintln!("Error receiving msg to broadcast: {:?}", e);
            })
            .for_each(move |input| {
                match input {
                    Msg::Subscribe(sink) => {

                        // Subscribe a new sink to receive output:
                        outputters.lock().unwrap().push(sink);
                        future::Either::A(future::ok(()))

                    },
                    Msg::Broadcast(byte) => {

                        // Swap outputters out of the shared reference and map into
                        // an iterator of send promises:
                        let this_outputters = mem::replace(&mut *outputters.lock().unwrap(), Vec::new());
                        let sending = this_outputters.into_iter().map(move |sink| {
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

                        future::Either::B(sent)
                    }
                }
            })
    }));

    // return our interface:
    Broadcaster {
        sender: send_broadcaster
    }

}
