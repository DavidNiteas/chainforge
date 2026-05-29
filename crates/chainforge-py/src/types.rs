use chainforge_core::block::BlockHeader;
use chainforge_core::merkle::{MerkleProof, MerkleTree};
use chainforge_core::tx::Transaction;
use chainforge_crypto::keccak256;
use pyo3::prelude::*;
use pyo3::types::PyBytes;

use crate::error::into_py_err;

#[pyclass(name = "MerkleTree")]
pub struct PyMerkleTree {
    inner: MerkleTree,
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
                let siblings: Vec<Bound<'_, PyBytes>> = proof
                    .siblings
                    .iter()
                    .map(|s| PyBytes::new(py, s))
                    .collect();
                dict.set_item("siblings", siblings)?;
                dict.set_item("indices", proof.indices.clone())?;
                Ok(Some(dict.into_pyobject(py).unwrap().into_py(py)))
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
    inner: Transaction,
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
        Py::new(py, PyTransaction { inner: tx }).map(|p| p.into_pyobject(py).unwrap().into_any().unbind())
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
    inner: BlockHeader,
}

#[pymethods]
impl PyBlockHeader {
    #[new]
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
}
