#!/bin/bash
set -e

echo "\n\n--------------------- BUILD CORE --------------------\n\n"
python3 ./cram_core.py

echo "\n\n******************** BUILD SOC DEFS ***********************\n\n"
python3 ./cram_soc.py --svd-only
echo "Core+SoC build finished."

echo "\n\n******************** BUILD KERNEL ***********************\n\n"
cp build/software/soc.svd ../xous-cramium/precursors/
cp build/software/core.svd ../xous-cramium/precursors/
cd ../xous-cramium
cd ./loader
# set up the linker for our target
cp link-soc.x link.x
cd ../
cargo xtask hw-image --kernel-feature hwsim --feature hwsim
python3 ./mkimage.py
cd ../cram-soc

echo "\n\n******************** RUN SIM ***********************\n\n"

python3 ./cram_soc.py --gtkwave-savefile --threads 4 --jobs 20 --trace --trace-start 0 --trace-end 200_000_000_000 --trace-fst # --sim-debug
echo "Core+SoC build finished."

# 17:51 - 18 threads
# 14:59 - 9 threads
# 10:24 - 5 threads