use kilnchain_core::block::{Block, BlockHeader};
use kilnchain_core::light_client::LightClient;
use kilnchain_core::merkle::{MerkleProof, MerkleTree};
use kilnchain_core::mpt::MptProof;
use kilnchain_core::tx::Transaction;
use kilnchain_crypto::keccak256;
use pyo3::prelude::*;
use pyo3::types::PyBytes;

use crate::error::into_py_err;

#[pyclass(name = "MerkleTree")]
pub struct PyMerkleTree {
    pub(crate) inner: MerkleTree,
}

#[pymethods]
impl PyMerkleTree {
    #[new]
    fn new(leaves: Vec<Vec<u8>>) -> Self {
        let leaves: Vec<[u8; 32]> = leaves.into_iter().map(|v| keccak256(&v)).collect();
        PyMerkleTree {
            inner: MerkleTree::new(leaves),
        }
    }

    fn root<'py>(&self, py: Python<'py>) -> Bound<'py, PyBytes> {
        PyBytes::new(py, &self.inner.root())
    }

    fn proof<'py>(&self, py: Python<'py>, index: usize) -> PyResult<Option<PyObject>> {
        match self.inner.proof(index) {
            Some(proof) => {
                let dict = pyo3::types::PyDict::new(py);
                let siblings: Vec<Bound<'_, PyBytes>> =
                    proof.siblings.iter().map(|s| PyBytes::new(py, s)).collect();
                dict.set_item("siblings", siblings)?;
                dict.set_item("indices", proof.indices.clone())?;
                Ok(Some(dict.into_pyobject(py).unwrap().into_any().unbind()))
            }
            None => Ok(None),
        }
    }

    #[staticmethod]
    fn verify<'py>(
        _py: Python<'py>,
        root: &[u8],
        leaf: &[u8],
        proof_dict: &Bound<'py, pyo3::types::PyDict>,
    ) -> PyResult<bool> {
        let siblings = proof_dict.get_item("siblings")?.unwrap();
        let siblings: Vec<Vec<u8>> = siblings.extract()?;
        let siblings: Vec<[u8; 32]> = siblings
            .into_iter()
            .map(|v| {
                let mut arr = [0u8; 32];
                arr.copy_from_slice(&v);
                arr
            })
            .collect();
        let indices: Vec<bool> = proof_dict.get_item("indices")?.unwrap().extract()?;

        let mut root_arr = [0u8; 32];
        root_arr.copy_from_slice(root);
        let mut leaf_arr = [0u8; 32];
        leaf_arr.copy_from_slice(leaf);

        Ok(MerkleTree::verify(
            &root_arr,
            &leaf_arr,
            &MerkleProof { siblings, indices },
        ))
    }
}

#[pyclass(name = "Transaction")]
pub struct PyTransaction {
    pub(crate) inner: Transaction,
}

#[pymethods]
impl PyTransaction {
    #[new]
    #[pyo3(signature = (nonce=0, gas_price=0, gas_limit=0, to=None, value=0, data=Vec::new()))]
    fn new(
        nonce: u64,
        gas_price: u128,
        gas_limit: u64,
        to: Option<[u8; 20]>,
        value: u128,
        data: Vec<u8>,
    ) -> Self {
        PyTransaction {
            inner: Transaction {
                nonce,
                gas_price,
                gas_limit,
                to,
                value,
                data,
                v: 0,
                r: [0u8; 32],
                s: [0u8; 32],
            },
        }
    }

    fn hash<'py>(&self, py: Python<'py>) -> Bound<'py, PyBytes> {
        PyBytes::new(py, &self.inner.hash())
    }

    fn encode_rlp<'py>(&self, py: Python<'py>) -> Bound<'py, PyBytes> {
        PyBytes::new(py, &self.inner.encode_rlp())
    }

    #[staticmethod]
    fn decode_rlp(py: Python, data: &[u8]) -> PyResult<PyObject> {
        let tx = Transaction::decode_rlp(data).map_err(into_py_err)?;
        Py::new(py, PyTransaction { inner: tx })
            .map(|p| p.into_pyobject(py).unwrap().into_any().unbind())
    }

    #[getter]
    fn nonce(&self) -> u64 {
        self.inner.nonce
    }

    #[getter]
    fn gas_price(&self) -> u128 {
        self.inner.gas_price
    }

    #[getter]
    fn gas_limit(&self) -> u64 {
        self.inner.gas_limit
    }

    #[getter]
    fn value(&self) -> u128 {
        self.inner.value
    }

    #[getter]
    fn data<'py>(&self, py: Python<'py>) -> Bound<'py, PyBytes> {
        PyBytes::new(py, &self.inner.data)
    }
}

#[pyclass(name = "BlockHeader")]
pub struct PyBlockHeader {
    pub(crate) inner: BlockHeader,
}

#[pymethods]
impl PyBlockHeader {
    #[new]
    #[allow(clippy::too_many_arguments)]
    fn new(
        parent_hash: [u8; 32],
        number: u64,
        timestamp: u64,
        difficulty: u64,
        nonce: u64,
        extra_data: Vec<u8>,
        state_root: [u8; 32],
        txs_root: [u8; 32],
    ) -> PyResult<Self> {
        if extra_data.len() > 32 {
            return Err(pyo3::exceptions::PyValueError::new_err(
                "extra_data exceeds 32 bytes",
            ));
        }
        Ok(PyBlockHeader {
            inner: BlockHeader {
                parent_hash,
                number,
                timestamp,
                difficulty,
                nonce,
                extra_data,
                state_root,
                txs_root,
            },
        })
    }

    fn hash<'py>(&self, py: Python<'py>) -> Bound<'py, PyBytes> {
        PyBytes::new(py, &self.inner.hash())
    }

    #[getter]
    fn number(&self) -> u64 {
        self.inner.number
    }

    #[getter]
    fn timestamp(&self) -> u64 {
        self.inner.timestamp
    }

    #[getter]
    fn difficulty(&self) -> u64 {
        self.inner.difficulty
    }

    #[getter]
    fn nonce(&self) -> u64 {
        self.inner.nonce
    }

    #[getter]
    fn extra_data<'py>(&self, py: Python<'py>) -> Bound<'py, PyBytes> {
        PyBytes::new(py, &self.inner.extra_data)
    }

    #[getter]
    fn state_root<'py>(&self, py: Python<'py>) -> Bound<'py, PyBytes> {
        PyBytes::new(py, &self.inner.state_root)
    }

    #[getter]
    fn txs_root<'py>(&self, py: Python<'py>) -> Bound<'py, PyBytes> {
        PyBytes::new(py, &self.inner.txs_root)
    }

    #[getter]
    fn parent_hash<'py>(&self, py: Python<'py>) -> Bound<'py, PyBytes> {
        PyBytes::new(py, &self.inner.parent_hash)
    }
}

#[pyclass(name = "Block")]
pub struct PyBlock {
    pub(crate) inner: Block,
}

#[pymethods]
impl PyBlock {
    #[new]
    #[allow(clippy::too_many_arguments)]
    fn new(
        header: &Bound<'_, PyBlockHeader>,
        transactions: Vec<PyRef<'_, PyTransaction>>,
        uncle_headers: Vec<PyRef<'_, PyBlockHeader>>,
    ) -> Self {
        let header = header.borrow().inner.clone();
        let transactions = transactions.into_iter().map(|t| t.inner.clone()).collect();
        let uncle_headers = uncle_headers.into_iter().map(|h| h.inner.clone()).collect();
        PyBlock {
            inner: Block {
                header,
                transactions,
                uncle_headers,
            },
        }
    }

    fn hash<'py>(&self, py: Python<'py>) -> Bound<'py, PyBytes> {
        PyBytes::new(py, &self.inner.header.hash())
    }

    fn to_rlp<'py>(&self, py: Python<'py>) -> Bound<'py, PyBytes> {
        PyBytes::new(py, &self.inner.encode_rlp())
    }

    #[staticmethod]
    fn from_rlp(py: Python, data: &[u8]) -> PyResult<PyObject> {
        let block = Block::decode_rlp(data).map_err(into_py_err)?;
        Py::new(py, PyBlock { inner: block })
            .map(|p| p.into_pyobject(py).unwrap().into_any().unbind())
    }

    #[getter]
    fn header<'py>(&self, py: Python<'py>) -> PyResult<PyObject> {
        Py::new(
            py,
            PyBlockHeader {
                inner: self.inner.header.clone(),
            },
        )
        .map(|p| p.into_pyobject(py).unwrap().into_any().unbind())
    }

    #[getter]
    fn transactions<'py>(&self, py: Python<'py>) -> PyResult<Vec<PyObject>> {
        self.inner
            .transactions
            .iter()
            .map(|tx| {
                Py::new(py, PyTransaction { inner: tx.clone() })
                    .map(|p| p.into_pyobject(py).unwrap().into_any().unbind())
            })
            .collect()
    }

    fn compute_txs_root(&mut self) {
        self.inner.compute_txs_root();
    }
}

#[pyclass(name = "LightClient")]
pub struct PyLightClient {
    pub(crate) inner: LightClient,
}

#[pymethods]
impl PyLightClient {
    #[new]
    fn new(genesis: &Bound<'_, PyBlockHeader>) -> Self {
        PyLightClient {
            inner: LightClient::new(genesis.borrow().inner.clone()),
        }
    }

    fn add_header(&mut self, header: &Bound<'_, PyBlockHeader>) -> PyResult<()> {
        self.inner
            .add_header(header.borrow().inner.clone())
            .map_err(into_py_err)
    }

    fn latest_header<'py>(&self, py: Python<'py>) -> PyResult<PyObject> {
        Py::new(
            py,
            PyBlockHeader {
                inner: self.inner.latest_header().clone(),
            },
        )
        .map(|p| p.into_pyobject(py).unwrap().into_any().unbind())
    }

    fn get_header_by_number<'py>(
        &self,
        py: Python<'py>,
        number: u64,
    ) -> PyResult<Option<PyObject>> {
        match self.inner.get_header_by_number(number) {
            Some(h) => Py::new(py, PyBlockHeader { inner: h.clone() })
                .map(|p| Some(p.into_pyobject(py).unwrap().into_any().unbind())),
            None => Ok(None),
        }
    }

    fn len(&self) -> usize {
        self.inner.len()
    }

    fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    fn verify_transaction(
        &self,
        block_number: u64,
        tx_hash: &[u8],
        proof_dict: &Bound<'_, pyo3::types::PyDict>,
    ) -> PyResult<bool> {
        let mut tx_hash_arr = [0u8; 32];
        tx_hash_arr.copy_from_slice(tx_hash);
        let siblings = proof_dict.get_item("siblings")?.unwrap();
        let siblings: Vec<Vec<u8>> = siblings.extract()?;
        let siblings: Vec<[u8; 32]> = siblings
            .into_iter()
            .map(|v| {
                let mut arr = [0u8; 32];
                arr.copy_from_slice(&v);
                arr
            })
            .collect();
        let indices: Vec<bool> = proof_dict.get_item("indices")?.unwrap().extract()?;
        let proof = MerkleProof { siblings, indices };
        self.inner
            .verify_transaction(block_number, &tx_hash_arr, &proof)
            .map_err(into_py_err)
    }
}

#[pyclass(name = "MptProof")]
pub struct PyMptProof {
    pub(crate) inner: MptProof,
}

#[pymethods]
impl PyMptProof {
    #[new]
    fn new(key: Vec<u8>, proof_nodes: Vec<Vec<u8>>) -> Self {
        PyMptProof {
            inner: MptProof { key, proof_nodes },
        }
    }

    fn verify(&self, root: &[u8]) -> PyResult<Option<Vec<u8>>> {
        let mut root_arr = [0u8; 32];
        root_arr.copy_from_slice(root);
        self.inner.verify(&root_arr).map_err(into_py_err)
    }

    #[getter]
    fn key<'py>(&self, py: Python<'py>) -> Bound<'py, PyBytes> {
        PyBytes::new(py, &self.inner.key)
    }

    #[getter]
    fn proof_nodes<'py>(&self, py: Python<'py>) -> Vec<Bound<'py, PyBytes>> {
        self.inner
            .proof_nodes
            .iter()
            .map(|n| PyBytes::new(py, n))
            .collect()
    }
}
