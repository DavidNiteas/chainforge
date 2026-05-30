use chainforge_storage::cache::CachedStorage;
use chainforge_storage::memory::InMemoryStorage;
use chainforge_storage::traits::StorageEngine;
use pyo3::prelude::*;
use pyo3::types::PyBytes;
use pyo3_async_runtimes::tokio::future_into_py;

use crate::error::into_py_err;

#[cfg(feature = "rocksdb-backend")]
use chainforge_storage::rocksdb::RocksDBEngine;
#[cfg(feature = "rocksdb-backend")]
use std::sync::Arc;

#[pyclass(name = "InMemoryStorage")]
pub struct PyInMemoryStorage {
    engine: InMemoryStorage,
}

#[pymethods]
impl PyInMemoryStorage {
    #[new]
    fn new() -> Self {
        PyInMemoryStorage {
            engine: InMemoryStorage::new(),
        }
    }

    fn get<'py>(&self, py: Python<'py>, key: Vec<u8>) -> PyResult<Bound<'py, PyAny>> {
        let engine = self.engine.clone();
        future_into_py(py, async move {
            let result = engine.get(&key).await.map_err(into_py_err)?;
            Python::with_gil(|py| match result {
                Some(v) => Ok(PyBytes::new(py, &v)
                    .into_pyobject(py)
                    .unwrap()
                    .into_any()
                    .unbind()),
                None => Ok(py.None()),
            })
        })
    }

    fn put<'py>(
        &self,
        py: Python<'py>,
        key: Vec<u8>,
        value: Vec<u8>,
    ) -> PyResult<Bound<'py, PyAny>> {
        let engine = self.engine.clone();
        future_into_py(py, async move {
            engine.put(&key, &value).await.map_err(into_py_err)?;
            Python::with_gil(|py| Ok(py.None()))
        })
    }

    fn delete<'py>(&self, py: Python<'py>, key: Vec<u8>) -> PyResult<Bound<'py, PyAny>> {
        let engine = self.engine.clone();
        future_into_py(py, async move {
            engine.delete(&key).await.map_err(into_py_err)?;
            Python::with_gil(|py| Ok(py.None()))
        })
    }
}

#[pyclass(name = "CachedStorage")]
pub struct PyCachedStorage {
    engine: CachedStorage<InMemoryStorage>,
}

#[pymethods]
impl PyCachedStorage {
    #[new]
    fn new(capacity: usize) -> Self {
        PyCachedStorage {
            engine: CachedStorage::new(InMemoryStorage::new(), capacity),
        }
    }

    fn get<'py>(&self, py: Python<'py>, key: Vec<u8>) -> PyResult<Bound<'py, PyAny>> {
        let engine = self.engine.clone();
        future_into_py(py, async move {
            let result = engine.get(&key).await.map_err(into_py_err)?;
            Python::with_gil(|py| match result {
                Some(v) => Ok(PyBytes::new(py, &v)
                    .into_pyobject(py)
                    .unwrap()
                    .into_any()
                    .unbind()),
                None => Ok(py.None()),
            })
        })
    }

    fn put<'py>(
        &self,
        py: Python<'py>,
        key: Vec<u8>,
        value: Vec<u8>,
    ) -> PyResult<Bound<'py, PyAny>> {
        let engine = self.engine.clone();
        future_into_py(py, async move {
            engine.put(&key, &value).await.map_err(into_py_err)?;
            Python::with_gil(|py| Ok(py.None()))
        })
    }

    fn delete<'py>(&self, py: Python<'py>, key: Vec<u8>) -> PyResult<Bound<'py, PyAny>> {
        let engine = self.engine.clone();
        future_into_py(py, async move {
            engine.delete(&key).await.map_err(into_py_err)?;
            Python::with_gil(|py| Ok(py.None()))
        })
    }
}

#[cfg(feature = "rocksdb-backend")]
#[pyclass(name = "RocksDBEngine")]
pub struct PyRocksDBEngine {
    engine: Arc<RocksDBEngine>,
}

#[cfg(feature = "rocksdb-backend")]
#[pymethods]
impl PyRocksDBEngine {
    #[new]
    fn new(path: &str) -> PyResult<Self> {
        let engine = RocksDBEngine::open(std::path::Path::new(path)).map_err(into_py_err)?;
        Ok(PyRocksDBEngine {
            engine: Arc::new(engine),
        })
    }

    fn get<'py>(&self, py: Python<'py>, key: Vec<u8>) -> PyResult<Bound<'py, PyAny>> {
        let engine = self.engine.clone();
        future_into_py(py, async move {
            let result = engine.get(&key).await.map_err(into_py_err)?;
            Python::with_gil(|py| match result {
                Some(v) => Ok(PyBytes::new(py, &v)
                    .into_pyobject(py)
                    .unwrap()
                    .into_any()
                    .unbind()),
                None => Ok(py.None()),
            })
        })
    }

    fn put<'py>(
        &self,
        py: Python<'py>,
        key: Vec<u8>,
        value: Vec<u8>,
    ) -> PyResult<Bound<'py, PyAny>> {
        let engine = self.engine.clone();
        future_into_py(py, async move {
            engine.put(&key, &value).await.map_err(into_py_err)?;
            Python::with_gil(|py| Ok(py.None()))
        })
    }

    fn delete<'py>(&self, py: Python<'py>, key: Vec<u8>) -> PyResult<Bound<'py, PyAny>> {
        let engine = self.engine.clone();
        future_into_py(py, async move {
            engine.delete(&key).await.map_err(into_py_err)?;
            Python::with_gil(|py| Ok(py.None()))
        })
    }
}
