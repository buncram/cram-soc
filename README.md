# Cramium SoC

This is the open source RISC-V core on the Cramium SoC.

Compiled documentation: [RV core complex register set](https://ci.betrusted.io/cramium/index.html).

## Structure ##

- The Vex CPU source is located in [./VexRiscv/GenCramSoC.scala](./VexRiscv/GenCramSoC.scala). This is compiled into a [verilog file](./VexRiscv/VexRiscv_CramSoC.v) and included into LiteX via a CPU wrapper.
- The Vex core is wrapped in a custom LiteX CPU wrapper, at [./deps/litex/litex/soc/cores/cpu/vexriscv_axi/core.py](./deps/litex/litex/soc/cores/cpu/vexriscv_axi/core.py). For now, this project relies on a fork of LiteX.
- The CPU is instantiated for SoC integration using [cram_core.py](./cram_core.py). This creates a "SoC"-less LiteX project which wraps the CPU core in an AXI crossbar from the `verilog-axi` project, allowing us to (a) have a private CSR space for RV-only peripherals, and (b) adapt the RV's bus widths to those specified by the Cramium SoC. `verilog-axi` is used because it seems more mature than the AXI primitives in LiteX as of 2023.
- `cram_core.py` will output an artifact named [cram_axi.v](./candidate/cram_axi.v). This is the verilog file as integrated for tape-out. It is instantited with register initializations turned off, since on real silicon you can't pre-load registers with values on boot.
- `cram_soc.py` and `cram_arty.py` are wrappers of `cram_core.py` that put the production SoC core into either a simulation framework (`soc`), or something that can target the Arty FPGA board (`arty`). The main reason we have two targets is so that in simulation we use abstract RAM models that are faster to simulate (and more accurate to the SoC) than the DRAM models used for the Arty implementation. Both of these scripts rely on a common model for simulation inside `cram_common.py`.

## Building ##

1. Check out this repo with `git clone --recurse-submodules <repo>`.
1. Ensure you have Python 3.5 or newer installed.
1. Ensure the following python packages are available in your environment: `pycryptodome` (signing - PEM read), `cryptography` (signind - x509 read), `pynacl` (signing - ed25519 signatures), `progressbar2` (updates), `pyusb` (updates).
1. Ensure the `riscv32imac-unknown-none-elf` Rust target is installed via `rustup target add riscv32imac-unknown-none-elf`.
1. Ensure you have `make` installed.
1. Download the Risc-V toolchain from https://github.com/xpack-dev-tools/riscv-none-elf-gcc-xpack/releases and put it in your PATH. The bootloader is pure Rust, except for a few assembly instructions to set up the runtime environment (which are easily verified with a hexdump). However, the full toolchain is still required for LiteX to run correctly.
1. Go to https://www.xilinx.com/support/download.html and download Vivado. This was tested under 2022.2.
1. Do a minimal Xilinx install to /opt/Xilinx/, and untick everything except `Design Tools / Vivado Design Suite / Vivado` and `Devices / Production Devices / 7 Series`
1. Run `./cram_core.py` (or `python3 ./cram_core.py`) to build the core complex
1. Run `./cram_soc.py` (or `python3 ./cram-soc.py`) to build the SoC validation model. This is the core complex wrapped in additional peripherals so that we can do end-to-end testing in simulation.
1. Run `./cram_fpga.py` (or `python3 ./cram-fpga.py`) to build the FPGA validation model. This is the core complex wrapped in an "approximation" of the SoC that allows us to run the core complex at real-time, albeit with some compromises in how accurately it reflects the SoC.

For simulation, the `verilate.sh` script will get you running. However, it requires you to build and install the latest version of verilator (the version packaged in most distros is too old to work with the tools here).

## Using lxbuildenv.py Environment ##

`lxbuildenv` is a Python module.  It sets up the build environment and ensures you have the correct dependencies.  To use it, start your program off with:

```python
#!/usr/bin/env python3
import lxbuildenv
```

*`lxbuildenv.py` has some very surprising behaviors* that you should be aware of:

1. In order to set environment variables such as `PYTHONHASHSEED`, `lxbuildenv` will actually re-exec the Python interpreter.  This will, among other things, cause the pid to change.  *This is why lxbuildenv should be imported first*.
1. The environment variable `PYTHONPATH` is replaced to include every directory under `deps/`.  If you rely on `PYTHONPATH` to be something else, this may surprise you.
1. `lxbuildenv` has several command line parameters that it can accept.  To display these, run your command with the `--lx-help` parameter.
1. The `deps/` directory includes its own `site.py` implementation, adapted from a Debian implementation.  This is because some distros force `/usr/share/python/site-packages/` to be first in the dependency list, which causes confusing dependency interactions.  If you're relying on site packages being in a certain order, this may cause problems.  You can try deleting `deps/site/` in order to disable this behavior.

In exchange for some deviation from other build environments, `lxbuildenv` gives you several benefits that come in handy for hardware projects:

1. Python dicts enumerate in constant order, giving some consistency to build results.
1. You will probably be modifying code in the dependencies.  By keeping them inside the project directory, this becomes much simpler.
1. Additionally, all dependencies remain under version control, which you would otherwise lose when installing dependencies as packages.
1. Hardware, moreso than software, depends on exact version numbers.  By using `git` to track dependency versions, this build becomes more reproducible.
1. It is cross-platform, and works anywhere Xilinx does.
1. The `lxbuildenv` environment doesn't rely on opaque environment variables, or otherwise have a special environment you enter.  Everything is documented behind `--help` flags.

## Xilinx PATH ##

If your Xilinx install is in the default path (`C:\\Xilinx` on Windows, `/opt/Xilinx` on Linux), then the build system should be able to automatically find Xilinx.

If not, you can add the Xilinx `bin` directory to your PATH.

## PyCharm integration ##

To use PyCharm, open this directory as a `Project` by going to the *File* menu and selecting *Open...*.  Make sure you open the entire directory, and not just a single file in this directory.

When you first open this project, you'll see lots of red squiggly lines indicating errors.  PyCharm needs to know about the dependency structure in order to allow you to drill down into modules and auto-complete statements.

Open this directory in PyCharm and expand the `deps/` directory.  Then hold down `Shift` and select all subdirectories under `deps/`.  This will include `litedram`, `liteeth`, and so on.

Then, right-click and select `Mark directory as...` and select `Sources Root`.  The red squiggly lines should go away, and PyCharm should now be configured.

When running your module from within PyCharm, you may find it useful to set environment variables.  You can use the `--lx-print-env` command.  For example: `./betrusted_soc.py --lx-print-env > pycharm.env` to create a `.env`-compatible file.  There are several PyCharm plugins that can make use of this file.

## Visual Studio Code integration ##

Visual Studio Code needs to know where modules are.  These are specified in environment variables, which are automatically read from a .env file in your project root.  Create this file to enable `pylint` and debugging in Visual Studio Code:

```sh
$ python ./betrusted_soc.py --lx-print-env > .env
```

The analyzer will also need to know where your imports are. This would involve editing your `settings.json` file and adding a record that looks a bit like this:
```json
    "python.analysis.extraPaths": ["C:\\PATH-TO-PROJECT\\betrusted-soc\\deps\\litex", "C:\\PATH-TO-PROJECT\\betrusted-soc\\deps\\migen", "C:\\PATH-TO-PROJECT\\betrusted-soc\\deps\\gateware", "C:\\PATH-TO-PROJECT\\betrusted-soc\\deps\\valentyusb"]
```

## Contribution Guidelines

Please see [CONTRIBUTING](./CONTRIBUTING.md) for details on
how to make a contribution.

Please note that this project is released with a
[Contributor Code of Conduct](./CODE_OF_CONDUCT.md/).
By participating in this project you agree to abide its terms.

## License

Copyright © 2019 - 2023

Licensed under the [CERN OHL v2-W](https://ohwr.org/cern_ohl_w_v2.txt) license.
