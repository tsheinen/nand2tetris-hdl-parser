# nand2tetris hdl parser (but its a python wrapper)

## install

download a wheel from the releases and install it. I haven't added anything to pypi yet

## use

```python
from nand2tetris_hdl_parser import parse_hdl
hdl = open("example.hdl","r").read()
parse_hdl(hdl)
```
