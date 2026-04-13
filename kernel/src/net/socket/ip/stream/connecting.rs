// SPDX-License-Identifier: MPL-2.0

use aster_bigtcp::{
    errors::tcp::ConnectError,
    socket::{ConnectState, RawTcpOption, RawTcpSetOption},
    wire::IpEndpoint,
};

use super::{connected::ConnectedStream, init::InitStream, observer::StreamObserver};
use crate::{
    events::IoEvents,
    net::iface::{BoundPort, Iface, TcpConnection},
    prelude::*,
};

pub(super) struct ConnectingStream {
    tcp_conn: TcpConnection,
    remote_endpoint: IpEndpoint,
}

pub(super) enum ConnResult {
    Connecting(ConnectingStream),
    Connected(ConnectedStream),
    Refused(InitStream),
}

impl ConnectingStream {
    pub(super) fn new(
        bound_port: BoundPort,
        remote_endpoint: IpEndpoint,
        option: &RawTcpOption,
        observer: StreamObserver,
    ) -> core::result::Result<Self, (Error, BoundPort)> {
        emerg!("ConnectingStream::new: bound_port addr={}, remote={}", bound_port.addr(), remote_endpoint);
        let tcp_conn =
            match TcpConnection::new_connect(bound_port, remote_endpoint, option, observer) {
                Ok(tcp_conn) => {
                    emerg!("ConnectingStream::new: TcpConnection created successfully");
                    tcp_conn
                }
                Err((bp, ConnectError::AddressInUse)) => {
                    emerg!("ConnectingStream::new: AddressInUse error");
                    return Err((
                        Error::with_message(Errno::EADDRNOTAVAIL, "connection key conflicts"),
                        bp,
                    ));
                }
                Err((bp, err)) => {
                    emerg!("ConnectingStream::new: connection error {:?}", err);
                    // The only reason this method might go to this branch is because
                    // we're trying to connect to an unspecified address (i.e. 0.0.0.0).
                    // We currently have no support for binding to,
                    // listening on, or connecting to the unspecified address.
                    //
                    // We assume the remote will just refuse to connect,
                    // so we return `ECONNREFUSED`.
                    return Err((
                        Error::with_message(
                            Errno::ECONNREFUSED,
                            "connecting to an unspecified address is not supported",
                        ),
                        bp,
                    ));
                }
            };

        Ok(Self {
            tcp_conn,
            remote_endpoint,
        })
    }

    pub(super) fn has_result(&self) -> bool {
        match self.tcp_conn.connect_state() {
            ConnectState::Connecting => false,
            ConnectState::Connected => true,
            ConnectState::Refused => true,
        }
    }

    pub(super) fn into_result(self) -> ConnResult {
        let next_state = self.tcp_conn.connect_state();

        match next_state {
            ConnectState::Connecting => ConnResult::Connecting(self),
            ConnectState::Connected => ConnResult::Connected(ConnectedStream::new(
                self.tcp_conn,
                self.remote_endpoint,
                true,
            )),
            ConnectState::Refused => ConnResult::Refused(InitStream::new_refused(
                self.tcp_conn.into_bound_port().unwrap(),
            )),
        }
    }

    pub(super) fn local_endpoint(&self) -> IpEndpoint {
        self.tcp_conn.local_endpoint().unwrap()
    }

    pub(super) fn remote_endpoint(&self) -> IpEndpoint {
        self.remote_endpoint
    }

    pub(super) fn iface(&self) -> &Arc<Iface> {
        self.tcp_conn.iface()
    }

    pub(super) fn bound_port(&self) -> &BoundPort {
        self.tcp_conn.bound_port()
    }

    pub(super) fn check_io_events(&self) -> IoEvents {
        use aster_bigtcp::socket::ConnectState;
        
        match self.tcp_conn.connect_state() {
            ConnectState::Connected => {
                emerg!("check_io_events: connection established, returning OUT");
                IoEvents::OUT
            }
            ConnectState::Connecting => {
                emerg!("check_io_events: still connecting");
                IoEvents::empty()
            }
            ConnectState::Refused => {
                emerg!("check_io_events: connection refused");
                IoEvents::empty()
            }
        }
    }

    pub(super) fn set_raw_option<R>(
        &self,
        set_option: impl FnOnce(&dyn RawTcpSetOption) -> R,
    ) -> R {
        set_option(&self.tcp_conn)
    }

    pub(super) fn into_connection(self) -> TcpConnection {
        self.tcp_conn
    }
}
