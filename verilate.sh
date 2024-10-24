#!/bin/bash

# Set default values for the options
TARGET="iron"
SPEED="normal"

# Function to display the script usage
function display_usage {
    echo "Usage: $0 [-t xous] [-s fast]"
    echo "-t: Select target [xous, iron]"
    echo "-s: Run fast (but don't save waveforms) [normal, fast]"
}

# Parse command line options
while getopts ":s:t:" opt; do
    case $opt in
        t)
            TARGET=$OPTARG
            ;;
        s)
            SPEED=$OPTARG
            ;;
        \?)
            echo "Invalid option: -$OPTARG"
            display_usage
            exit 1
            ;;
        :)
            echo "Option -$OPTARG requires an argument."
            display_usage
            exit 1
            ;;
    esac
done

mkdir -p listings

# Shift the parsed options, so the remaining arguments are at the end of the argument list
shift $((OPTIND - 1))

# Check if any non-option arguments are passed (if required)
if [ $# -ne 0 ]; then
    echo "Invalid arguments: $@"
    display_usage
    exit 1
fi

# Use the parsed options in your script logic
echo "Target: $TARGET"
echo "Speed: $SPEED"


set -e

echo "--------------------- BUILD CORE --------------------"
python3 ./cram_core.py

echo "******************** BUILD SOC DEFS ***********************"
python3 ./cram_soc.py --svd-only
echo "Core+SoC build finished."

echo "******************** BUILD KERNEL ***********************"
if [ $TARGET == "xous" ]
then
  echo "Simulating Xous target"
  cp build/software/soc.svd ../xous-core/utralib/cramium/
  cp build/software/core.svd ../xous-core/utralib/cramium/
  cd ../xous-core
  # cd ./loader
  # set up the linker for our target
  # cp link-soc.x link.x
  # cd ../
  cargo xtask cramium-soc # cram-mbox1 cram-mbox2 --kernel-feature fake-rng
  # cargo xtask cramium-fpga --kernel-feature fake-rng
  cd ../cram-soc
  python3 ./mkimage.py
  ./disasm_load.sh
  BIOS="./simspi.init"
else
  echo "Simulating raw iron target"
  # regenerate PIO include from source
  #python3 ./pio_to_svd.py
  #cp include/pio_generated.rs ../xous-cramium/libs/xous-pio/src/
  #cp include/pio.svd ../xous-cramium/precursors/

  # copy over all the latest SVD files
  cp build/software/soc.svd ../nto-tests/svd
  cp build/software/core.svd ../nto-tests/svd
  cp include/daric.svd ../nto-tests/svd

  # build the binary
  cd ../nto-tests
  cp tests/link.x.straight tests/link.x
  # change --boot-offset in the cramy_soc.py commandline to match what is in link.x!!
  cargo xtask boot-image --no-default-features --feature fast-fclk --feature quirks-pll --feature aes-zkn --feature bio-mul
  python3 ./merge_cm7.py --rv32=rv32.bin --cm7=../daric/daricval/examples/mbox/mbox.bin --out-file=boot.bin

  riscv-none-elf-objdump -h target/riscv32imac-unknown-none-elf/release/tests > /mnt/f/code/cram-soc/listings/boot.lst
  riscv-none-elf-nm -r --size-sort --print-size target/riscv32imac-unknown-none-elf/release/tests | rustfilt >> /mnt/f/code/cram-soc/listings/boot.lst
  riscv-none-elf-objdump target/riscv32imac-unknown-none-elf/release/tests -S -d | rustfilt >> /mnt/f/code/cram-soc/listings/boot.lst

  cd ../cram-soc
  BIOS="../nto-tests/boot.bin"
fi
echo "******************** RUN SIM ***********************"

cp soc_oss/rtl/amba/apb_sfr_v0.1.sv build/sim/gateware/
cp soc_oss/rtl/common/template.sv build/sim/gateware/
cp soc_oss/rtl/common/amba_interface_def_v0.2.sv build/sim/gateware/
cp soc_oss/rtl/model/artisan_ram_def_v0.1.svh build/sim/gateware/
cp soc_oss/rtl/common/icg_v0.2.v build/sim/gateware/

THREADS=5

# remember - trace-start is not 0!

#for THREADS in 1 2 3 4 5 6 7 8 9
#do
  echo "Don't forget: finisher.v needs to have the XOUS variable defined according to the target config."
  echo -e "\n\nRun with $THREADS threads" >> stats.txt
  date >> stats.txt
  /usr/bin/time -a --output stats.txt python3 ./cram_soc.py --speed $SPEED --bios $BIOS  --boot-offset 0x000000 --gtkwave-savefile --threads $THREADS --jobs 20 --trace --trace-start 0 --trace-end 200000000000 --trace-fst # --sim-debug
  echo "Core+SoC build finished."
#done

# Run with 1 threads
# 504.73user 9.09system 8:46.71elapsed 97%CPU
# Run with 2 threads
# 1009.23user 5.51system 10:19.70elapsed 163%CPU
# Run with 3 threads
# 1375.28user 9.12system 12:36.14elapsed 183%CPU
# Run with 4 threads
# 1843.01user 5.70system 12:03.51elapsed 255%CPU
# Run with 5 threads
# 2173.25user 8.82system 10:02.39elapsed 362%CPU
# Run with 6 threads
# 2711.23user 5.60system 10:25.95elapsed 434%CPU
# Run with 7 threads
# 3947.65user 32.89system 12:21.29elapsed 536%CPU
# Run with 8 threads
# 5211.18user 31.81system 13:53.02elapsed 629%CPU
# Run with 9 threads
# 6420.21user 31.14system 14:58.35elapsed 718%CPU
#
# Run with 1 threads
# 614.51user 7.60system 10:18.79elapsed 100%CPU
# Run with 2 threads
# 1021.01user 4.43system 10:09.27elapsed 168%CPU
# Run with 3 threads
# 1338.06user 4.45system 9:43.67elapsed 230%CPU
# Run with 4 threads
# 1836.40user 4.06system 10:12.53elapsed 300%CPU
# Run with 5 threads
# 1934.28user 7.14system 9:14.44elapsed 350%CPU
# Run with 6 threads
# 2812.04user 4.42system 10:41.47elapsed 439%CPU
# Run with 7 threads
# 4693.68user 8.28system 14:10.57elapsed 552%CPU
# Run with 8 threads
# 4885.04user 7.43system 13:13.18elapsed 616%CPU
# Run with 9 threads
# 5916.85user 7.89system 13:59.61elapsed 705%CPU
