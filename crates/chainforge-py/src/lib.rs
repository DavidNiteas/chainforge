use chainforge_error::ChainforgeError;
use pyo3::prelude::*;

mod crypto;
mod error;
mod storage;
mod types;

use crypto::{PyPublicKey, PySecretKey};
use storage::PyInMemoryStorage;
use types::{PyBlockHeader, PyMerkleTree, PyTransaction};

#[pyfunction]
fn raise_invalid_parameter(msg: String) -> PyResult<()> {
    Err(crate::error::into_py_err(ChainforgeError::InvalidParameter(msg)))
}

#[pyfunction]
fn raise_crypto(msg: String) -> PyResult<()> {
    Err(crate::error::into_py_err(ChainforgeError::Crypto(msg)))
}

#[pyfunction]
fn raise_storage(msg: String) -> PyResult<()> {
    Err(crate::error::into_py_err(ChainforgeError::Storage(msg)))
}

#[pyfunction]
fn raise_serialization(msg: String) -> PyResult<()> {
    Err(crate::error::into_py_err(ChainforgeError::Serialization(msg)))
}

#[pyfunction]
fn raise_state_root_mismatch(expected: String, actual: String) -> PyResult<()> {
    Err(crate::error::into_py_err(ChainforgeError::StateRootMismatch { expected, actual }))
}

#[pyfunction]
fn keccak256(data: &[u8]) -> [u8; 32] {
    chainforge_crypto::keccak256(data)
}

/// 初始化 chainforge._internal 模块
#[pymodule]
fn _internal(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add("ChainforgeError", m.py().get_type::<pyo3::exceptions::PyRuntimeError>())?;

    m.add_class::<PyMerkleTree>()?;
    m.add_class::<PyTransaction>()?;
    m.add_class::<PyBlockHeader>()?;
    m.add_class::<PySecretKey>()?;
    m.add_class::<PyPublicKey>()?;
    m.add_class::<PyInMemoryStorage>()?;

    m.add_function(wrap_pyfunction!(raise_invalid_parameter, m)?)?;
    m.add_function(wrap_pyfunction!(raise_crypto, m)?)?;
    m.add_function(wrap_pyfunction!(raise_storage, m)?)?;
    m.add_function(wrap_pyfunction!(raise_serialization, m)?)?;
    m.add_function(wrap_pyfunction!(raise_state_root_mismatch, m)?)?;
    m.add_function(wrap_pyfunction!(keccak256, m)?)?;

    Ok(())
}
