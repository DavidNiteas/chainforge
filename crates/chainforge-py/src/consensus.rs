//! PyO3 bindings for chainforge-consensus (HotStuff consensus engine).

use chainforge_consensus::{
    BlockNode, BlockTree, ConsensusEngine, LeaderRotator, Pacemaker, Phase, QuorumCertificate,
    SafetyRules, Vote,
};
use pyo3::prelude::*;
use pyo3::types::PyBytes;

use crate::types::{PyBlock, PyTransaction};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn parse_hash(bytes: &[u8]) -> PyResult<[u8; 32]> {
    bytes
        .try_into()
        .map_err(|_| pyo3::exceptions::PyValueError::new_err("hash must be 32 bytes"))
}

// ---------------------------------------------------------------------------
// Phase
// ---------------------------------------------------------------------------

#[pyclass(name = "Phase", eq, eq_int)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum PyPhase {
    Prepare = 0,
    PreCommit = 1,
    Commit = 2,
    Decide = 3,
}

impl From<Phase> for PyPhase {
    fn from(p: Phase) -> Self {
        match p {
            Phase::Prepare => PyPhase::Prepare,
            Phase::PreCommit => PyPhase::PreCommit,
            Phase::Commit => PyPhase::Commit,
            Phase::Decide => PyPhase::Decide,
        }
    }
}

impl From<PyPhase> for Phase {
    fn from(p: PyPhase) -> Self {
        match p {
            PyPhase::Prepare => Phase::Prepare,
            PyPhase::PreCommit => Phase::PreCommit,
            PyPhase::Commit => Phase::Commit,
            PyPhase::Decide => Phase::Decide,
        }
    }
}

// ---------------------------------------------------------------------------
// Vote
// ---------------------------------------------------------------------------

#[pyclass(name = "Vote")]
pub struct PyVote {
    pub(crate) inner: Vote,
}

#[pymethods]
impl PyVote {
    #[new]
    fn new(block_hash: &[u8], view_number: u64, phase: PyPhase) -> PyResult<Self> {
        let hash = parse_hash(block_hash)?;
        Ok(PyVote {
            inner: Vote {
                block_hash: hash,
                view_number,
                phase: phase.into(),
                voter: chainforge_crypto::ecdsa::PublicKey::from_bytes(&[0u8; 33]).unwrap(),
                signature: chainforge_crypto::ecdsa::Signature::from_bytes(&[0u8; 64], 0).unwrap(),
            },
        })
    }

    #[getter]
    fn block_hash<'py>(&self, py: Python<'py>) -> Bound<'py, PyBytes> {
        PyBytes::new(py, &self.inner.block_hash)
    }

    #[getter]
    fn view_number(&self) -> u64 {
        self.inner.view_number
    }

    #[getter]
    fn phase(&self) -> PyPhase {
        self.inner.phase.into()
    }

    #[getter]
    fn voter<'py>(&self, py: Python<'py>) -> Bound<'py, PyBytes> {
        PyBytes::new(py, &self.inner.voter.to_bytes())
    }

    #[getter]
    fn signature<'py>(&self, py: Python<'py>) -> Bound<'py, PyBytes> {
        PyBytes::new(py, &self.inner.signature.to_bytes())
    }

    #[getter]
    fn recovery_id(&self) -> u8 {
        self.inner.signature.recovery_id()
    }
}

// ---------------------------------------------------------------------------
// QuorumCertificate
// ---------------------------------------------------------------------------

#[pyclass(name = "QuorumCertificate")]
pub struct PyQuorumCertificate {
    pub(crate) inner: QuorumCertificate,
}

#[pymethods]
impl PyQuorumCertificate {
    #[staticmethod]
    fn new(block_hash: &[u8], view_number: u64, phase: PyPhase) -> PyResult<Self> {
        let hash = parse_hash(block_hash)?;
        Ok(PyQuorumCertificate {
            inner: QuorumCertificate::new(hash, view_number, phase.into()),
        })
    }

    #[getter]
    fn block_hash<'py>(&self, py: Python<'py>) -> Bound<'py, PyBytes> {
        PyBytes::new(py, &self.inner.block_hash)
    }

    #[getter]
    fn view_number(&self) -> u64 {
        self.inner.view_number
    }

    #[getter]
    fn phase(&self) -> PyPhase {
        self.inner.phase.into()
    }

    #[getter]
    fn votes<'py>(&self, py: Python<'py>) -> PyResult<Vec<PyObject>> {
        self.inner
            .votes
            .iter()
            .map(|v| {
                Py::new(py, PyVote { inner: v.clone() })
                    .map(|p| p.into_pyobject(py).unwrap().into_any().unbind())
            })
            .collect()
    }

    fn has_quorum(&self, quorum: usize) -> bool {
        self.inner.has_quorum(quorum)
    }

    fn verify(&self) -> bool {
        self.inner.verify(&[])
    }
}

// ---------------------------------------------------------------------------
// BlockNode
// ---------------------------------------------------------------------------

#[pyclass(name = "BlockNode")]
pub struct PyBlockNode {
    pub(crate) inner: BlockNode,
}

#[pymethods]
impl PyBlockNode {
    #[getter]
    fn block<'py>(&self, py: Python<'py>) -> PyResult<PyObject> {
        Py::new(
            py,
            PyBlock {
                inner: self.inner.block.clone(),
            },
        )
        .map(|p| p.into_pyobject(py).unwrap().into_any().unbind())
    }

    #[getter]
    fn parent<'py>(&self, py: Python<'py>) -> Option<Bound<'py, PyBytes>> {
        self.inner.parent.map(|h| PyBytes::new(py, &h))
    }

    #[getter]
    fn prepare_qc<'py>(&self, py: Python<'py>) -> Option<PyObject> {
        self.inner.prepare_qc.clone().map(|qc| {
            Py::new(py, PyQuorumCertificate { inner: qc })
                .map(|p| p.into_pyobject(py).unwrap().into_any().unbind())
                .unwrap()
        })
    }

    #[getter]
    fn precommit_qc<'py>(&self, py: Python<'py>) -> Option<PyObject> {
        self.inner.precommit_qc.clone().map(|qc| {
            Py::new(py, PyQuorumCertificate { inner: qc })
                .map(|p| p.into_pyobject(py).unwrap().into_any().unbind())
                .unwrap()
        })
    }

    #[getter]
    fn commit_qc<'py>(&self, py: Python<'py>) -> Option<PyObject> {
        self.inner.commit_qc.clone().map(|qc| {
            Py::new(py, PyQuorumCertificate { inner: qc })
                .map(|p| p.into_pyobject(py).unwrap().into_any().unbind())
                .unwrap()
        })
    }
}

// ---------------------------------------------------------------------------
// BlockTree
// ---------------------------------------------------------------------------

#[pyclass(name = "BlockTree")]
pub struct PyBlockTree {
    pub(crate) inner: BlockTree,
}

#[pymethods]
impl PyBlockTree {
    #[new]
    fn new() -> Self {
        PyBlockTree {
            inner: BlockTree::new(),
        }
    }

    fn insert(&mut self, block: &Bound<'_, PyBlock>, qc: &Bound<'_, PyQuorumCertificate>) {
        self.inner
            .insert(block.borrow().inner.clone(), qc.borrow().inner.clone());
    }

    fn add_precommit_qc(
        &mut self,
        hash: &[u8],
        qc: &Bound<'_, PyQuorumCertificate>,
    ) -> PyResult<()> {
        let h = parse_hash(hash)?;
        self.inner.add_precommit_qc(&h, qc.borrow().inner.clone());
        Ok(())
    }

    fn add_commit_qc(&mut self, hash: &[u8], qc: &Bound<'_, PyQuorumCertificate>) -> PyResult<()> {
        let h = parse_hash(hash)?;
        self.inner.add_commit_qc(&h, qc.borrow().inner.clone());
        Ok(())
    }

    fn committed_blocks<'py>(&self, py: Python<'py>) -> PyResult<Vec<PyObject>> {
        self.inner
            .committed_blocks()
            .into_iter()
            .map(|b| {
                Py::new(py, PyBlock { inner: b.clone() })
                    .map(|p| p.into_pyobject(py).unwrap().into_any().unbind())
            })
            .collect()
    }

    fn committed_height(&self) -> u64 {
        self.inner.committed_height()
    }

    fn get<'py>(&self, py: Python<'py>, hash: &[u8]) -> PyResult<Option<PyObject>> {
        let h = parse_hash(hash)?;
        Ok(self.inner.get(&h).cloned().map(|node| {
            Py::new(py, PyBlockNode { inner: node })
                .map(|p| p.into_pyobject(py).unwrap().into_any().unbind())
                .unwrap()
        }))
    }

    #[getter]
    fn locked_qc<'py>(&self, py: Python<'py>) -> Option<PyObject> {
        self.inner.locked_qc.clone().map(|qc| {
            Py::new(py, PyQuorumCertificate { inner: qc })
                .map(|p| p.into_pyobject(py).unwrap().into_any().unbind())
                .unwrap()
        })
    }

    #[setter]
    fn set_locked_qc(&mut self, qc: Option<PyRef<'_, PyQuorumCertificate>>) {
        self.inner.locked_qc = qc.map(|q| q.inner.clone());
    }

    #[getter]
    fn view(&self) -> u64 {
        self.inner.view
    }

    #[setter]
    fn set_view(&mut self, view: u64) {
        self.inner.view = view;
    }
}

// ---------------------------------------------------------------------------
// SafetyRules
// ---------------------------------------------------------------------------

#[pyclass(name = "SafetyRules")]
pub struct PySafetyRules {
    pub(crate) inner: SafetyRules,
}

#[pymethods]
impl PySafetyRules {
    #[new]
    fn new() -> Self {
        PySafetyRules {
            inner: SafetyRules::new(),
        }
    }

    #[getter]
    fn locked_view(&self) -> u64 {
        self.inner.locked_view
    }

    #[setter]
    fn set_locked_view(&mut self, view: u64) {
        self.inner.locked_view = view;
    }

    #[getter]
    fn locked_qc<'py>(&self, py: Python<'py>) -> Option<PyObject> {
        self.inner.locked_qc.clone().map(|qc| {
            Py::new(py, PyQuorumCertificate { inner: qc })
                .map(|p| p.into_pyobject(py).unwrap().into_any().unbind())
                .unwrap()
        })
    }

    fn can_vote_prepare(
        &self,
        block: &Bound<'_, PyBlock>,
        high_qc: &Bound<'_, PyQuorumCertificate>,
    ) -> bool {
        self.inner
            .can_vote_prepare(&block.borrow().inner, &high_qc.borrow().inner)
    }

    fn update_locked(&mut self, qc: &Bound<'_, PyQuorumCertificate>) {
        self.inner.update_locked(qc.borrow().inner.clone());
    }

    fn check_double_vote(&self, block_hash: &[u8], phase: PyPhase) -> PyResult<bool> {
        let h = parse_hash(block_hash)?;
        Ok(self.inner.check_double_vote(&h, phase.into()))
    }
}

// ---------------------------------------------------------------------------
// LeaderRotator
// ---------------------------------------------------------------------------

#[pyclass(name = "LeaderRotator")]
pub struct PyLeaderRotator {
    pub(crate) inner: LeaderRotator,
}

#[pymethods]
impl PyLeaderRotator {
    #[new]
    fn new(node_count: usize) -> Self {
        PyLeaderRotator {
            inner: LeaderRotator::new(node_count),
        }
    }

    #[getter]
    fn node_count(&self) -> usize {
        self.inner.node_count
    }

    fn leader_for(&self, view: u64) -> usize {
        self.inner.leader_for(view)
    }
}

// ---------------------------------------------------------------------------
// Pacemaker
// ---------------------------------------------------------------------------

#[pyclass(name = "Pacemaker")]
pub struct PyPacemaker {
    pub(crate) inner: Pacemaker,
}

#[pymethods]
impl PyPacemaker {
    #[new]
    #[pyo3(signature = (node_id, node_count, timeout_ms=5000))]
    fn new(node_id: usize, node_count: usize, timeout_ms: u64) -> Self {
        PyPacemaker {
            inner: Pacemaker::new(node_id, node_count, timeout_ms),
        }
    }

    #[getter]
    fn current_view(&self) -> u64 {
        self.inner.current_view
    }

    #[getter]
    fn node_id(&self) -> usize {
        self.inner.node_id
    }

    #[getter]
    fn timeout_ms(&self) -> u64 {
        self.inner.timeout_ms
    }

    fn is_leader(&self) -> bool {
        self.inner.is_leader()
    }

    fn advance_view(&mut self, view: u64) {
        self.inner.advance_view(view);
    }

    fn current_leader(&self) -> usize {
        self.inner.current_leader()
    }

    fn leader_rotator<'py>(&self, py: Python<'py>) -> PyResult<PyObject> {
        Py::new(
            py,
            PyLeaderRotator {
                inner: self.inner.leader_rotator.clone(),
            },
        )
        .map(|p| p.into_pyobject(py).unwrap().into_any().unbind())
    }
}

// ---------------------------------------------------------------------------
// ConsensusEngine
// ---------------------------------------------------------------------------

#[pyclass(name = "ConsensusEngine")]
pub struct PyConsensusEngine {
    pub(crate) inner: ConsensusEngine,
}

#[pymethods]
impl PyConsensusEngine {
    #[new]
    fn new(node_id: usize, node_count: usize) -> Self {
        PyConsensusEngine {
            inner: ConsensusEngine::new(node_id, node_count),
        }
    }

    #[getter]
    fn node_id(&self) -> usize {
        self.inner.node_id
    }

    fn propose_block<'py>(
        &self,
        py: Python<'py>,
        parent_hash: &[u8],
        number: u64,
        txs: Vec<Py<PyTransaction>>,
        high_qc: &Bound<'_, PyQuorumCertificate>,
    ) -> PyResult<Bound<'py, PyAny>> {
        let hash = parse_hash(parent_hash)?;
        let rust_txs: Vec<chainforge_core::tx::Transaction> =
            txs.iter().map(|t| t.borrow(py).inner.clone()).collect();
        let block =
            self.inner
                .propose_block(hash, number, rust_txs, high_qc.borrow().inner.clone());
        Py::new(py, PyBlock { inner: block }).map(|p| p.into_pyobject(py).unwrap().into_any())
    }

    fn vote_prepare(
        &mut self,
        block: &Bound<'_, PyBlock>,
        high_qc: &Bound<'_, PyQuorumCertificate>,
    ) -> Option<PyVote> {
        self.inner
            .vote_prepare(&block.borrow().inner, &high_qc.borrow().inner)
            .map(|v| PyVote { inner: v })
    }

    fn form_qc(
        &self,
        votes: Vec<Py<PyVote>>,
        phase: PyPhase,
        quorum: usize,
    ) -> Option<PyQuorumCertificate> {
        let rust_votes: Vec<Vote> =
            Python::with_gil(|py| votes.iter().map(|v| v.borrow(py).inner.clone()).collect());
        self.inner
            .form_qc(rust_votes, phase.into(), quorum)
            .map(|qc| PyQuorumCertificate { inner: qc })
    }

    fn on_prepare_qc(&mut self, block: &Bound<'_, PyBlock>, qc: &Bound<'_, PyQuorumCertificate>) {
        self.inner
            .on_prepare_qc(block.borrow().inner.clone(), qc.borrow().inner.clone());
    }

    fn on_precommit_qc(
        &mut self,
        hash: &[u8],
        qc: &Bound<'_, PyQuorumCertificate>,
    ) -> PyResult<()> {
        let h = parse_hash(hash)?;
        self.inner.on_precommit_qc(&h, qc.borrow().inner.clone());
        Ok(())
    }

    fn on_commit_qc(&mut self, hash: &[u8], qc: &Bound<'_, PyQuorumCertificate>) -> PyResult<()> {
        let h = parse_hash(hash)?;
        self.inner.on_commit_qc(&h, qc.borrow().inner.clone());
        Ok(())
    }

    fn advance_view(&mut self, view: u64) {
        self.inner.advance_view(view);
    }

    fn block_tree<'py>(&self, py: Python<'py>) -> PyResult<PyObject> {
        Py::new(
            py,
            PyBlockTree {
                inner: self.inner.block_tree.clone(),
            },
        )
        .map(|p| p.into_pyobject(py).unwrap().into_any().unbind())
    }

    fn safety<'py>(&self, py: Python<'py>) -> PyResult<PyObject> {
        Py::new(
            py,
            PySafetyRules {
                inner: self.inner.safety.clone(),
            },
        )
        .map(|p| p.into_pyobject(py).unwrap().into_any().unbind())
    }

    fn pacemaker<'py>(&self, py: Python<'py>) -> PyResult<PyObject> {
        Py::new(
            py,
            PyPacemaker {
                inner: self.inner.pacemaker.clone(),
            },
        )
        .map(|p| p.into_pyobject(py).unwrap().into_any().unbind())
    }
}
