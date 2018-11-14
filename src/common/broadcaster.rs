use tokio::prelude::*;
use futures::sync::mpsc;

#[derive(Clone)]
pub struct Broadcaster {
    sender: mpsc::Sender<Msg>
}

enum Msg {
    Subscribe(Box<dyn Sink<SinkItem=u8, SinkError=()> + Send + Sync + 'static>),
    Broadcast(u8)
}

/// This structure adds a convenient interface which you to
/// subscribe and send messages to the broadcaster:
impl Broadcaster {
    pub fn new() -> Broadcaster {
        make_broadcaster()
    }

    pub async fn subscribe(&mut self, sink: impl Sink<SinkItem=u8, SinkError=()> + Send + Sync + 'static) -> () {
        let msg = Msg::Subscribe(Box::new(sink));
        let _ = await!(self.sender.send_async(msg));
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

    let (send_broadcaster, mut recv_broadcaster) = mpsc::channel(0);

    tokio::spawn_async(async move {

        let mut outputters: Vec<BoxedSink<u8,()>> = vec![];
        while let Some(res) = await!(recv_broadcaster.next()) {

            let msg = match res {
                Ok(byte) => byte,
                Err(e) => { return eprintln!("Error receiving msg to broadcast: {:?}", e); }
            };

            match msg {

                // Subscribe a Sink to being sent output:
                Msg::Subscribe(sink) => {

                    // Subscribe a new sink to receive output. We have to newtype
                    // the sink into our own struct since Sink isn't implemented
                    // on Box<dyn Sink> for some reason:
                    outputters.push(BoxedSink(sink));

                },

                // Get given some output to send:
                Msg::Broadcast(byte) => {

                    // Send a message to each sink, recording any that failed:
                    let mut errored = vec![];
                    for (i, sink) in outputters.iter_mut().enumerate() {
                        if let Err(_) = await!(sink.send_async(byte)) {
                            errored.push(i);
                        }
                    }

                    // If sending to a sink fails, remove it from the vec:
                    if errored.len() > 0 {
                        outputters = outputters.into_iter().enumerate().filter_map(|(i,sink)| {
                            if errored.iter().find(|&&n| i == n).is_some() {
                                None
                            } else {
                                Some(sink)
                            }
                        }).collect();
                    }

                }
            }

        }
    });

    // return our interface:
    Broadcaster {
        sender: send_broadcaster
    }

}

// This is necessary to make Boxed Sinks actually impl the Sink trait,
// as for some reason they do not appear to at the moment:
struct BoxedSink<I,E>(Box<dyn Sink<SinkItem=I, SinkError=E> + Send + Sync + 'static>);
impl <I,E> Sink for BoxedSink<I,E> {
    type SinkItem = I;
    type SinkError = E;
    fn start_send(&mut self, input: Self::SinkItem) -> Result<AsyncSink<Self::SinkItem>, Self::SinkError> {
        self.0.start_send(input)
    }
    fn poll_complete(&mut self) -> Poll<(), Self::SinkError> {
        self.0.poll_complete()
    }
}