# Cramium SoC

This repo contains is the open source components of the Cramium SoC: the RISC-V core, cryptographic accelerators, and other miscellaneous primitives.

Compiled documentation: [RV core complex register set](https://ci.betrusted.io/cramium/index.html).

## Overview

The Cramium SoC is composed of a mix of closed and open source components. The physical design targets TSMC 22ULL, using traditional closed-source tooling, e.g. Synopsis & Cadence, with standard cell libraries and memory macros provided by TSMC and ARM.

Specific RTL components, such as the RISC-V CPU, SCE (Secure Crypto Engine), DA (Data Access Controller), and peripheral functions are shared with a CERN-OHL-W license.

Developers can use the shared RTL to disambiguate device functions, check header files, and more fully exploit the features of the chip through a better understanding of the underlying hardware implementation.

System integrators and auditors can use the shared RTL to inspect key security-related IP, and construct FPGA-based analogs that perform the same logical security operations as what is provided in the chip. This can also be used to verify that the chip behaves in a fashion consistent with the provided design files.

Verification of the physical construction of the chip is outside the scope of this repository, and is delegated to the [IRIS](https://arxiv.org/abs/2303.07406) project.

## Structure

The Vex CPU source is located in [./VexRiscv/GenCramSoC.scala](./VexRiscv/GenCramSoC.scala). This is compiled into a [verilog file](./VexRiscv/VexRiscv_CramSoC.v) and included into LiteX via a CPU wrapper.

The Vex core is wrapped in a custom LiteX CPU wrapper, at [./deps/litex/litex/soc/cores/cpu/vexriscv_axi/core.py](./deps/litex/litex/soc/cores/cpu/vexriscv_axi/core.py). For now, this project relies on a fork of LiteX.

The CPU is instantiated for SoC integration using [cram_core.py](./cram_core.py). This creates a "SoC"-less LiteX project which wraps the CPU core in an AXI crossbar from the `verilog-axi` project, allowing us to (a) have a private CSR space for RV-only peripherals, and (b) adapt the RV's bus widths to those specified by the Cramium SoC. `verilog-axi` is used because it seems more mature than the AXI primitives in LiteX as of 2023.

`cram_core.py` will output an artifact named [cram_axi.v](./candidate/cram_axi.v). This is the verilog file as integrated for tape-out. It is instantited with register initializations turned off, since on real silicon you can't pre-load registers with values on boot.

`cram_soc.py` and `cram_arty.py` are wrappers of `cram_core.py` that put the production SoC core into either a simulation framework (`soc`), or something that can target the Arty FPGA board (`arty`). The main reason we have two targets is so that in simulation we use abstract RAM models that are faster to simulate (and more accurate to the SoC) than the DRAM models used for the Arty implementation. Both of these scripts rely on a common model for simulation inside `cram_common.py`.

## OSS Tooling Flow

The primary targets of this repository are developers who need accurate header files that describe the hardware, and integrators/auditors who want to inspect the function of key RTL blocks. Thus, there are two tooling flows to cover: one for automated header file extraction, and one for simulating the RTL blocks.

### Header File Extraction

Header files are extracted using the following methodology:

1. A Python script scans the OSS RTL, and attempts to extract all of the instantiated registers in the RTL using heuristics to identify register idioms.
2. A SVD file is produced which contains an abstract description of the extracted registers
3. A secondary SVD file is produced which contains descriptions of registers coming from closed-source components
4. The SVD files are merged and run through tools to extract header files and automatically produce documentation

#### Bootable OS Image

The system is designed to run [Xous](https://github.com/betrusted-io/xous-core/), a secure microkernel operating system written in pure Rust. The generated SVD files are copied into the Xous tree, which contains the tooling necessary to merge and convert the SVD files to Rust, using the command `cargo xtask cramium-soc` within the cloned Xous repository.

### RTL Simulation

The primary RTL simulation flow is through Verilator, a fast-and-loose [1] OSS Verilog simulator. Here is the methodology:

1. All HDL is converted to Verilog. For portions written natively in SystemVerilog or Verilog, there is nothing to do. However, portions of the project are written in SpinalHDL (VexRiscv CPU) and LiteX (CPU integration & some peripherals). Non-Verilog portions are converted to verilog and committed to the `candidate` directory.
2. Test benches are coded in Rust, and compiled into a bootable binary ROM image.
3. A top-level integration integration script in LiteX is used to run Verilator. A full system simulation is run, of the compiled Rust test bench on the assembled RTL artifacts.
4. GTKWave is used to view the results, along with inspection of the emulated serial logs output by Verilator.

[1] Verilator is "fast and loose" in that it run faster than most other simulators, but loose in that it takes shortcuts. It is suitable for high-level functional verification but its inability to represent X/Z states means it misses important corner cases for physical design.

## Contribution Guidelines

Please see [CONTRIBUTING](./CONTRIBUTING.md) for details on
how to make a contribution.

Please note that this project is released with a
[Contributor Code of Conduct](./CODE_OF_CONDUCT.md/).
By participating in this project you agree to abide its terms.

## License

Copyright © 2019 - 2023

Licensed under the [CERN OHL v2-W](https://ohwr.org/cern_ohl_w_v2.txt) license.
