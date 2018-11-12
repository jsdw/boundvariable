use std::{ mem, sync::{ Mutex, Arc } };
use tokio::prelude::*;
use futures::sync::mpsc;

#[derive(Clone)]
pub struct Broadcaster {
    sender: mpsc::Sender<Msg>
}

enum Msg {
    Subscribe(BoxedSink),
    Broadcast(u8)
}

type BoxedSink = Box<dyn Sink<SinkItem=u8, SinkError=()> + Send + Sync>;

/// This structure adds a convenient interface which you to
/// subscribe and send messages to the broadcaster:
impl Broadcaster {
    pub fn new() -> Broadcaster {
        make_broadcaster()
    }

    pub fn subscribe(&self, sink: impl Sink<SinkItem=u8, SinkError=()> + Send + Sync + 'static) -> impl Future<Item=(), Error=()> {
        // Clone to avoid giving sender away; subscriptions are not a hot path:
        self.sender.clone().send(Msg::Subscribe(Box::new(sink)))
            .and_then(|s| Ok(Broadcaster{ sender: s }))
            .map_err(|_| ())
            .map(|_| ())
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

/// Create a new byte broadcaster (this will panic if it does not execute in the context
/// of a tokio runtime). You can subscribe new Sinks and broadcast bytes to them. If a sink
/// errors (eg it is no longer possible to send to it) it is no longer broadcasted to.
fn make_broadcaster() -> Broadcaster {

    let (send_broadcaster, recv_broadcaster) = mpsc::channel(0);

    tokio::spawn(future::lazy(move || {
        let mut outputters = Arc::new(vec![]);
        recv_broadcaster
            .map_err(|e| {
                eprintln!("Error receiving msg to broadcast: {:?}", e);
            })
            .for_each(move |input| {
                match input {
                    Msg::Subscribe(sink) => {

                        // Subscribe a new sink to receive output:
                        Arc::get_mut(&mut outputters).expect("Sole access 1").push(sink);
                        future::Either::A(future::ok(()))

                    },
                    Msg::Broadcast(byte) => {

                        // Swap outputters out of the shared reference and map into
                        // an iterator of send promises:
                        let this_outputters = mem::replace(Arc::get_mut(&mut outputters).expect("Sole access 2"), Vec::new());
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
                        let mut outputters2 = outputters.clone();
                        let sent = future::join_all(sending)
                            .and_then(move |sinks| {

                                // In order to swap values back into the vec after close, we are forced to clone the Arc
                                // To use it here. However, that stops get_mut from working and so we hit this error.
                                // trying current_runtime would help but tokio-fs (stdin/stdout) does not seem to play well
                                // with it. async/await may help by allowing a borrow to be preserved and getting rid of closure.
                                // While I know that access to outputters is sequential I can't prove it to the compiler at the moment :(.
                                *Arc::get_mut(&mut outputters2).expect("Sole access 3") = sinks.into_iter().filter_map(|s| s).collect();
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
