use chainforge_evm::state::InMemoryEvmState;
use chainforge_evm::EvmExecutor;
use pyo3::prelude::*;
use pyo3::types::PyBytes;
use revm::primitives::{Address, ExecutionResult, Output, U256};

use crate::error::into_py_err;

fn u128_to_u256(v: u128) -> U256 {
    U256::from(v)
}

#[pyclass(name = "EvmState")]
pub struct PyEvmState {
    inner: InMemoryEvmState,
}

#[pymethods]
impl PyEvmState {
    #[new]
    fn new() -> Self {
        PyEvmState {
            inner: InMemoryEvmState::new(),
        }
    }

    fn set_balance(&mut self, address: [u8; 20], balance: u128) {
        self.inner
            .set_balance(Address::new(address), u128_to_u256(balance));
    }

    fn set_nonce(&mut self, address: [u8; 20], nonce: u64) {
        self.inner.set_nonce(Address::new(address), nonce);
    }

    fn set_code(&mut self, address: [u8; 20], code: Vec<u8>) {
        self.inner.set_code(
            Address::new(address),
            revm::primitives::Bytecode::new_raw(code.into()),
        );
    }

    fn set_storage(&mut self, address: [u8; 20], slot: u128, value: u128) {
        self.inner.set_storage(
            Address::new(address),
            u128_to_u256(slot),
            u128_to_u256(value),
        );
    }

    fn balance(&self, address: [u8; 20]) -> u128 {
        self.inner
            .balance(Address::new(address))
            .try_into()
            .unwrap_or(u128::MAX)
    }

    fn nonce(&self, address: [u8; 20]) -> u64 {
        self.inner.nonce(Address::new(address))
    }

    fn code<'py>(&self, py: Python<'py>, address: [u8; 20]) -> Bound<'py, PyBytes> {
        PyBytes::new(py, &self.inner.code(Address::new(address)))
    }
}

#[pyclass(name = "ExecutionResult")]
pub struct PyExecutionResult {
    inner: ExecutionResult,
}

#[pymethods]
impl PyExecutionResult {
    fn is_success(&self) -> bool {
        matches!(self.inner, ExecutionResult::Success { .. })
    }

    fn is_revert(&self) -> bool {
        matches!(self.inner, ExecutionResult::Revert { .. })
    }

    fn is_halt(&self) -> bool {
        matches!(self.inner, ExecutionResult::Halt { .. })
    }

    fn gas_used(&self) -> u64 {
        match &self.inner {
            ExecutionResult::Success { gas_used, .. } => *gas_used,
            ExecutionResult::Revert { gas_used, .. } => *gas_used,
            ExecutionResult::Halt { gas_used, .. } => *gas_used,
        }
    }

    fn output<'py>(&self, py: Python<'py>) -> Option<Bound<'py, PyBytes>> {
        match &self.inner {
            ExecutionResult::Success { output, .. } => match output {
                Output::Call(bytes) => Some(PyBytes::new(py, bytes.as_ref())),
                Output::Create(bytes, _) => Some(PyBytes::new(py, bytes.as_ref())),
            },
            ExecutionResult::Revert { output, .. } => Some(PyBytes::new(py, output.as_ref())),
            ExecutionResult::Halt { .. } => None,
        }
    }

    fn contract_address<'py>(&self, py: Python<'py>) -> Option<Bound<'py, PyBytes>> {
        match &self.inner {
            ExecutionResult::Success {
                output: Output::Create(_, addr),
                ..
            } => addr.map(|a| PyBytes::new(py, a.as_slice())),
            _ => None,
        }
    }
}

#[pyclass(name = "EvmExecutor")]
pub struct PyEvmExecutor {
    inner: EvmExecutor<InMemoryEvmState>,
}

#[pymethods]
impl PyEvmExecutor {
    #[new]
    fn new(state: &Bound<'_, PyEvmState>) -> Self {
        PyEvmExecutor {
            inner: EvmExecutor::new(state.borrow().inner.clone()),
        }
    }

    fn transfer(
        &mut self,
        py: Python,
        from: [u8; 20],
        to: [u8; 20],
        value: u128,
    ) -> PyResult<PyObject> {
        let result = self
            .inner
            .transfer(Address::new(from), Address::new(to), u128_to_u256(value))
            .map_err(into_py_err)?;
        Py::new(py, PyExecutionResult { inner: result })
            .map(|p| p.into_pyobject(py).unwrap().into_any().unbind())
    }

    fn deploy(
        &mut self,
        py: Python,
        from: [u8; 20],
        code: Vec<u8>,
        value: u128,
    ) -> PyResult<PyObject> {
        let result = self
            .inner
            .deploy(Address::new(from), code, u128_to_u256(value))
            .map_err(into_py_err)?;
        Py::new(py, PyExecutionResult { inner: result })
            .map(|p| p.into_pyobject(py).unwrap().into_any().unbind())
    }

    fn call(
        &mut self,
        py: Python,
        from: [u8; 20],
        to: [u8; 20],
        data: Vec<u8>,
        value: u128,
    ) -> PyResult<PyObject> {
        let result = self
            .inner
            .call(
                Address::new(from),
                Address::new(to),
                data,
                u128_to_u256(value),
            )
            .map_err(into_py_err)?;
        Py::new(py, PyExecutionResult { inner: result })
            .map(|p| p.into_pyobject(py).unwrap().into_any().unbind())
    }

    fn balance(&self, address: [u8; 20]) -> u128 {
        self.inner
            .balance(Address::new(address))
            .try_into()
            .unwrap_or(u128::MAX)
    }

    fn nonce(&self, address: [u8; 20]) -> u64 {
        self.inner.nonce(Address::new(address))
    }
}
