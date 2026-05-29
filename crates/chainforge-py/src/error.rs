use chainforge_core::ChainforgeError;
use pyo3::exceptions::{PyRuntimeError, PyValueError};
use pyo3::PyErr;

/// 将 `ChainforgeError` 转换为对应的 Python 异常。
pub fn into_py_err(err: ChainforgeError) -> PyErr {
    match err {
        ChainforgeError::InvalidParameter(_) | ChainforgeError::Serialization(_) => {
            PyValueError::new_err(err.to_string())
        }
        _ => PyRuntimeError::new_err(err.to_string()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pyo3::Python;

    #[test]
    fn test_invalid_parameter_maps_to_value_error() {
        Python::with_gil(|py| {
            let err = ChainforgeError::InvalidParameter("bad arg".to_string());
            let py_err = into_py_err(err);
            assert!(py_err.is_instance_of::<PyValueError>(py));
            assert!(py_err.to_string().contains("bad arg"));
        });
    }

    #[test]
    fn test_crypto_maps_to_runtime_error() {
        Python::with_gil(|py| {
            let err = ChainforgeError::Crypto("hash failed".to_string());
            let py_err = into_py_err(err);
            assert!(py_err.is_instance_of::<PyRuntimeError>(py));
            assert!(py_err.to_string().contains("hash failed"));
        });
    }

    #[test]
    fn test_storage_maps_to_runtime_error() {
        Python::with_gil(|py| {
            let err = ChainforgeError::Storage("db down".to_string());
            let py_err = into_py_err(err);
            assert!(py_err.is_instance_of::<PyRuntimeError>(py));
            assert!(py_err.to_string().contains("db down"));
        });
    }

    #[test]
    fn test_serialization_maps_to_value_error() {
        Python::with_gil(|py| {
            let err = ChainforgeError::Serialization("bad bytes".to_string());
            let py_err = into_py_err(err);
            assert!(py_err.is_instance_of::<PyValueError>(py));
            assert!(py_err.to_string().contains("bad bytes"));
        });
    }

    #[test]
    fn test_state_root_mismatch_maps_to_runtime_error() {
        Python::with_gil(|py| {
            let err = ChainforgeError::StateRootMismatch {
                expected: "0x1".to_string(),
                actual: "0x2".to_string(),
            };
            let py_err = into_py_err(err);
            assert!(py_err.is_instance_of::<PyRuntimeError>(py));
            let msg = py_err.to_string();
            assert!(msg.contains("expected 0x1"));
            assert!(msg.contains("got 0x2"));
        });
    }
}
