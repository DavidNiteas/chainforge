use chainforge_error::ChainforgeError;
use pyo3::prelude::*;

mod error;
use error::into_py_err;

#[pyfunction]
fn raise_invalid_parameter(msg: String) -> PyResult<()> {
    Err(into_py_err(ChainforgeError::InvalidParameter(msg)))
}

#[pyfunction]
fn raise_crypto(msg: String) -> PyResult<()> {
    Err(into_py_err(ChainforgeError::Crypto(msg)))
}

#[pyfunction]
fn raise_storage(msg: String) -> PyResult<()> {
    Err(into_py_err(ChainforgeError::Storage(msg)))
}

#[pyfunction]
fn raise_serialization(msg: String) -> PyResult<()> {
    Err(into_py_err(ChainforgeError::Serialization(msg)))
}

#[pyfunction]
fn raise_state_root_mismatch(expected: String, actual: String) -> PyResult<()> {
    Err(into_py_err(ChainforgeError::StateRootMismatch { expected, actual }))
}

/// 初始化 chainforge._internal 模块
#[pymodule]
fn _internal(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add("ChainforgeError", m.py().get_type::<pyo3::exceptions::PyRuntimeError>())?;

    m.add_function(wrap_pyfunction!(raise_invalid_parameter, m)?)?;
    m.add_function(wrap_pyfunction!(raise_crypto, m)?)?;
    m.add_function(wrap_pyfunction!(raise_storage, m)?)?;
    m.add_function(wrap_pyfunction!(raise_serialization, m)?)?;
    m.add_function(wrap_pyfunction!(raise_state_root_mismatch, m)?)?;

    Ok(())
}
