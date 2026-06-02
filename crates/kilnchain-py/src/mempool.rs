use kilnchain_mempool::Mempool;
use pyo3::prelude::*;
use pyo3::types::PyBytes;

use crate::types::PyTransaction;

#[pyclass(name = "Mempool")]
pub struct PyMempool {
    inner: Mempool,
}

#[pymethods]
impl PyMempool {
    #[new]
    #[pyo3(signature = (capacity=10000))]
    fn new(capacity: usize) -> Self {
        PyMempool {
            inner: Mempool::with_capacity(capacity),
        }
    }

    fn insert(&mut self, tx: &Bound<'_, PyTransaction>) {
        self.inner.insert(tx.borrow().inner.clone());
    }

    fn get<'py>(&self, py: Python<'py>, hash: &[u8]) -> PyResult<Option<PyObject>> {
        let mut hash_arr = [0u8; 32];
        hash_arr.copy_from_slice(hash);
        match self.inner.get(&hash_arr) {
            Some(tx) => Py::new(py, PyTransaction { inner: tx.clone() })
                .map(|p| Some(p.into_pyobject(py).unwrap().into_any().unbind())),
            None => Ok(None),
        }
    }

    fn remove<'py>(&mut self, py: Python<'py>, hash: &[u8]) -> PyResult<Option<PyObject>> {
        let mut hash_arr = [0u8; 32];
        hash_arr.copy_from_slice(hash);
        match self.inner.remove(&hash_arr) {
            Some(tx) => Py::new(py, PyTransaction { inner: tx })
                .map(|p| Some(p.into_pyobject(py).unwrap().into_any().unbind())),
            None => Ok(None),
        }
    }

    fn contains(&self, hash: &[u8]) -> bool {
        let mut hash_arr = [0u8; 32];
        hash_arr.copy_from_slice(hash);
        self.inner.contains(&hash_arr)
    }

    fn len(&self) -> usize {
        self.inner.len()
    }

    fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    fn txs<'py>(&self, py: Python<'py>) -> PyResult<PyObject> {
        let dict = pyo3::types::PyDict::new(py);
        for (hash, tx) in self.inner.txs() {
            let key = PyBytes::new(py, hash);
            let value = Py::new(py, PyTransaction { inner: tx.clone() })?;
            dict.set_item(key, value)?;
        }
        Ok(dict.into_pyobject(py).unwrap().into_any().unbind())
    }

    fn pop_highest_priority<'py>(
        &mut self,
        py: Python<'py>,
        limit: usize,
    ) -> PyResult<Vec<PyObject>> {
        let txs = self.inner.pop_highest_priority(limit);
        txs.into_iter()
            .map(|tx| {
                Py::new(py, PyTransaction { inner: tx })
                    .map(|p| p.into_pyobject(py).unwrap().into_any().unbind())
            })
            .collect()
    }

    fn next_nonce(&self, sender: &[u8]) -> u64 {
        let mut sender_arr = [0u8; 20];
        sender_arr.copy_from_slice(sender);
        self.inner.next_nonce(&sender_arr)
    }

    fn is_nonce_valid(&self, tx: &Bound<'_, PyTransaction>) -> bool {
        self.inner.is_nonce_valid(&tx.borrow().inner)
    }

    fn produce_block(
        &mut self,
        py: Python,
        parent_hash: [u8; 32],
        number: u64,
        timestamp: u64,
        max_txs: usize,
    ) -> PyResult<PyObject> {
        let block = kilnchain_block_producer::produce_block(
            parent_hash,
            number,
            timestamp,
            &mut self.inner,
            max_txs,
        );
        Py::new(py, crate::types::PyBlock { inner: block })
            .map(|p| p.into_pyobject(py).unwrap().into_any().unbind())
    }
}
