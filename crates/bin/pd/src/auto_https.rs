use std::{
    pin::Pin,
    task::{Context, Poll},
};

use pin_project::pin_project;
use rustls_acme::futures_rustls::server::TlsStream;
use tokio::{
    io::{AsyncRead, AsyncWrite, ReadBuf},
    net::TcpStream,
};
use tokio_util::compat::Compat;
use tonic::transport::server::Connected;

/// Wrapper type needed to convert between futures_io and tokio traits
#[pin_project]
pub struct Wrapper {
    #[pin]
    pub inner: Compat<TlsStream<Compat<TcpStream>>>,
}

impl Connected for Wrapper {
    type ConnectInfo = <TcpStream as Connected>::ConnectInfo;

    fn connect_info(&self) -> Self::ConnectInfo {
        self.inner.get_ref().get_ref().0.get_ref().connect_info()
    }
}

impl AsyncRead for Wrapper {
    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<std::io::Result<()>> {
        self.project().inner.poll_read(cx, buf)
    }
}

impl AsyncWrite for Wrapper {
    fn poll_write(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<std::io::Result<usize>> {
        self.project().inner.poll_write(cx, buf)
    }
    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<std::io::Result<()>> {
        self.project().inner.poll_flush(cx)
    }
    fn poll_shutdown(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<std::io::Result<()>> {
        self.project().inner.poll_shutdown(cx)
    }
}
