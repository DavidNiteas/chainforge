use pyo3::prelude::*;

/// 初始化 chainforge._internal 模块
#[pymodule]
fn _internal(_m: &Bound<'_, PyModule>) -> PyResult<()> {
    Ok(())
}
