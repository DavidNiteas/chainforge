use kilnchain_error::KilnchainError;
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
    KilnchainCryptoError, KilnchainRuntimeError, KilnchainStateError, KilnchainStorageError,
    KilnchainValueError,
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
    Err(crate::error::into_py_err(KilnchainError::InvalidParameter(
        msg,
    )))
}

#[pyfunction]
fn raise_crypto(msg: String) -> PyResult<()> {
    Err(crate::error::into_py_err(KilnchainError::Crypto(msg)))
}

#[pyfunction]
fn raise_storage(msg: String) -> PyResult<()> {
    Err(crate::error::into_py_err(KilnchainError::Storage(msg)))
}

#[pyfunction]
fn raise_serialization(msg: String) -> PyResult<()> {
    Err(crate::error::into_py_err(KilnchainError::Serialization(
        msg,
    )))
}

#[pyfunction]
fn raise_state_root_mismatch(expected: String, actual: String) -> PyResult<()> {
    Err(crate::error::into_py_err(
        KilnchainError::StateRootMismatch { expected, actual },
    ))
}

#[pyfunction]
fn keccak256(data: &[u8]) -> [u8; 32] {
    kilnchain_crypto::keccak256(data)
}

/// 初始化 kilnchain._internal 模块
#[pymodule]
fn _internal(m: &Bound<'_, PyModule>) -> PyResult<()> {
    // 异常类型
    m.add(
        "KilnchainError",
        m.py().get_type::<pyo3::exceptions::PyRuntimeError>(),
    )?;
    m.add(
        "KilnchainValueError",
        m.py().get_type::<KilnchainValueError>(),
    )?;
    m.add(
        "KilnchainCryptoError",
        m.py().get_type::<KilnchainCryptoError>(),
    )?;
    m.add(
        "KilnchainStorageError",
        m.py().get_type::<KilnchainStorageError>(),
    )?;
    m.add(
        "KilnchainStateError",
        m.py().get_type::<KilnchainStateError>(),
    )?;
    m.add(
        "KilnchainRuntimeError",
        m.py().get_type::<KilnchainRuntimeError>(),
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
