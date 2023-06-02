# Boot Test

A "raw iron" test program for the Cramium SoC.

This is designed to be loaded at the top of ReRAM (0x6000_0000). It can be optionally configured to target either the fully open-source simulation model peripherals, or the final production SoC peripherals (some of which are closed-source or have yet to be open sourced).

## Building on Windows

To build the firmware, first you will need the Visual Studio build tools, and the Rust toolchain.

1. Install the [Microsoft C++ Build Tools](https://visualstudio.microsoft.com/visual-cpp-build-tools/), by clicking "Download Build Tools" and running the installer (vs_BuildTools.exe). Select "Desktop Development with C++" for the tool set to install. This will consume about 10GiB of disk space.
2. Install the [Rust Toolchain](https://www.rust-lang.org/tools/install) by clicking on "Download Rustup-init.exe" and running the program. Select the "default" installation.
3. Check that you have Rust installed by opening a new terminal window and running `rustc --version`. When this documentation was written, the version was 1.70.0.
4. Install the RV32 target: `rustup target add riscv32imac-unknown-none-elf`
5. Install the [`xpack` tools](https://github.com/xpack-dev-tools/riscv-none-elf-gcc-xpack/releases) and put it in your PATH.
   - To see the windows binaries, you may have to click "Show all assets". The file will have a name something like `xpack-riscv-none-elf-gcc-12.2.0.3-win32-x64.zip`.
   - Extract it to a permanent location
   - Type `environment variables` into the Windows Start bar and select `Edit the System Environment Variables`.
   - Click on `Environment Variables...`
   - Click on `Path` and then `Edit`
   - Click `New` then browse to the location where you extracted the `xpack` tools and select the `bin/` subdirectory
   - Click OK until all the dialog boxes are closed.
   - Start a new terminal session, and type `riscv-none-elf-objdump.exe --version` and confirm that the command can succeed.

You are now ready to prepare the build environment.
## Preparing the Build Environment

### Clone the Code
You will need to have the following directory structure to build the firmware:

```
your code folder
   |
   |------------ cram-soc/
   |
   |------------ xous-cramium/
```

In other words, you will need to clone both [this repository](https://github.com/buncram/cram-soc) and the [xous-cramium](https://github.com/buncram/xous-cramium.git) repository.

You can do this with:

- `cd my_code_folder`
- `git clone https://github.com/buncram/cram-soc.git`
- `git clone https://github.com/buncram/xous-cramium.git`

and then `cd cram-soc`

### Copy the SoC Descriptor Files

If you are not building the SoC from source, you will need to copy the descriptor files. Do this with the following commands from the `cram-soc` directory:

`mkdir -p build/software`
`cp candidate/soc.svd build/software/`
`cp candidate/core.svd build/software/`

## Building the Test Firmware

Make sure you have already run `rustup target add riscv32imac-unknown-none-elf`.

From `cram-soc/boot` run:

`cargo xtask boot-image --feature daric`.

The binary file for the CPU will be located at

`boot.bin`

You can load this at the top of ReRAM and the CPU should start executing instructions from the reset vector at 0x6000_0000.

## Other Notes

If you want to check the output of the build, run this command first:

`cargo install rustfilt`

This will install a tool that can help reveal the meaning of the symbols in the object files. Then, you can run this:

```
riscv-none-elf-objdump -h target/riscv32imac-unknown-none-elf/release/betrusted-boot > boot.lst

riscv-none-elf-nm -r --size-sort --print-size target/riscv32imac-unknown-none-elf/release/betrusted-boot | rustfilt >> boot.lst

riscv-none-elf-objdump target/riscv32imac-unknown-none-elf/release/betrusted-boot -S -d | rustfilt >> boot.lst
```

It will create a file `boot.lst` that contains the location of all the offsets of symbols, as well as an assembly listing of the code.
