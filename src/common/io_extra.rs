use tokio::prelude::*;
use futures::try_ready;
use bytes::BytesMut;

/// Turn an AsyncRead into a Stream which emits 1 byte at a time.
pub fn stream_bytes<R: AsyncRead>(reader: R) -> impl Stream<Item = u8, Error = tokio::io::Error> {
    ByteStream::new(reader)
}

struct ByteStream<R>{
    r: R,
    mode: ReadMode
}

#[derive(PartialEq,Eq)]
enum ReadMode {
    Reading,
    Draining{ buf: BytesMut, pos: usize }
}

impl <R: AsyncRead> ByteStream<R> {
    fn new(reader: R) -> ByteStream<R> {
        ByteStream {
            r: reader,
            mode: ReadMode::Reading
        }
    }
}

impl <R: AsyncRead> ByteStream<R> {

    fn try_drain(&mut self) -> Option<u8> {
        if let ReadMode::Draining{ buf, pos } = &mut self.mode {
            if let Some(byte) = buf.get(*pos) {
                *pos = *pos + 1;
                return Some(*byte);
            } else {
                self.mode = ReadMode::Reading;
            }
        }
        None
    }

}

impl <R: AsyncRead> Stream for ByteStream<R> {

    type Item = u8;
    type Error = tokio::io::Error;

    fn poll(&mut self) -> Poll<Option<Self::Item>, Self::Error> {

        // If it's possible to drain bytes from buffer, do so:
        if let Some(byte) = self.try_drain() {
            return Ok(Async::Ready(Some(byte)));
        }

        let mut buf = BytesMut::with_capacity(31); // This size avoids allocation on 64bit

        // Try to read any available bytes into the buffer:
        let n = try_ready!(self.r.read_buf(&mut buf));
        if n == 0 {
            // no bytes left; assume the stream is done:
            return Ok(Async::Ready(None));
        }

        // having read into the buffer, ready it for draining:
        debug_assert!(self.mode == ReadMode::Reading);
        self.mode = ReadMode::Draining{ buf, pos: 0 };

        // Give back what we can straight away:
        if let Some(byte) = self.try_drain() {
            Ok(Async::Ready(Some(byte)))
        } else {
            Ok(Async::NotReady)
        }

    }

}

/// Turn an AsyncWrite into a Sink which takes 1 byte at a time.
pub fn sink_bytes<W: AsyncWrite>(writer: W) -> impl Sink<SinkItem = u8, SinkError = tokio::io::Error> {
    ByteSink::new(writer)
}

#[derive(Debug)]
struct ByteSink<W> {
    w: W,
    mode: WriteMode
}

#[derive(PartialEq, Debug)]
enum WriteMode {
    Writing,
    Flushing
}

impl <W: AsyncWrite> ByteSink<W> {

    fn new(writer: W) -> ByteSink<W> {
        ByteSink {
            w: writer,
            mode: WriteMode::Writing
        }
    }

}

impl <W: AsyncWrite> Sink for ByteSink<W> {

    type SinkItem = u8;
    type SinkError = tokio::io::Error;

    fn start_send(&mut self, item: u8) -> Result<AsyncSink<Self::SinkItem>, Self::SinkError> {

        // try flushing if needbe. If this succeeds we end up
        // in writing mode and can write our byte. if it fails
        // we'll be woken up to try again at some point.
        //
        // NOTE: It is important to ensure that if NotReady is
        // returned, the task will be notified to try again.
        // Having this first ensures that.
        self.poll_complete()?;

        // if we are in write mode, try writing:
        if let WriteMode::Writing = self.mode {
            if let Async::Ready(_) = self.w.poll_write(&[item;1])? {
                self.mode = WriteMode::Flushing;
                return Ok(AsyncSink::Ready);
            }
        }

        // We didn't succeed in writing our byte, or we are
        // stuck in Flushing mode. Both of which will wake us again:
        Ok(AsyncSink::NotReady(item))

    }

    fn poll_complete(&mut self) -> Poll<(), Self::SinkError> {

        // if we are in flushing mode, try flushing. If the flush
        // works, put us back in write mode. If not, return
        // NotReady to prompt another flush attempt.
        if let WriteMode::Flushing = self.mode {
            if let Async::NotReady = self.w.poll_flush()? {
                return Ok(Async::NotReady);
            } else {
                self.mode = WriteMode::Writing;
            }
        }

        // if our flush worked or we're in write mode,
        // we're happy to receive more:
        Ok(Async::Ready(()))

    }

}