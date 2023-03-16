#!/usr/bin/env python3
#
# This file is Copyright (c) 2022 Cramium Labs, Inc.
#
# Incorporates CSR, header, and SVD generators from LiteX with the following copyrights:
#
# This file is Copyright (c) 2013-2014 Sebastien Bourdeauducq <sb@m-labs.hk>
# This file is Copyright (c) 2014-2019 Florent Kermarrec <florent@enjoy-digital.fr>
# This file is Copyright (c) 2018 Dolu1990 <charles.papon.90@gmail.com>
# This file is Copyright (c) 2019 Gabriel L. Somlo <gsomlo@gmail.com>
# This file is Copyright (c) 2018 Jean-François Nguyen <jf@lambdaconcept.fr>
# This file is Copyright (c) 2019 Antmicro <www.antmicro.com>
# This file is Copyright (c) 2013 Robert Jordens <jordens@gmail.com>
# This file is Copyright (c) 2018 Sean Cross <sean@xobs.io>
# This file is Copyright (c) 2018 Sergiusz Bazanski <q3k@q3k.org>
# This file is Copyright (c) 2018-2016 Tim 'mithro' Ansell <me@mith.ro>
# This file is Copyright (c) 2015 whitequark <whitequark@whitequark.org>
# This file is Copyright (c) 2018 William D. Jones <thor0505@comcast.net>
# This file is Copyright (c) 2020 Piotr Esden-Tempski <piotr@esden.net>
# This file is Copyright (c) 2022 Franck Jullien <franck.jullien@collshade.fr>
#
# SPDX-License-Identifier: BSD-2-Clause

from pathlib import Path
import argparse
import textwrap
import datetime
import time
import logging
from math import log2
import sys
import subprocess
import os

from litex.soc.doc.csr import DocumentedCSRRegion
from litex.soc.interconnect.csr import _CompoundCSR, CSRStorage, CSRStatus, CSRField

import re

def colorer(s, color="bright"):
    header  = {
        "bright": "\x1b[1m",
        "green":  "\x1b[32m",
        "cyan":   "\x1b[36m",
        "red":    "\x1b[31m",
        "yellow": "\x1b[33m",
        "underline": "\x1b[4m"}[color]
    trailer = "\x1b[0m"
    return header + str(s) + trailer

def log2_int(n, need_pow2=True):
    if n == 0:
        return 0
    r = (n - 1).bit_length()
    if need_pow2 and (1 << r) != n:
        raise ValueError("Not a power of 2")
    return r

class SoCCSRRegion:
    def __init__(self, origin, busword, obj):
        self.origin  = origin
        self.busword = busword
        self.obj     = obj

class SoCError(Exception):
    def __init__(self):
        sys.stderr = None # Error already described, avoid traceback/exception.

# SoCRegion ----------------------------------------------------------------------------------------

class SoCRegion:
    def __init__(self, origin=None, size=None, mode="rw", cached=True, linker=False, decode=True):
        self.logger    = logging.getLogger("SoCRegion")
        self.origin    = origin
        self.decode    = decode
        self.size      = size
        if size != 2**log2_int(size, False):
            self.logger.info("Region size {} internally from {} to {}.".format(
                colorer("rounded", color="cyan"),
                colorer("0x{:08x}".format(size)),
                colorer("0x{:08x}".format(2**log2_int(size, False)))))
        self.size_pow2 = 2**log2_int(size, False)
        self.mode      = mode
        self.cached    = cached
        self.linker    = linker

    def decoder(self, bus):
        origin = self.origin
        size   = self.size_pow2
        if (origin & (size - 1)) != 0:
            self.logger.error("Origin needs to be aligned on size:")
            self.logger.error(self)
            raise SoCError()
        if not self.decode or (origin == 0) and (size == 2**bus.address_width):
            return lambda a: True
        origin >>= int(log2(bus.data_width//8)) # bytes to words aligned.
        size   >>= int(log2(bus.data_width//8)) # bytes to words aligned.
        return lambda a: (a[log2_int(size):] == (origin >> log2_int(size)))

    def __str__(self):
        r = ""
        if self.origin is not None:
            r += "Origin: {}, ".format(colorer("0x{:08x}".format(self.origin)))
        if self.size is not None:
            r += "Size: {}, ".format(colorer("0x{:08x}".format(self.size)))
        r += "Mode: {}, ".format(colorer(self.mode.upper()))
        r += "Cached: {} ".format(colorer(self.cached))
        r += "Linker: {}".format(colorer(self.linker))
        return r

class SoCIORegion(SoCRegion): pass

def pad_first_line_if_necessary(s):
    if not isinstance(s, str):
        return s
    lines = s.split("\n")

    # If there aren't at least two lines, don't do anything
    if len(lines) < 2:
        return s

    # If the first line is blank, don't do anything
    if lines[0].strip() == "":
        return s

    # If the pading on line 1 is greater than line 2, pad line 1
    # and return the result
    line_0_padding = len(lines[0]) - len(lines[0].lstrip(' '))
    line_1_padding = len(lines[1]) - len(lines[1].lstrip(' '))
    if (line_1_padding > 0) and (line_1_padding > line_0_padding):
        lines[0] = " " * (line_1_padding - line_0_padding) + lines[0]
        return "\n".join(lines)
    return s

def reflow(s, width=80):
    """Reflow the jagged text that gets generated as part
    of this Python comment.

    In this comment, the first line would be indented relative
    to the rest.  Additionally, the width of this block would
    be limited to the original text width.

    To reflow text, break it along \n\n, then dedent and reflow
    each line individually.

    Finally, append it to a new string to be returned.
    """
    if not isinstance(s, str):
        return s
    out = []
    s = pad_first_line_if_necessary(s)
    for piece in textwrap.dedent(s).split("\n\n"):
        trimmed_piece = textwrap.fill(textwrap.dedent(piece).strip(), width=width)
        out.append(trimmed_piece)
    return "\n\n".join(out)


# C Export -----------------------------------------------------------------------------------------

def get_git_header():
    from litex.build.tools import get_litex_git_revision
    r = generated_banner("//")
    r += "#ifndef __GENERATED_GIT_H\n#define __GENERATED_GIT_H\n\n"
    r += f"#define LITEX_GIT_SHA1 \"{get_litex_git_revision()}\"\n"
    r += "#endif\n"
    return r

def get_mem_header(regions):
    r = generated_banner("//")
    r += "#ifndef __GENERATED_MEM_H\n#define __GENERATED_MEM_H\n\n"
    for name, region in regions.items():
        r += f"#ifndef {name.upper()}_BASE\n"
        r += f"#define {name.upper()}_BASE 0x{region.origin:08x}L\n"
        r += f"#define {name.upper()}_SIZE 0x{region.size:08x}\n"
        r += "#endif\n\n"

    r += "#ifndef MEM_REGIONS\n"
    r += "#define MEM_REGIONS \"";
    name_length = max([len(name) for name in regions.keys()])
    for name, region in regions.items():
        r += f"{name.upper()} {' '*(name_length-len(name))} 0x{region.origin:08x} 0x{region.size:x} \\n"
    r = r[:-2]
    r += "\"\n"
    r += "#endif\n"

    r += "#endif\n"
    return r

def get_soc_header(constants, with_access_functions=True):
    r = generated_banner("//")
    r += "#ifndef __GENERATED_SOC_H\n#define __GENERATED_SOC_H\n"
    funcs = ""

    for name, value in constants.items():
        if value is None:
            r += "#define "+name+"\n"
            continue
        if isinstance(value, str):
            value = "\"" + value + "\""
            ctype = "const char *"
        else:
            value = str(value)
            ctype = "int"
        r += "#define "+name+" "+value+"\n"
        if with_access_functions:
            funcs += "static inline "+ctype+" "+name.lower()+"_read(void) {\n"
            funcs += "\treturn "+value+";\n}\n"

    if with_access_functions:
        r += "\n#ifndef __ASSEMBLER__\n"
        r += funcs
        r += "#endif // !__ASSEMBLER__\n"

    r += "\n#endif\n"
    return r

def _get_csr_addr(csr_base, addr, with_csr_base_define=True):
    if with_csr_base_define:
        return f"(CSR_BASE + {hex(addr)}L)"
    else:
        return f"{hex(csr_base + addr)}L"

def _get_rw_functions_c(reg_name, reg_base, nwords, busword, alignment, read_only, csr_base, with_csr_base_define, with_access_functions):
    r = ""

    addr_str = f"CSR_{reg_name.upper()}_ADDR"
    size_str = f"CSR_{reg_name.upper()}_SIZE"
    r += f"#define {addr_str} {_get_csr_addr(csr_base, reg_base, with_csr_base_define)}\n"

    r += f"#define {size_str} {nwords}\n"

    size = nwords*busword//8
    if size > 8:
        # Downstream should select appropriate `csr_[rd|wr]_buf_uintX()` pair!
        return r
    elif size > 4:
        ctype = "uint64_t"
    elif size > 2:
        ctype = "uint32_t"
    elif size > 1:
        ctype = "uint16_t"
    else:
        ctype = "uint8_t"

    stride = alignment//8;
    if with_access_functions:
        r += f"static inline {ctype} {reg_name}_read(void) {{\n"
        if nwords > 1:
            r += f"\t{ctype} r = csr_read_simple({_get_csr_addr(csr_base, reg_base, with_csr_base_define)});\n"
            for sub in range(1, nwords):
                r += f"\tr <<= {busword};\n"
                r += f"\tr |= csr_read_simple({_get_csr_addr(csr_base, reg_base+sub*stride, with_csr_base_define)});\n"
            r += "\treturn r;\n}\n"
        else:
            r += f"\treturn csr_read_simple({_get_csr_addr(csr_base, reg_base, with_csr_base_define)});\n}}\n"

        if not read_only:
            r += f"static inline void {reg_name}_write({ctype} v) {{\n"
            for sub in range(nwords):
                shift = (nwords-sub-1)*busword
                if shift:
                    v_shift = "v >> {}".format(shift)
                else:
                    v_shift = "v"
                r += f"\tcsr_write_simple({v_shift}, {_get_csr_addr(csr_base, reg_base+sub*stride, with_csr_base_define)});\n"
            r += "}\n"
    return r

def get_litex_git_revision():
    import litex
    d = os.getcwd()
    os.chdir(os.path.dirname(litex.__file__))
    try:
        r = subprocess.check_output(["git", "rev-parse", "--short", "HEAD"],
                stderr=subprocess.DEVNULL)[:-1].decode("utf-8")
    except:
        r = "--------"
    os.chdir(d)
    return r

def generated_banner(line_comment="//"):
    r = line_comment + "-"*80 + "\n"
    r += line_comment + " Auto-generated by sce_to_svd (derived from LiteX) ({}) on ".format(get_litex_git_revision())
    r += "{}\n".format(datetime.datetime.fromtimestamp(time.time()).strftime("%Y-%m-%d %H:%M:%S"))
    r += line_comment + "-"*80 + "\n"
    return r


def get_csr_header(regions, constants, csr_base=None, with_csr_base_define=True, with_access_functions=True):
    alignment = constants.get("CONFIG_CSR_ALIGNMENT", 32)
    r = generated_banner("//")
    if with_access_functions: # FIXME
        r += "#include <generated/soc.h>\n"
    r += "#ifndef __GENERATED_CSR_H\n#define __GENERATED_CSR_H\n"
    if with_access_functions:
        r += "#include <stdint.h>\n"
        r += "#include <system.h>\n"
        r += "#ifndef CSR_ACCESSORS_DEFINED\n"
        r += "#include <hw/common.h>\n"
        r += "#endif /* ! CSR_ACCESSORS_DEFINED */\n"
    csr_base = csr_base if csr_base is not None else regions[next(iter(regions))].origin
    if with_csr_base_define:
        r += "#ifndef CSR_BASE\n"
        r += f"#define CSR_BASE {hex(csr_base)}L\n"
        r += "#endif\n"
    for name, region in regions.items():
        origin = region.origin - csr_base
        r += "\n/* "+name+" */\n"
        r += f"#define CSR_{name.upper()}_BASE {_get_csr_addr(csr_base, origin, with_csr_base_define)}\n"
        #if not isinstance(region.obj, Memory):
        for csr in region.obj:
            nr = (csr.size + region.busword - 1)//region.busword
            r += _get_rw_functions_c(
                reg_name              = name + "_" + csr.name,
                reg_base              = origin,
                nwords                = nr,
                busword               = region.busword,
                alignment             = alignment,
                read_only             = getattr(csr, "read_only", False),
                csr_base              = csr_base,
                with_csr_base_define  = with_csr_base_define,
                with_access_functions = with_access_functions,
            )
            origin += alignment//8*nr
            if hasattr(csr, "fields"):
                for field in csr.fields.fields:
                    offset = str(field.offset)
                    size = str(field.size)
                    r += f"#define CSR_{name.upper()}_{csr.name.upper()}_{field.name.upper()}_OFFSET {offset}\n"
                    r += f"#define CSR_{name.upper()}_{csr.name.upper()}_{field.name.upper()}_SIZE {size}\n"
                    if with_access_functions and csr.size <= 32: # FIXME: Implement extract/read functions for csr.size > 32-bit.
                        reg_name   = name + "_" + csr.name.lower()
                        field_name = reg_name + "_" + field.name.lower()
                        r += "static inline uint32_t " + field_name + "_extract(uint32_t oldword) {\n"
                        r += f"\tuint32_t mask = 0x{(1<<int(size))-1:x};\n"
                        r += "\treturn ( (oldword >> " + offset + ") & mask );\n}\n"
                        r += "static inline uint32_t " + field_name + "_read(void) {\n"
                        r += "\tuint32_t word = " + reg_name + "_read();\n"
                        r += "\treturn " + field_name + "_extract(word);\n"
                        r += "}\n"
                        if not getattr(csr, "read_only", False):
                            r += "static inline uint32_t " + field_name + "_replace(uint32_t oldword, uint32_t plain_value) {\n"
                            r += f"\tuint32_t mask = 0x{(1<<int(size))-1:x};\n"
                            r += "\treturn (oldword & (~(mask << " + offset + "))) | (mask & plain_value)<< " + offset + " ;\n}\n"
                            r += "static inline void " + field_name + "_write(uint32_t plain_value) {\n"
                            r += "\tuint32_t oldword = " + reg_name + "_read();\n"
                            r += "\tuint32_t newword = " + field_name + "_replace(oldword, plain_value);\n"
                            r += "\t" + reg_name + "_write(newword);\n"
                            r += "}\n"

    r += "\n#endif\n"
    return r

# SVD Export --------------------------------------------------------------------------------------

def get_csr_svd(soc, vendor="litex", name="soc", description=None):
    def sub_csr_bit_range(busword, csr, offset):
        nwords = (csr.size + busword - 1)//busword
        i = nwords - offset - 1
        nbits = min(csr.size - i*busword, busword) - 1
        name = (csr.name + str(i) if nwords > 1 else csr.name).upper()
        origin = i*busword
        return (origin, nbits, name)

    def print_svd_register(csr, csr_address, description, length, svd):
        svd.append('                <register>')
        svd.append('                    <name>{}</name>'.format(csr.short_numbered_name))
        if description is not None:
            svd.append('                    <description><![CDATA[{}]]></description>'.format(description))
        svd.append('                    <addressOffset>0x{:04x}</addressOffset>'.format(csr_address))
        svd.append('                    <resetValue>0x{:02x}</resetValue>'.format(csr.reset_value))
        svd.append('                    <size>{}</size>'.format(length))
        # svd.append('                    <access>{}</access>'.format(csr.access))  # 'access' is a lie: "read-only" registers can legitimately change state based on a write, and is in fact used to handle the "pending" field in events
        csr_address = csr_address + 4
        svd.append('                    <fields>')
        if hasattr(csr, "fields") and len(csr.fields) > 0:
            for field in csr.fields:
                svd.append('                        <field>')
                svd.append('                            <name>{}</name>'.format(field.name))
                svd.append('                            <msb>{}</msb>'.format(field.offset +
                                                                         field.size - 1))
                svd.append('                            <bitRange>[{}:{}]</bitRange>'.format(
                    field.offset + field.size - 1, field.offset))
                svd.append('                            <lsb>{}</lsb>'.format(field.offset))
                svd.append('                            <description><![CDATA[{}]]></description>'.format(
                    reflow(field.description)))
                svd.append('                        </field>')
        else:
            field_size = csr.size
            field_name = csr.short_name.lower()
            # Strip off "ev_" from eventmanager fields
            if field_name == "ev_enable":
                field_name = "enable"
            elif field_name == "ev_pending":
                field_name = "pending"
            elif field_name == "ev_status":
                field_name = "status"
            svd.append('                        <field>')
            svd.append('                            <name>{}</name>'.format(field_name))
            svd.append('                            <msb>{}</msb>'.format(field_size - 1))
            svd.append('                            <bitRange>[{}:{}]</bitRange>'.format(field_size - 1, 0))
            svd.append('                            <lsb>{}</lsb>'.format(0))
            svd.append('                        </field>')
        svd.append('                    </fields>')
        svd.append('                </register>')

    interrupts = {}
    for csr, irq in sorted(soc.irq.locs.items()):
        interrupts[csr] = irq

    documented_regions = []
    for region_name, region in soc.csr.regions.items():
        documented_regions.append(DocumentedCSRRegion(
            name           = region_name,
            region         = region,
            csr_data_width = soc.csr.data_width)
        )

    svd = []
    svd.append('<?xml version="1.0" encoding="utf-8"?>')
    svd.append('')
    svd.append('<device schemaVersion="1.1" xmlns:xs="http://www.w3.org/2001/XMLSchema-instance" xs:noNamespaceSchemaLocation="CMSIS-SVD.xsd" >')
    svd.append('    <vendor>{}</vendor>'.format(vendor))
    svd.append('    <name>{}</name>'.format(name.upper()))
    if description is not None:
        svd.append('    <description><![CDATA[{}]]></description>'.format(reflow(description)))
    else:
        fmt = "%Y-%m-%d %H:%M:%S"
        build_time = datetime.datetime.fromtimestamp(time.time()).strftime(fmt)
        svd.append('    <description><![CDATA[{}]]></description>'.format(reflow("Litex SoC " + build_time)))
    svd.append('')
    svd.append('    <addressUnitBits>8</addressUnitBits>')
    svd.append('    <width>32</width>')
    svd.append('    <size>32</size>')
    svd.append('    <access>read-write</access>')
    svd.append('    <resetValue>0x00000000</resetValue>') # this is not correct
    svd.append('    <resetMask>0xFFFFFFFF</resetMask>')
    svd.append('')
    svd.append('    <peripherals>')

    for region in documented_regions:
        csr_address = 0
        svd.append('        <peripheral>')
        svd.append('            <name>{}</name>'.format(region.name.upper()))
        svd.append('            <baseAddress>0x{:08X}</baseAddress>'.format(region.origin))
        svd.append('            <groupName>{}</groupName>'.format(region.name.upper()))
        if len(region.sections) > 0:
            svd.append('            <description><![CDATA[{}]]></description>'.format(
                reflow(region.sections[0].body())))
        svd.append('            <registers>')
        for csr in region.csrs:
            description = None
            if hasattr(csr, "description"):
                description = csr.description
            if isinstance(csr, _CompoundCSR) and len(csr.simple_csrs) > 1:
                is_first = True
                for i in range(len(csr.simple_csrs)):
                    (start, length, name) = sub_csr_bit_range(
                        region.busword, csr, i)
                    if length > 0:
                        bits_str = "Bits {}-{} of `{}`.".format(
                            start, start+length, csr.name)
                    else:
                        bits_str = "Bit {} of `{}`.".format(
                            start, csr.name)
                    if is_first:
                        if description is not None:
                            print_svd_register(
                                csr.simple_csrs[i], csr_address, bits_str + " " + description, length, svd)
                        else:
                            print_svd_register(
                                csr.simple_csrs[i], csr_address, bits_str, length, svd)
                        is_first = False
                    else:
                        print_svd_register(
                            csr.simple_csrs[i], csr_address, bits_str, length, svd)
                    csr_address = csr_address + 4
            else:
                length = ((csr.size + region.busword - 1) //
                            region.busword) * region.busword
                print_svd_register(
                    csr, csr_address, description, length, svd)
                csr_address = csr_address + 4
        svd.append('            </registers>')
        svd.append('            <addressBlock>')
        svd.append('                <offset>0</offset>')
        svd.append('                <size>0x{:x}</size>'.format(csr_address))
        svd.append('                <usage>registers</usage>')
        svd.append('            </addressBlock>')
        if region.name in interrupts:
            svd.append('            <interrupt>')
            svd.append('                <name>{}</name>'.format(region.name))
            svd.append('                <value>{}</value>'.format(interrupts[region.name]))
            svd.append('            </interrupt>')
        svd.append('        </peripheral>')
    svd.append('    </peripherals>')
    svd.append('    <vendorExtensions>')

    if len(soc.mem_regions) > 0:
        svd.append('        <memoryRegions>')
        for region_name, region in soc.mem_regions.items():
            svd.append('            <memoryRegion>')
            svd.append('                <name>{}</name>'.format(region_name.upper()))
            svd.append('                <baseAddress>0x{:08X}</baseAddress>'.format(region.origin))
            svd.append('                <size>0x{:08X}</size>'.format(region.size))
            svd.append('            </memoryRegion>')
        svd.append('        </memoryRegions>')

    svd.append('        <constants>')
    for name, value in soc.constants.items():
        svd.append('            <constant name="{}" value="{}" />'.format(name, value))
    svd.append('        </constants>')

    svd.append('    </vendorExtensions>')
    svd.append('</device>')
    return "\n".join(svd)

class DocIrq():
    def __init__(self):
        self.locs = {
            "test": 1
        }

class DocCsr():
    def __init__(self):
        self.regions = {
            "test_reg" : SoCCSRRegion(
                0x4000_0000,
                32,
                [
                    CSRStatus(
                        name="fifo_status",
                        fields=[
                        CSRField(name="rx_words", size=11, description="Number of words available to read"),
                        CSRField(name="tx_words", size=11, description="Number of words pending in write FIFO. Free space is {} - `tx_avail`".format(10)),
                        CSRField(name="abort_in_progress", size=1, description="This bit is set if an `aborting` event was initiated and is still in progress."),
                        CSRField(name="abort_ack", size=1,
                        description="""This bit is set by the peer that acknowledged the incoming abort
            (the later of the two, in case of an imperfect race condition). The abort response handler should
            check this bit; if it is set, no new acknowledgement shall be issued. The bit is cleared
            when an initiator initiates a new abort. The initiator shall also ignore the state of this
            bit if it is intending to initiate a new abort cycle."""),
                        CSRField(name="tx_err", size=1, description="Set if the write FIFO overflowed because we wrote too much data. Cleared on register read."),
                        CSRField(name="rx_err", size=1, description="Set if read FIFO underflowed because we read too much data. Cleared on register read."),
                    ]),
                    CSRStorage(
                        name="abort",
                        fields=[
                            CSRField(name="abort", size=1, description=
                            """Write `1` to this field to both initiate and acknowledge an abort.
                Empties both FIFOs, asserts `aborting`, and prevents an interrupt from being generated by
                an incoming abort request. New reads & writes are ignored until `aborted` is asserted
                from the peer.""", pulse=True)
                    ])
                ]
            )
        }
        self.data_width = 32

class DocSoc():
    def __init__(self):
        self.irq = DocIrq()
        self.csr = DocCsr()
        self.constants = {
            "ACLK" : 800_000_000
        }
        self.mem_regions = {
            "sram" : SoCRegion(
                origin=0x6000_0000,
                size = 0x100_0000,
                mode="rw",
                cached=True
            )
        }

def main():
    parser = argparse.ArgumentParser(description="Extract SVD from SCE design")
    parser.add_argument(
        "--path", required=False, help="Path to SCE data", type=str, default="do_not_checkin/crypto/")
    args = parser.parse_args()

    doc_soc = DocSoc()

    sce_path = Path(args.path)

    # generate SVD
    if False:
        svd = get_csr_svd(doc_soc, vendor="cramium", name="soc", description="Cramium SoC, product name TBD")
        print(svd)

    # generate C header
    if False:
        reg_header = get_csr_header(doc_soc.csr.regions, doc_soc.constants)
        print(reg_header)
        mem_header = get_mem_header(doc_soc.mem_regions)
        print(mem_header)
        const_header = get_soc_header(doc_soc.constants)
        print(const_header)

if __name__ == "__main__":
    main()
    exit(0)