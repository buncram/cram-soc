#!/bin/bash

# sync just the docs without regenerating all the things

rsync -a --delete ../build/gateware/build/documentation/_build/html/* bunnie@ci.betrusted.io:/var/cramium-cpu/
rsync -a --delete ../include/daric_doc/_build/html/* bunnie@ci.betrusted.io:/var/cramium/
