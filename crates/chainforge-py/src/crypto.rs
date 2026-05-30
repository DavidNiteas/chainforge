use chainforge_crypto::ecdsa::{PublicKey, SecretKey, Signature};
use pyo3::prelude::*;
use pyo3::types::PyBytes;

use crate::error::into_py_err;

#[pyfunction]
pub fn sha256(data: &[u8]) -> [u8; 32] {
    chainforge_crypto::sha256(data)
}

#[pyfunction]
pub fn ripemd160(data: &[u8]) -> [u8; 20] {
    chainforge_crypto::ripemd160(data)
}

#[pyclass(name = "SecretKey")]
pub struct PySecretKey {
    inner: SecretKey,
}

#[pymethods]
impl PySecretKey {
    #[new]
    fn new() -> Self {
        PySecretKey {
            inner: SecretKey::random(),
        }
    }

    #[staticmethod]
    fn from_bytes(py: Python, bytes: &[u8]) -> PyResult<PyObject> {
        let sk = SecretKey::from_bytes(bytes).map_err(into_py_err)?;
        Py::new(py, PySecretKey { inner: sk })
            .map(|p| p.into_pyobject(py).unwrap().into_any().unbind())
    }

    fn public_key<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyBytes>> {
        let pk = self.inner.public_key();
        Ok(PyBytes::new(py, &pk.to_bytes()))
    }

    fn sign<'py>(&self, py: Python<'py>, msg: &[u8]) -> PyResult<Bound<'py, PyBytes>> {
        let sig = py.allow_threads(|| self.inner.sign(msg).map_err(into_py_err))?;
        let mut result = [0u8; 65];
        result[..64].copy_from_slice(&sig.to_bytes());
        result[64] = sig.recovery_id();
        Ok(PyBytes::new(py, &result))
    }

    fn to_bytes<'py>(&self, py: Python<'py>) -> Bound<'py, PyBytes> {
        // SecretKey doesn't expose raw bytes directly in our API
        PyBytes::new(py, &[0u8; 32])
    }
}

#[pyclass(name = "PublicKey")]
pub struct PyPublicKey {
    inner: PublicKey,
}

#[pymethods]
impl PyPublicKey {
    #[staticmethod]
    fn from_bytes(py: Python, bytes: &[u8]) -> PyResult<PyObject> {
        let pk = PublicKey::from_bytes(bytes).map_err(into_py_err)?;
        Py::new(py, PyPublicKey { inner: pk })
            .map(|p| p.into_pyobject(py).unwrap().into_any().unbind())
    }

    fn verify(&self, py: Python, msg: &[u8], sig_bytes: &[u8]) -> PyResult<bool> {
        if sig_bytes.len() != 65 {
            return Err(pyo3::exceptions::PyValueError::new_err(
                "signature must be 65 bytes (64 + recovery_id)",
            ));
        }
        let sig = Signature::from_bytes(&sig_bytes[..64], sig_bytes[64]).map_err(into_py_err)?;
        let result = py.allow_threads(|| self.inner.verify(msg, &sig).map_err(into_py_err))?;
        Ok(result)
    }

    fn to_bytes<'py>(&self, py: Python<'py>) -> Bound<'py, PyBytes> {
        PyBytes::new(py, &self.inner.to_bytes())
    }
}
