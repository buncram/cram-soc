# Cramium SoC

This is the open source RISC-V core on the Cramium SoC.

Compiled documentation: [RV core complex register set](https://ci.betrusted.io/cramium/index.html).

## Structure ##

- The Vex CPU source is located in [./VexRiscv/GenCramSoC.scala](./VexRiscv/GenCramSoC.scala). This is compiled into a [verilog file](./VexRiscv/VexRiscv_CramSoC.v) and included into LiteX via a CPU wrapper.
- The Vex core is wrapped in a custom LiteX CPU wrapper, at [./deps/litex/litex/soc/cores/cpu/vexriscv_axi/core.py](./deps/litex/litex/soc/cores/cpu/vexriscv_axi/core.py). For now, this project relies on a fork of LiteX.
- The CPU is instantiated for SoC integration using [cram_core.py](./cram_core.py). This creates a "SoC"-less LiteX project which wraps the CPU core in an AXI crossbar from the `verilog-axi` project, allowing us to (a) have a private CSR space for RV-only peripherals, and (b) adapt the RV's bus widths to those specified by the Cramium SoC. `verilog-axi` is used because it seems more mature than the AXI primitives in LiteX as of 2023.
- `cram_core.py` will output an artifact named [cram_axi.v](./candidate/cram_axi.v). This is the verilog file as integrated for tape-out. It is instantited with register initializations turned off, since on real silicon you can't pre-load registers with values on boot.
- `cram_soc.py` and `cram_arty.py` are wrappers of `cram_core.py` that put the production SoC core into either a simulation framework (`soc`), or something that can target the Arty FPGA board (`arty`). The main reason we have two targets is so that in simulation we use abstract RAM models that are faster to simulate (and more accurate to the SoC) than the DRAM models used for the Arty implementation. Both of these scripts rely on a common model for simulation inside `cram_common.py`.

## Contribution Guidelines

Please see [CONTRIBUTING](./CONTRIBUTING.md) for details on
how to make a contribution.

Please note that this project is released with a
[Contributor Code of Conduct](./CODE_OF_CONDUCT.md/).
By participating in this project you agree to abide its terms.

## License

Copyright © 2019 - 2023

Licensed under the [CERN OHL v2-W](https://ohwr.org/cern_ohl_w_v2.txt) license.
