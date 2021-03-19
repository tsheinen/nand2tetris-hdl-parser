# nand2tetris hdl parser (but its a python wrapper)

## install

I used [maturin](https://github.com/PyO3/maturin) to build this.  the process goes something like this:

```text
pip install maturin
venv dev # macro i have to select virtualenvs -- you'll want to be in one because it'll refuse to install otherwise
git checkout pyo3
maturin develop
```

you'll need Rust installed to build it -- it's pretty straightforward using [rustup](https://rustup.rs/) but if this is problematic hit me up and i'll build it for you.   

## use

```python
from nand2tetris_hdl_parser import parse_hdl
hdl = open("example.hdl","r").read()
parse_hdl(hdl)
```
