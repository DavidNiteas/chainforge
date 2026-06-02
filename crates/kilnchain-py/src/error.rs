use kilnchain_error::KilnchainError;
use pyo3::exceptions::{PyRuntimeError, PyValueError};
use pyo3::PyErr;

/// 将 `KilnchainError` 转换为细化的 Python 异常。
///
/// 映射规则：
/// - InvalidParameter / Serialization → KilnchainValueError (ValueError 子类)
/// - Crypto → KilnchainCryptoError (RuntimeError 子类)
/// - Storage → KilnchainStorageError (RuntimeError 子类)
/// - StateRootMismatch → KilnchainStateError (RuntimeError 子类)
/// - 其他 → KilnchainRuntimeError (RuntimeError 子类)
pub fn into_py_err(err: KilnchainError) -> PyErr {
    match err {
        KilnchainError::InvalidParameter(_) | KilnchainError::Serialization(_) => {
            KilnchainValueError::new_err(err.to_string())
        }
        KilnchainError::Crypto(_) => KilnchainCryptoError::new_err(err.to_string()),
        KilnchainError::Storage(_) => KilnchainStorageError::new_err(err.to_string()),
        KilnchainError::StateRootMismatch { .. } => KilnchainStateError::new_err(err.to_string()),
    }
}

// 自定义异常类型（细化映射）
pyo3::create_exception!(kilnchain._internal, KilnchainValueError, PyValueError);
pyo3::create_exception!(kilnchain._internal, KilnchainCryptoError, PyRuntimeError);
pyo3::create_exception!(kilnchain._internal, KilnchainStorageError, PyRuntimeError);
pyo3::create_exception!(kilnchain._internal, KilnchainStateError, PyRuntimeError);
pyo3::create_exception!(kilnchain._internal, KilnchainRuntimeError, PyRuntimeError);

#[cfg(test)]
mod tests {
    use super::*;
    use pyo3::Python;

    #[test]
    fn test_invalid_parameter_maps_to_value_error() {
        Python::with_gil(|py| {
            let err = KilnchainError::InvalidParameter("bad arg".to_string());
            let py_err = into_py_err(err);
            assert!(py_err.is_instance_of::<PyValueError>(py));
            assert!(py_err.to_string().contains("bad arg"));
        });
    }

    #[test]
    fn test_crypto_maps_to_runtime_error() {
        Python::with_gil(|py| {
            let err = KilnchainError::Crypto("hash failed".to_string());
            let py_err = into_py_err(err);
            assert!(py_err.is_instance_of::<PyRuntimeError>(py));
            assert!(py_err.to_string().contains("hash failed"));
        });
    }

    #[test]
    fn test_storage_maps_to_runtime_error() {
        Python::with_gil(|py| {
            let err = KilnchainError::Storage("db down".to_string());
            let py_err = into_py_err(err);
            assert!(py_err.is_instance_of::<PyRuntimeError>(py));
            assert!(py_err.to_string().contains("db down"));
        });
    }

    #[test]
    fn test_serialization_maps_to_value_error() {
        Python::with_gil(|py| {
            let err = KilnchainError::Serialization("bad bytes".to_string());
            let py_err = into_py_err(err);
            assert!(py_err.is_instance_of::<PyValueError>(py));
            assert!(py_err.to_string().contains("bad bytes"));
        });
    }

    #[test]
    fn test_state_root_mismatch_maps_to_runtime_error() {
        Python::with_gil(|py| {
            let err = KilnchainError::StateRootMismatch {
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
