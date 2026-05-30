use chainforge_block_producer::BlockBuilder;
use pyo3::prelude::*;

use crate::types::{PyBlock, PyTransaction};

#[pyclass(name = "BlockBuilder")]
pub struct PyBlockBuilder {
    inner: BlockBuilder,
}

#[pymethods]
impl PyBlockBuilder {
    #[new]
    fn new(parent_hash: [u8; 32], number: u64) -> Self {
        PyBlockBuilder {
            inner: BlockBuilder::new(parent_hash, number),
        }
    }

    fn timestamp(&self, py: Python, ts: u64) -> PyResult<PyObject> {
        let builder = self.inner.clone().timestamp(ts);
        Py::new(py, PyBlockBuilder { inner: builder })
            .map(|p| p.into_pyobject(py).unwrap().into_any().unbind())
    }

    fn extra_data(&self, py: Python, data: Vec<u8>) -> PyResult<PyObject> {
        let builder = self.inner.clone().extra_data(data);
        Py::new(py, PyBlockBuilder { inner: builder })
            .map(|p| p.into_pyobject(py).unwrap().into_any().unbind())
    }

    fn state_root(&self, py: Python, root: [u8; 32]) -> PyResult<PyObject> {
        let builder = self.inner.clone().state_root(root);
        Py::new(py, PyBlockBuilder { inner: builder })
            .map(|p| p.into_pyobject(py).unwrap().into_any().unbind())
    }

    fn transactions(&self, py: Python, txs: Vec<PyRef<'_, PyTransaction>>) -> PyResult<PyObject> {
        let txs: Vec<chainforge_core::tx::Transaction> =
            txs.into_iter().map(|t| t.inner.clone()).collect();
        let builder = self.inner.clone().transactions(txs);
        Py::new(py, PyBlockBuilder { inner: builder })
            .map(|p| p.into_pyobject(py).unwrap().into_any().unbind())
    }

    fn gas_limit(&self, py: Python, limit: u64) -> PyResult<PyObject> {
        let builder = self.inner.clone().gas_limit(limit);
        Py::new(py, PyBlockBuilder { inner: builder })
            .map(|p| p.into_pyobject(py).unwrap().into_any().unbind())
    }

    fn build(&self, py: Python) -> PyResult<PyObject> {
        let block = self.inner.clone().build();
        Py::new(py, PyBlock { inner: block })
            .map(|p| p.into_pyobject(py).unwrap().into_any().unbind())
    }
}
