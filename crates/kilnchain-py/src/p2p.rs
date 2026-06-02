//! PyO3 bindings for kilnchain-p2p (P2P networking layer).

use kilnchain_p2p::{
    discovery::RoutingTable,
    message::Message,
    node::{Node, NodeConfig},
    peer::{PeerId, PeerInfo},
};
use pyo3::prelude::*;
use pyo3::types::PyBytes;
use pyo3_async_runtimes::tokio::future_into_py;

use crate::error::into_py_err;
use crate::types::PyTransaction;

// ---------------------------------------------------------------------------
// PeerId
// ---------------------------------------------------------------------------

#[pyclass(name = "PeerId")]
pub struct PyPeerId {
    pub(crate) inner: PeerId,
}

#[pymethods]
impl PyPeerId {
    #[new]
    fn new(bytes: &[u8]) -> PyResult<Self> {
        let arr: [u8; 32] = bytes
            .try_into()
            .map_err(|_| pyo3::exceptions::PyValueError::new_err("PeerId must be 32 bytes"))?;
        Ok(PyPeerId { inner: PeerId(arr) })
    }

    #[staticmethod]
    fn from_public_key(pk: &[u8]) -> Self {
        PyPeerId {
            inner: PeerId::from_public_key(pk),
        }
    }

    fn to_bytes<'py>(&self, py: Python<'py>) -> Bound<'py, PyBytes> {
        PyBytes::new(py, &self.inner.0)
    }
}

// ---------------------------------------------------------------------------
// PeerInfo
// ---------------------------------------------------------------------------

#[pyclass(name = "PeerInfo")]
pub struct PyPeerInfo {
    pub(crate) inner: PeerInfo,
}

#[pymethods]
impl PyPeerInfo {
    #[new]
    fn new(id: &Bound<'_, PyPeerId>, addr: &str) -> PyResult<Self> {
        let addr = addr.parse().map_err(|e| {
            pyo3::exceptions::PyValueError::new_err(format!("invalid address: {}", e))
        })?;
        Ok(PyPeerInfo {
            inner: PeerInfo {
                id: id.borrow().inner,
                addr,
            },
        })
    }

    #[getter]
    fn id<'py>(&self, py: Python<'py>) -> PyResult<PyObject> {
        Py::new(
            py,
            PyPeerId {
                inner: self.inner.id,
            },
        )
        .map(|p| p.into_pyobject(py).unwrap().into_any().unbind())
    }

    #[getter]
    fn addr(&self) -> String {
        self.inner.addr.to_string()
    }
}

// ---------------------------------------------------------------------------
// Message
// ---------------------------------------------------------------------------

#[pyclass(name = "Message")]
pub struct PyMessage {
    pub(crate) inner: Message,
}

#[pymethods]
impl PyMessage {
    #[staticmethod]
    fn ping() -> Self {
        PyMessage {
            inner: Message::Ping,
        }
    }

    #[staticmethod]
    fn pong() -> Self {
        PyMessage {
            inner: Message::Pong,
        }
    }

    #[staticmethod]
    fn transaction(data: &[u8]) -> Self {
        PyMessage {
            inner: Message::Transaction(data.to_vec()),
        }
    }

    #[staticmethod]
    fn block(data: &[u8]) -> Self {
        PyMessage {
            inner: Message::Block(data.to_vec()),
        }
    }

    #[getter]
    fn is_ping(&self) -> bool {
        matches!(self.inner, Message::Ping)
    }

    #[getter]
    fn is_pong(&self) -> bool {
        matches!(self.inner, Message::Pong)
    }

    #[getter]
    fn is_transaction(&self) -> bool {
        matches!(self.inner, Message::Transaction(_))
    }

    #[getter]
    fn is_block(&self) -> bool {
        matches!(self.inner, Message::Block(_))
    }

    fn encode<'py>(&self, py: Python<'py>) -> Bound<'py, PyBytes> {
        PyBytes::new(py, &self.inner.encode())
    }

    #[staticmethod]
    fn decode(py: Python, data: &[u8]) -> PyResult<PyObject> {
        let msg = Message::decode(data)
            .map_err(|e| pyo3::exceptions::PyValueError::new_err(e.to_string()))?;
        Py::new(py, PyMessage { inner: msg })
            .map(|p| p.into_pyobject(py).unwrap().into_any().unbind())
    }

    fn decode_transaction<'py>(&self, py: Python<'py>) -> PyResult<Option<PyObject>> {
        match &self.inner {
            Message::Transaction(bytes) => {
                let tx = kilnchain_core::tx::Transaction::decode_rlp(bytes).map_err(into_py_err)?;
                Py::new(py, PyTransaction { inner: tx })
                    .map(|p| Some(p.into_pyobject(py).unwrap().into_any().unbind()))
            }
            _ => Ok(None),
        }
    }
}

// ---------------------------------------------------------------------------
// NodeConfig
// ---------------------------------------------------------------------------

#[pyclass(name = "NodeConfig")]
pub struct PyNodeConfig {
    pub(crate) inner: NodeConfig,
}

#[pymethods]
impl PyNodeConfig {
    #[new]
    fn new(static_key: &[u8]) -> PyResult<Self> {
        let arr: [u8; 32] = static_key
            .try_into()
            .map_err(|_| pyo3::exceptions::PyValueError::new_err("static_key must be 32 bytes"))?;
        Ok(PyNodeConfig {
            inner: NodeConfig::new(arr),
        })
    }

    #[getter]
    fn gossip_fanout(&self) -> usize {
        self.inner.gossip_fanout
    }

    #[setter]
    fn set_gossip_fanout(&mut self, v: usize) {
        self.inner.gossip_fanout = v;
    }

    #[getter]
    fn gossip_ttl_secs(&self) -> u64 {
        self.inner.gossip_ttl_secs
    }

    #[setter]
    fn set_gossip_ttl_secs(&mut self, v: u64) {
        self.inner.gossip_ttl_secs = v;
    }

    #[getter]
    fn local_id<'py>(&self, py: Python<'py>) -> PyResult<PyObject> {
        Py::new(
            py,
            PyPeerId {
                inner: self.inner.local_id,
            },
        )
        .map(|p| p.into_pyobject(py).unwrap().into_any().unbind())
    }
}

// ---------------------------------------------------------------------------
// RoutingTable
// ---------------------------------------------------------------------------

#[pyclass(name = "RoutingTable")]
pub struct PyRoutingTable {
    pub(crate) inner: RoutingTable,
}

#[pymethods]
impl PyRoutingTable {
    #[new]
    fn new(local_id: &Bound<'_, PyPeerId>) -> Self {
        PyRoutingTable {
            inner: RoutingTable::new(local_id.borrow().inner),
        }
    }

    fn update(&mut self, peer: &Bound<'_, PyPeerInfo>) {
        self.inner.update(peer.borrow().inner.clone());
    }

    fn find_closest<'py>(
        &self,
        py: Python<'py>,
        target: &Bound<'_, PyPeerId>,
        k: usize,
    ) -> PyResult<Vec<PyObject>> {
        let peers = self.inner.find_closest(&target.borrow().inner, k);
        peers
            .into_iter()
            .map(|p| {
                Py::new(py, PyPeerInfo { inner: p })
                    .map(|p| p.into_pyobject(py).unwrap().into_any().unbind())
            })
            .collect()
    }

    fn len(&self) -> usize {
        self.inner.len()
    }

    fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }
}

// ---------------------------------------------------------------------------
// Node
// ---------------------------------------------------------------------------

#[pyclass(name = "Node")]
pub struct PyNode {
    pub(crate) inner: Node,
}

#[pymethods]
impl PyNode {
    #[new]
    fn new(config: &Bound<'_, PyNodeConfig>) -> Self {
        PyNode {
            inner: Node::new(config.borrow().inner.clone()),
        }
    }

    fn handle_message<'py>(
        &self,
        py: Python<'py>,
        msg: &Bound<'_, PyMessage>,
    ) -> PyResult<Bound<'py, PyAny>> {
        let node = self.inner.clone();
        let msg = msg.borrow().inner.clone();
        future_into_py(py, async move {
            let result = node.handle_message(&msg).await;
            Python::with_gil(|py| {
                let list = pyo3::types::PyList::empty(py);
                for m in result {
                    let py_msg = Py::new(py, PyMessage { inner: m })
                        .map(|p| p.into_pyobject(py).unwrap().into_any().unbind())?;
                    list.append(py_msg)?;
                }
                Ok(list.into_pyobject(py).unwrap().into_any().unbind())
            })
        })
    }

    fn gossip_targets<'py>(
        &self,
        py: Python<'py>,
        msg: &Bound<'_, PyMessage>,
    ) -> PyResult<Bound<'py, PyAny>> {
        let node = self.inner.clone();
        let msg = msg.borrow().inner.clone();
        future_into_py(py, async move {
            let result = node.gossip_targets(&msg).await;
            Python::with_gil(|py| {
                let list = pyo3::types::PyList::empty(py);
                for p in result {
                    let py_peer = Py::new(py, PyPeerInfo { inner: p })
                        .map(|p| p.into_pyobject(py).unwrap().into_any().unbind())?;
                    list.append(py_peer)?;
                }
                Ok(list.into_pyobject(py).unwrap().into_any().unbind())
            })
        })
    }

    #[pyo3(signature = (limit=100))]
    fn drain_inbox<'py>(&self, py: Python<'py>, limit: usize) -> PyResult<Bound<'py, PyAny>> {
        let node = self.inner.clone();
        future_into_py(py, async move {
            let result = node.drain_inbox(limit).await;
            Python::with_gil(|py| {
                let list = pyo3::types::PyList::empty(py);
                for m in result {
                    let py_msg = Py::new(py, PyMessage { inner: m })
                        .map(|p| p.into_pyobject(py).unwrap().into_any().unbind())?;
                    list.append(py_msg)?;
                }
                Ok(list.into_pyobject(py).unwrap().into_any().unbind())
            })
        })
    }

    fn routing_table<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        let rt = self.inner.routing_table.clone();
        future_into_py(py, async move {
            let table = rt.read().await.clone();
            Python::with_gil(|py| {
                Py::new(py, PyRoutingTable { inner: table })
                    .map(|p| p.into_pyobject(py).unwrap().into_any().unbind())
            })
        })
    }
}
