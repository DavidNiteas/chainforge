use chainforge_error::ChainforgeError;
use pyo3::prelude::*;

mod block_producer;
mod consensus;
mod crypto;
mod error;
mod evm;
mod mempool;
mod p2p;
mod storage;
mod types;

use block_producer::PyBlockBuilder;
use consensus::{
    PyBlockNode, PyBlockTree, PyConsensusEngine, PyLeaderRotator, PyPacemaker, PyPhase,
    PyQuorumCertificate, PySafetyRules, PyVote,
};
use crypto::{PyPublicKey, PySecretKey};
use error::{
    ChainforgeCryptoError, ChainforgeRuntimeError, ChainforgeStateError, ChainforgeStorageError,
    ChainforgeValueError,
};
use evm::{PyEvmExecutor, PyEvmState, PyExecutionResult};
use mempool::PyMempool;
use p2p::{PyMessage, PyNode, PyNodeConfig, PyPeerId, PyPeerInfo, PyRoutingTable};
use storage::{PyCachedStorage, PyInMemoryStorage};
use types::{PyBlock, PyBlockHeader, PyLightClient, PyMerkleTree, PyMptProof, PyTransaction};

#[cfg(feature = "rocksdb-backend")]
use storage::PyRocksDBEngine;

#[pyfunction]
fn raise_invalid_parameter(msg: String) -> PyResult<()> {
    Err(crate::error::into_py_err(
        ChainforgeError::InvalidParameter(msg),
    ))
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
    Err(crate::error::into_py_err(ChainforgeError::Serialization(
        msg,
    )))
}

#[pyfunction]
fn raise_state_root_mismatch(expected: String, actual: String) -> PyResult<()> {
    Err(crate::error::into_py_err(
        ChainforgeError::StateRootMismatch { expected, actual },
    ))
}

#[pyfunction]
fn keccak256(data: &[u8]) -> [u8; 32] {
    chainforge_crypto::keccak256(data)
}

/// 初始化 chainforge._internal 模块
#[pymodule]
fn _internal(m: &Bound<'_, PyModule>) -> PyResult<()> {
    // 异常类型
    m.add(
        "ChainforgeError",
        m.py().get_type::<pyo3::exceptions::PyRuntimeError>(),
    )?;
    m.add(
        "ChainforgeValueError",
        m.py().get_type::<ChainforgeValueError>(),
    )?;
    m.add(
        "ChainforgeCryptoError",
        m.py().get_type::<ChainforgeCryptoError>(),
    )?;
    m.add(
        "ChainforgeStorageError",
        m.py().get_type::<ChainforgeStorageError>(),
    )?;
    m.add(
        "ChainforgeStateError",
        m.py().get_type::<ChainforgeStateError>(),
    )?;
    m.add(
        "ChainforgeRuntimeError",
        m.py().get_type::<ChainforgeRuntimeError>(),
    )?;

    // 类
    m.add_class::<PyMerkleTree>()?;
    m.add_class::<PyTransaction>()?;
    m.add_class::<PyBlockHeader>()?;
    m.add_class::<PyBlock>()?;
    m.add_class::<PyLightClient>()?;
    m.add_class::<PyMptProof>()?;
    m.add_class::<PySecretKey>()?;
    m.add_class::<PyPublicKey>()?;
    m.add_class::<PyInMemoryStorage>()?;
    m.add_class::<PyCachedStorage>()?;
    m.add_class::<PyMempool>()?;
    m.add_class::<PyBlockBuilder>()?;
    m.add_class::<PyEvmState>()?;
    m.add_class::<PyEvmExecutor>()?;
    m.add_class::<PyExecutionResult>()?;

    // Consensus
    m.add_class::<PyPhase>()?;
    m.add_class::<PyVote>()?;
    m.add_class::<PyQuorumCertificate>()?;
    m.add_class::<PyBlockNode>()?;
    m.add_class::<PyBlockTree>()?;
    m.add_class::<PySafetyRules>()?;
    m.add_class::<PyLeaderRotator>()?;
    m.add_class::<PyPacemaker>()?;
    m.add_class::<PyConsensusEngine>()?;

    // P2P
    m.add_class::<PyPeerId>()?;
    m.add_class::<PyPeerInfo>()?;
    m.add_class::<PyMessage>()?;
    m.add_class::<PyNodeConfig>()?;
    m.add_class::<PyRoutingTable>()?;
    m.add_class::<PyNode>()?;

    #[cfg(feature = "rocksdb-backend")]
    m.add_class::<PyRocksDBEngine>()?;

    // 函数
    m.add_function(wrap_pyfunction!(raise_invalid_parameter, m)?)?;
    m.add_function(wrap_pyfunction!(raise_crypto, m)?)?;
    m.add_function(wrap_pyfunction!(raise_storage, m)?)?;
    m.add_function(wrap_pyfunction!(raise_serialization, m)?)?;
    m.add_function(wrap_pyfunction!(raise_state_root_mismatch, m)?)?;
    m.add_function(wrap_pyfunction!(keccak256, m)?)?;
    m.add_function(wrap_pyfunction!(crypto::sha256, m)?)?;
    m.add_function(wrap_pyfunction!(crypto::ripemd160, m)?)?;

    Ok(())
}
