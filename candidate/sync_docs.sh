#!/bin/bash

# sync just the docs without regenerating all the things

rsync -aiv --delete ../build/gateware/build/documentation/_build/html/* bunnie@ci.betrusted.io:/var/cramium-cpu/
rsync -aiv --delete ../build/doc/daric_doc/_build/html/* bunnie@ci.betrusted.io:/var/cramium/
