//! 下游写停滞超时；只在真正写出字节后重置计时。
//!
//! 每条连接独立计时：写入 Pending 时开始计时，写出字节时清除计时器。
//! 再次推进连接时，如果计时器已到期，就返回超时错误。

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
    // None：当前没有正在计时的写入停滞。
    // Some：从一次写入 Pending 开始计时，后续 Pending 复用同一个截止时间。
    // Box 在堆上保存 Sleep；Pin 保证 Sleep 被 poll 后，其位置不会改变。
    write_timer: Option<Pin<Box<Sleep>>>,
}

impl WriteTimeoutIo {
    fn check_timeout(&mut self, cx: &mut Context<'_>) -> io::Result<()> {
        // cx 提供当前任务的 Waker，不保存计时器；计时器保存在 self.write_timer。
        // poll 返回 Pending 时，Sleep 会安排到期后的唤醒；唤醒后还需要再次 poll。
        // 没有计时器或尚未到期都返回 Ok；Ok 本身不表示网络写入已经完成。
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
                    // 真正写出了字节，结束这一轮停滞计时。
                    // 写出 0 字节不算取得进展，因此不清除已有计时器。
                    self.write_timer = None;
                }
                Poll::Ready(Ok(count))
            }
            Poll::Ready(Err(error)) => Poll::Ready(Err(error)),
            Poll::Pending => {
                // 首次写不动才创建 Sleep；重复被唤醒但仍写不动时不重置它，
                // 否则每次尝试都会延长截止时间，持续停滞可能永远不超时。
                if self.write_timer.is_none() {
                    self.write_timer = Some(Box::pin(tokio::time::sleep(self.timeout)));
                }
                // 检查是否已经超时，并让尚未到期的 Sleep 登记当前任务的 Waker。
                // 网络可写和计时器到期都可能触发唤醒；唤醒不代表写入必然成功。
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
        // 收到客户端数据不代表下游写入取得进展，不重置写计时器。
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
        // 先检查已有截止时间，再尝试底层 socket 写入。
        // 底层 socket 使用同一个 cx，在暂时不可写时登记后续唤醒。
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
        // 分散写入和普通写入共享同一套停滞计时规则。
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
        // 复用 Axum 对 accept 错误的处理与重试；每个新连接拥有独立计时器。
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
