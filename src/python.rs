use crate::{Chip, HDLParseError};
use pyo3::create_exception;
use pyo3::prelude::*;

create_exception!(
    nand2tetris_hdl_parser,
    PyHDLParseError,
    pyo3::exceptions::PyException
);

// struct HDLParserErrorWrapper {
//     details: String,
// }

impl From<HDLParseError> for PyErr {
    fn from(error: HDLParseError) -> Self {
        PyHDLParseError::new_err(error.details)
    }
}


#[pymodule]
fn nand2tetris_hdl_parser(py: Python, m: &PyModule) -> PyResult<()> {
    #[pyfn(m, "parse_hdl")]
    pub fn parse_hdl_python(hdl: String) -> PyResult<Chip> {
        crate::parse_hdl(&hdl).map_err(|x| x.into())
    }

    // m.add_function(wrap_pyfunction!(parse_hdl, m)?)?;
    m.add("HDLParseError", py.get_type::<PyHDLParseError>())?;
    Ok(())
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
