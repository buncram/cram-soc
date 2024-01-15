# Code generators

The Python scripts here are used to generate SVD descriptions of the registers within
the SoC. They will also automatically call svd2rust to generate Rust header files
from the same, although C header files can also be generated from SVD files with
third party programs. The script also attempts to generate an automatic datasheet
based upon the SVD descriptions.

- `daric_to_svd.py` is the full-chip register generator. This is probably the only
  one any end user needs to run.
- `pio_to_svd.py` generates just a set of registers for the PIO block
- `sce_to_svd.py` generates just a set of registers for the SCE block

The latter two scripts were written before `daric_to_svd.py`, and are basically
merged into `daric_to_svd.py`. Thus one may consider the latter two scripts
as primarily historical, although they have value when doing targeted debug
of a specific subsystem.

## Methodology

Reading the Verilog source code of the SoC relies primarily on heuristics to
recognize the APB register set. There are two major banks of regsiters in the
SoC, one uses the "SFR" convention, and the other uses the "UDMA" convention.

Handlers for both can be fund in `daric_to_svd.py`.

A best-effort is made to try and make sense of the register names based upon
the symbols they are bound to in the RTL. Some fix-up will always be required,
and furthermore, the constant values associated with many registers are not
automatically extracted. In this case, developers are referred to the source
code to find out more.