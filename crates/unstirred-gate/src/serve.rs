#![doc = "The listener. One hyper connection per accepted socket, answered by the dispatcher, with \
no middleware of any kind: nothing between the socket and the handler that could record who \
connected. The peer address is dropped at the accept, which is the whole of the no-IP-logging \
rule (LEAKS L9, A39)."]

use std::net::{SocketAddr, TcpListener};
use std::sync::Arc;
use std::thread::JoinHandle;

use hyper::server::conn::http1;
use hyper::service::service_fn;
use hyper_util::rt::TokioIo;

use crate::handlers::dispatch;
use crate::state::GateState;

/// Why the listener stopped, or never started.
#[derive(Debug, thiserror::Error)]
pub enum ServeError {
    /// The address could not be bound.
    #[error("the gate could not bind {listen}: {source}")]
    Bind {
        /// The address that was tried.
        listen: SocketAddr,
        /// What the operating system reported.
        source: std::io::Error,
    },
    /// No async runtime could be started to serve on.
    #[error("the gate could not start a runtime: {source}")]
    Runtime {
        /// What the runtime builder reported.
        source: std::io::Error,
    },
    /// The listener itself failed while accepting.
    #[error("the gate's listener stopped: {source}")]
    Listener {
        /// What the operating system reported.
        source: std::io::Error,
    },
    /// The serving thread ended without returning anything, so why it stopped is not known.
    #[error("the gate's thread ended without a result")]
    ThreadLost,
}

/// A gate that is listening. Dropping this value leaves it listening until the process ends;
/// there is no shutdown route, so a gate is stopped the way a server is stopped.
#[derive(Debug)]
pub struct RunningGate {
    address: SocketAddr,
    thread: JoinHandle<Result<(), ServeError>>,
}

impl RunningGate {
    /// The address actually bound, which is what a config asking for port 0 needs to be told.
    pub fn address(&self) -> SocketAddr {
        self.address
    }

    /// The base URL a holder would knock on.
    pub fn url(&self) -> String {
        format!("http://{}", self.address)
    }

    /// Blocks until the listener stops, which it does only on an error.
    pub fn wait(self) -> Result<(), ServeError> {
        match self.thread.join() {
            Ok(result) => result,
            Err(_) => Err(ServeError::ThreadLost),
        }
    }
}

/// Binds now and serves on a thread of its own, so the caller knows the port before the first
/// request. The startup lines are written here: they are the only place a gate prints an address.
pub fn start(state: GateState) -> Result<RunningGate, ServeError> {
    let listener = TcpListener::bind(state.listen).map_err(|source| ServeError::Bind {
        listen: state.listen,
        source,
    })?;
    let address = listener
        .local_addr()
        .map_err(|source| ServeError::Listener { source })?;
    listener
        .set_nonblocking(true)
        .map_err(|source| ServeError::Listener { source })?;
    announce(&state, address);
    let thread = std::thread::spawn(move || run(listener, state));
    Ok(RunningGate { address, thread })
}

/// Serves until the listener fails. This is what `unstirred gate serve` calls.
pub fn serve(state: GateState) -> Result<(), ServeError> {
    start(state)?.wait()
}

fn announce(state: &GateState, address: SocketAddr) {
    state.log.line(&format!("gate listening on {address}"));
    state.log.line(&format!("database {}", state.database.display()));
    state
        .log
        .line(&format!("root sources {}", state.roots.describe().join(", ")));
    if let Some(note) = state.note() {
        state.log.line(note);
    }
    for room in state.rooms() {
        let window = room.manifest.window;
        state.log.line(&format!(
            "room {} network {} window h0 {} through h1 {} h1_root {} (operator-pinned)",
            room.context(),
            room.manifest.network,
            window.h0,
            window.h1,
            hex::encode(room.h1_root)
        ));
    }
}

fn run(listener: TcpListener, state: GateState) -> Result<(), ServeError> {
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .map_err(|source| ServeError::Runtime { source })?;
    runtime.block_on(accept_forever(listener, state))
}

async fn accept_forever(listener: TcpListener, state: GateState) -> Result<(), ServeError> {
    let listener = tokio::net::TcpListener::from_std(listener).map_err(|source| ServeError::Listener { source })?;
    let state = Arc::new(state);
    loop {
        let (stream, _peer_address_the_gate_never_keeps) = listener
            .accept()
            .await
            .map_err(|source| ServeError::Listener { source })?;
        let state = Arc::clone(&state);
        tokio::task::spawn(async move {
            let io = TokioIo::new(stream);
            let service = service_fn(move |request| {
                let state = Arc::clone(&state);
                async move { Ok::<_, std::convert::Infallible>(dispatch(&state, request).await) }
            });
            let _served = http1::Builder::new().serve_connection(io, service).await;
        });
    }
}
