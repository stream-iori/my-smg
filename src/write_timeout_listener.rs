//! 下游写停滞超时；只在真正写出字节后重置计时。

use axum::serve::Listener;
use std::future::Future;
use std::io::{self, IoSlice};
use std::net::SocketAddr;
use std::pin::Pin;
use std::task::{Context, Poll};
use std::time::Duration;
use tokio::io::{AsyncRead, AsyncWrite, ReadBuf};
use tokio::net::{TcpListener, TcpStream};
use tokio::time::Sleep;

pub(crate) struct WriteTimeoutListener {
    inner: TcpListener,
    timeout: Duration,
}

impl WriteTimeoutListener {
    pub(crate) fn new(inner: TcpListener, timeout: Duration) -> Self {
        Self { inner, timeout }
    }
}

pub(crate) struct WriteTimeoutIo {
    inner: TcpStream,
    timeout: Duration,
    write_timer: Option<Pin<Box<Sleep>>>,
}

impl WriteTimeoutIo {
    fn check_timeout(&mut self, cx: &mut Context<'_>) -> io::Result<()> {
        if let Some(timer) = self.write_timer.as_mut()
            && timer.as_mut().poll(cx).is_ready()
        {
            return Err(io::Error::new(
                io::ErrorKind::TimedOut,
                "downstream socket write stalled",
            ));
        }
        Ok(())
    }

    fn finish_write(
        &mut self,
        cx: &mut Context<'_>,
        result: Poll<io::Result<usize>>,
    ) -> Poll<io::Result<usize>> {
        match result {
            Poll::Ready(Ok(count)) => {
                if count > 0 {
                    //真正写出了字节，结束这一轮停滞计时。
                    self.write_timer = None;
                }
                Poll::Ready(Ok(count))
            }
            Poll::Ready(Err(error)) => Poll::Ready(Err(error)),
            Poll::Pending => {
                if self.write_timer.is_none() {
                    self.write_timer = Some(Box::pin(tokio::time::sleep(self.timeout)));
                }
                //poll Sleep, 登记当前链接任务的Waker.
                match self.check_timeout(cx) {
                    Ok(()) => Poll::Pending,
                    Err(error) => Poll::Ready(Err(error)),
                }
            }
        }
    }
}

impl AsyncRead for WriteTimeoutIo {
    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buffer: &mut ReadBuf<'_>,
    ) -> Poll<io::Result<()>> {
        let this = self.get_mut();
        if let Err(error) = this.check_timeout(cx) {
            return Poll::Ready(Err(error));
        }
        //收到客户端数据不代表下游写入取得进展,不重置写计时器。
        Pin::new(&mut this.inner).poll_read(cx, buffer)
    }
}

impl AsyncWrite for WriteTimeoutIo {
    fn poll_write(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buffer: &[u8],
    ) -> Poll<io::Result<usize>> {
        let this = self.get_mut();
        if let Err(error) = this.check_timeout(cx) {
            return Poll::Ready(Err(error));
        }
        let result = Pin::new(&mut this.inner).poll_write(cx, buffer);
        this.finish_write(cx, result)
    }

    fn poll_write_vectored(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buffers: &[IoSlice<'_>],
    ) -> Poll<io::Result<usize>> {
        let this = self.get_mut();
        if let Err(error) = this.check_timeout(cx) {
            return Poll::Ready(Err(error));
        }
        let result = Pin::new(&mut this.inner).poll_write_vectored(cx, buffers);
        this.finish_write(cx, result)
    }

    fn is_write_vectored(&self) -> bool {
        self.inner.is_write_vectored()
    }

    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        let this = self.get_mut();
        if let Err(error) = this.check_timeout(cx) {
            return Poll::Ready(Err(error));
        }
        // TCP flush 成功不等于又写出了字节，不能重置计时器。
        Pin::new(&mut this.inner).poll_flush(cx)
    }

    fn poll_shutdown(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        let this = self.get_mut();
        if let Err(error) = this.check_timeout(cx) {
            return Poll::Ready(Err(error));
        }
        Pin::new(&mut this.inner).poll_shutdown(cx)
    }
}

impl Listener for WriteTimeoutListener {
    type Io = WriteTimeoutIo;

    type Addr = SocketAddr;

    async fn accept(&mut self) -> (Self::Io, Self::Addr) {
        //复用Axum对accpet错误的处理与重试
        let (stream, address) = Listener::accept(&mut self.inner).await;
        (
            WriteTimeoutIo {
                inner: stream,
                timeout: self.timeout,
                write_timer: None,
            },
            address,
        )
    }

    fn local_addr(&self) -> io::Result<Self::Addr> {
        self.inner.local_addr()
    }
}
