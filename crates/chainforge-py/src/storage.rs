use chainforge_storage::memory::InMemoryStorage;
use chainforge_storage::traits::StorageEngine;
use pyo3::prelude::*;
use pyo3::types::PyBytes;
use pyo3_async_runtimes::tokio::future_into_py;

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
            let result = engine
                .get(&key)
                .await
                .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(e.to_string()))?;
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
            engine
                .put(&key, &value)
                .await
                .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(e.to_string()))?;
            Python::with_gil(|py| Ok(py.None()))
        })
    }

    fn delete<'py>(&self, py: Python<'py>, key: Vec<u8>) -> PyResult<Bound<'py, PyAny>> {
        let engine = self.engine.clone();
        future_into_py(py, async move {
            engine
                .delete(&key)
                .await
                .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(e.to_string()))?;
            Python::with_gil(|py| Ok(py.None()))
        })
    }
}
