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
import ast
import operator as op
import pprint

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
            # "test": 1
        }

class DocCsr():
    def __init__(self):
        self.regions = {
            # "test_reg" : SoCCSRRegion(
            #     0x4000_0000,
            #     32,
            #     [
            #         CSRStatus(
            #             name="fifo_status",
            #             fields=[
            #             CSRField(name="rx_words", size=11, description="Number of words available to read"),
            #             CSRField(name="tx_words", size=11, description="Number of words pending in write FIFO. Free space is {} - `tx_avail`".format(10)),
            #             CSRField(name="abort_in_progress", size=1, description="This bit is set if an `aborting` event was initiated and is still in progress."),
            #             CSRField(name="abort_ack", size=1,
            #             description="""This bit is set by the peer that acknowledged the incoming abort
            # (the later of the two, in case of an imperfect race condition). The abort response handler should
            # check this bit; if it is set, no new acknowledgement shall be issued. The bit is cleared
            # when an initiator initiates a new abort. The initiator shall also ignore the state of this
            # bit if it is intending to initiate a new abort cycle."""),
            #             CSRField(name="tx_err", size=1, description="Set if the write FIFO overflowed because we wrote too much data. Cleared on register read."),
            #             CSRField(name="rx_err", size=1, description="Set if read FIFO underflowed because we read too much data. Cleared on register read."),
            #         ]),
            #         CSRStorage(
            #             name="abort",
            #             fields=[
            #                 CSRField(name="abort", size=1, description=
            #                 """Write `1` to this field to both initiate and acknowledge an abort.
            #     Empties both FIFOs, asserts `aborting`, and prevents an interrupt from being generated by
            #     an incoming abort request. New reads & writes are ignored until `aborted` is asserted
            #     from the peer.""", pulse=True)
            #         ])
            #     ]
            # )
        }
        self.data_width = 32

class DocSoc():
    def __init__(self):
        self.irq = DocIrq()
        self.csr = DocCsr()
        self.constants = {
            # "ACLK" : 800_000_000
        }
        self.mem_regions = {
            # "sram" : SoCRegion(
            #     origin=0x6000_0000,
            #     size = 0x100_0000,
            #     mode="rw",
            #     cached=True
            # )
        }
        self._submodules = {}

# from https://stackoverflow.com/questions/2319019/using-regex-to-remove-comments-from-source-files
def remove_comments(string):
    pattern = r"(\".*?\"|\'.*?\')|(/\*.*?\*/|//[^\r\n]*$)"
    # first group captures quoted strings (double or single)
    # second group captures comments (//single-line or /* multi-line */)
    regex = re.compile(pattern, re.MULTILINE|re.DOTALL)
    def _replacer(match):
        # if the 2nd group (capturing comments) is not None,
        # it means we have captured a non-quoted (real) comment string.
        if match.group(2) is not None:
            return "" # so we will return empty to remove the comment
        else: # otherwise, we will return the 1st group
            return match.group(1) # captured quoted-string
    return regex.sub(_replacer, string)

# cleans up comma separated entities inside braced expressions. Assumes they are all in one line.
# (awful special case handling of parameter list expansion, does not generalize well)
def cleanup_braces(expr):
    output = []
    signal_list = ''
    for e in expr:
        if '{' in e:
            signal_list += e
            continue
        elif '}' in e:
            signal_list += ', ' + e
            output += [signal_list]
            signal_list = ''
        else:
            if signal_list != '':
                signal_list += ', ' + e
            else:
                output += [e]
    if signal_list != '':
        logging.error(f"Curly brace list did not resolve within a single line, case is not handled: {expr}")
    return output

def try_convert_numeric(element: str):
    if type(element) is int:
        return element
    if type(element) is float:
        if not element.is_integer():
            logging.warning("Rounding {} to {}".format(element, int(element)))
        return int(element)

    if type(element) is not str:
        return None
    if "'" in element:
        s = element.split("'")
        # hex check first, so that the hex digit 'd' can be disambiguated from the decimal specifier 'd'
        if 'h'in s[1]:
            return int(s[1].lstrip('h'), base=16)
        elif 'd' in s[1]:
            return int(s[1].lstrip('d'))
        elif 'b' in s[1]:
            return int(s[1].lstrip('b'), base=2)
        else:
            try:
                return int(s[1])
            except ValueError:
                return None
    else:
        try:
            return int(element)
        except ValueError:
            return None

# ------ mini-math evaluator -------
# from https://stackoverflow.com/questions/2371436/evaluating-a-mathematical-expression-in-a-string
# supported operators
operators = {ast.Add: op.add, ast.Sub: op.sub, ast.Mult: op.mul,
            ast.Div: op.truediv, ast.Pow: op.pow, ast.USub: op.neg}

def eval_expr(expr):
    """
    >>> eval_expr('2^6')
    4
    >>> eval_expr('2**6')
    64
    >>> eval_expr('1 + 2*3**(4^5) / (6 + -7)')
    -5.0
    """
    return eval_(ast.parse(expr, mode='eval').body)

def eval_(node):
    if isinstance(node, ast.Num): # <number>
        return node.n
    elif isinstance(node, ast.BinOp): # <left> <operator> <right>
        return operators[type(node.op)](eval_(node.left), eval_(node.right))
    elif isinstance(node, ast.UnaryOp): # <operator> <operand> e.g., -1
        return operators[type(node.op)](eval_(node.operand))
    else:
        raise TypeError(node)

class Expr():
    # this takes an expression 'e' which is just a string value, and stores it for later evaluation
    # once the full file has been read in
    def __init__(self, e):
        self.expression = e
        self.evaluated = False
        self.eval_result = -1
        self.tokens = re.findall(r"([\w:']*[\.]?[\w:']+\b|[\(\)\+\*\-\/])", self.expression)

    def is_fully_expanded(self):
        for t in self.tokens:
            if type(t) == int:
                continue
            else:
                if t == '+' or t == '-' or t == '/' or t == '*' or t == '**':
                    continue
                elif type(t) == str:
                    continue
                else:
                    return False
        return True

    # this should take the schema, expand the string into tokens, and evaluate any expression to an integer value
    def eval(self, schema, module):
        if self.evaluated:
            return self.eval_result

        if '{' in self.expression:
            logging.warning(f"Not expanding braced expression: {self.expression}")
            self.evaluated = True
            self.eval_result = self.expression
            return self.eval_result
        for (index, t) in enumerate(self.tokens):
            num = try_convert_numeric(t)
            if num is not None:
                self.tokens[index] = num
            else:
                if t == '+' or t == '-' or t == '/' or t == '*' or t == '**':
                    pass # these are handled later
                else:
                    # must be an identifier. Resolve the identifier.
                    ident_path = t.split('::')
                    # could do this recursively, but in practice there's only two cases: it's local, or in one other package.
                    if len(ident_path) == 1:
                        # local identifier
                        # check if it's in the param dict; if not, then return as a token
                        if ident_path[0] in schema[module]['localparam']:
                            expansion = schema[module]['localparam'][ident_path[0]]
                        else:
                            expansion = ident_path[0]
                        # if it's an expression, recurse and expand more
                        if type(expansion) is Expr:
                            self.tokens[index] = expansion.eval(schema, module)
                        else:
                            self.tokens[index] = expansion
                    else:
                        assert(len(ident_path) == 2)
                        # check for out-of-scope packages
                        if ident_path[0] not in schema:
                            logging.warning(f"Package not in scope: {ident_path[0]}, cannot expand")
                            self.tokens[index] = f"Unavailable: {ident_path[0]}::{ident_path[1]}"
                            continue
                        # check if we've reached a terminus
                        if ident_path[1] in schema[ident_path[0]]['localparam']:
                            expansion = schema[ident_path[0]]['localparam'][ident_path[1]]
                        else:
                            expansion = ident_path[1]
                        # recurse if necessary
                        if type(expansion) is Expr:
                            self.tokens[index] = expansion.eval(schema, ident_path[0])
                        else:
                            self.tokens[index] = expansion
        if self.is_fully_expanded():
            # condense tokens into a string again
            eval_str = ''
            for t in self.tokens:
                if type(t) is int:
                    eval_str += str(t)
                    eval_str += ' '
                else:
                    eval_str += t
                    eval_str += ' '
            try:
                # if it's a numerical expression, reduce it
                self.eval_result = eval_expr(eval_str)
                self.evaluated = True
                return self.eval_result
            except:
                # logging.error(f"Couldn't evaluate expression:\n  {self.expression}\ntoken tree:\n  {self.tokens}")
                # it's a string that represents a token of some type. The result is that string.
                self.evaluated = True
                self.eval_result = self.expression
                return self.eval_result
        else:
            self.eval(schema, module)

    def as_str(self):
        if self.evaluated:
            return str(self.eval_result)
        else:
            return self.expression

# localparam format notes:
# localparam <[array_size]> <type> <[bit_width]> ident = expr;
#  - if only one bracketed expression, ignore
#  - if no bracketed expression, it's just ident = expr
#  - bitwidth can generally be ignored
def extract_localparam(schema, module, code_line):
    # this matcher keys off of '=', allows zero or more whitespaces to create the last two groups
    # the tricky bit is getting rid of all the type information before the lhs. We do this with the
    # '\s([\S]+)' match group pulling in ' ' as part of the group. Thus, there *must* be a space before
    # the lhs token and any type definition.
    # So this would not work: "localparam foo [7:0]bogus = bar;" because it's missing a space after the ']'
    # TODO: revise this to use \b, "word boundary" instead of the explicit \s

    # sometimes multiple params are put on one line, this handles that case
    plist = code_line.rstrip().rstrip(';').split(';')
    for p in plist:
        matcher = re.compile('localparam(.*)\s([\S]+)\s*=\s*(.*)')
        maybe_match = matcher.match(p.lstrip())
        if maybe_match is None:
            if p != '': # sometimes we have trailing space after the ;, don't flag an error on this
                logging.error(f"localparameter did not extract: {p}")
            return
        groups = maybe_match.groups()
        if len(groups) != 3:
            logging.error(f"localparemeter didn't get expected number of groups (got {len(groups)}): {p}")
            return
        lhs = groups[1]
        rhs = groups[2]
        logging.debug(f"localparam extract lhs {lhs} | rhs {rhs}")
        schema[module]['localparam'][lhs] = Expr(rhs)

def extract_parameter(schema, module, code_line):
    matcher = re.compile('parameter(.*)\s([\S]+)\s*=\s*(.*)')
    code_line = code_line.rstrip(',') # don't care about the trailing comma
    maybe_match = matcher.match(code_line)
    if maybe_match is None:
        logging.error(f"Parameter did not extract: {code_line}")
        return
    groups = maybe_match.groups()
    if len(groups) != 3:
        logging.error(f"Parameter didn't get expected number of groups (got {len(groups)}): {code_line}")
        return
    lhs = groups[1]
    rhs = groups[2]
    logging.debug(f"parameter extract lhs {lhs} | rhs {rhs}")
    schema[module]['localparam'][lhs] = Expr(rhs)

def expand_param_list(params):
        arg_re = re.compile('[.](.+?)\((.+)\)')
        expanded_params = {}
        for param in params:
            exprs = arg_re.match(param)
            if exprs is None:
                if param != '.prdata32()' and param != '.*':
                    logging.error(f"Error parsing argument expression {param}, ignoring!!")
                continue
            p_name = exprs.group(1)
            p_expr = Expr(exprs.group(2))
            expanded_params[p_name] = p_expr
        return expanded_params

def extract_bitwidth(schema, module, code_line):
    bw_re = re.compile('[\s]*(bit|logic)[\s]*(\[.*\])*(.*)')
    matches = bw_re.search(code_line.strip(';'))
    if matches is not None:
        bw = matches.group(2)
        names = matches.group(3).split(',')
        if bw is None:
            for name in names:
                schema[module]['localparam'][name.strip()] = 1
        else:
            bracket_re = re.compile('\[(.*):(.*)\]')
            bracket_matches = bracket_re.search(bw)
            if bracket_matches is not None:
                try:
                    width = int(bracket_matches.group(1)) - int(bracket_matches.group(2)) + 1
                    for name in names:
                        schema[module]['localparam'][name.strip()] = width
                except:
                    logging.debug(f"bit expression not handled: {bw}, not creating an entry")

def add_reg(schema, module, code_line):
    REGEX = '(apb_[c,f,a,s]r)\s#\((.+)\)\s(.+)\s\((.+)\);'
    line_matcher = re.match(REGEX, code_line)
    if line_matcher is None:
        logging.error('Regex match error (this is a script bug, regex needs to be fixed)')
        logging.error(f'Line:  {code_line}')
        logging.error(f'regex: {REGEX}')
    else:
        apb_type = line_matcher.group(1).strip()
        # create a list by mapping 'str.strip' onto the result of splitting the line_matcher's respective group on the ',' character
        # 'it made sense at the time' :-P
        params = cleanup_braces(list(map(str.strip, line_matcher.group(2).split(','))))
        reg_name = line_matcher.group(3).strip()
        args = cleanup_braces(list(map(str.strip, line_matcher.group(4).split(','))))
        #if module == 'aes':
        #    print(f'type: {apb_type}, params: {params}, reg_name: {reg_name}, args: {args}')
        if apb_type != 'apb_cr' and apb_type != 'apb_fr' and apb_type != 'apb_sr' and apb_type != 'apb_ar':
            logging.error(f"Parse error extracting APB register type: unrecognized register macro {apb_type}, ignoring!!!")
            return

        schema[module][apb_type][reg_name] = {
            'params' : expand_param_list(params),
            'args': expand_param_list(args),
        }

def is_m_or_p_empty(m_or_p):
    # we don't evaluate 'sfr_bank' because that's dependent on the apb_* not being empty
    if len(m_or_p['localparam']) == 0 \
        and len(m_or_p['apb_cr']) == 0 \
        and len(m_or_p['apb_sr']) == 0 \
        and len(m_or_p['apb_fr']) == 0 \
        and len(m_or_p['apb_ar']) == 0:
        return True
    else:
        return False

def eval_tree(tree, schema, module, level=0, do_print=False):
    if type(tree) is dict:
        for (k, v) in tree.items():
            #if k == 'localparam':
            #    continue
            if type(v) is dict:
                if len(v) == 0:
                    continue
                if do_print:
                    print(' ' * level + f'{k}:')
                eval_tree(v, schema, module, level + 2, do_print)
            else:
                if isinstance(v, Expr):
                    v.eval(schema, module)
                    if do_print:
                        print(' ' * level + f'{k}:{v.as_str()}')
                else:
                    if do_print:
                        print(' ' * level + f'{k}:{v}')

    else:
        if isinstance(tree, Expr):
            tree.eval(schema, module)
            if do_print:
                print(' ' * level + f'{tree.as_str()}')
        else:
            if do_print:
                print(' ' * level + f'{tree}')

def create_csrs(doc_soc, schema, module, banks, ctrl_offset=0x4002_8000):
    regtypes = ['cr', 'sr', 'fr', 'ar']
    regdescs = {
        'cr': ' read/write control register',
        'sr': ' read only status register',
        'ar': ' performs action on write of value: ',
        'fr': ' flag register. `1` means event happened, write back `1` in respective bit position to clear the flag',
    }
    regfuncs = {
        'cr': CSRStorage,
        'sr': CSRStatus,
        'ar': CSRStorage,
        'fr': CSRStatus,
    }
    if module in banks:
        csrs = []
        for (bank, leaves) in schema[module].items():
            for rtype in regtypes:
                if bank == 'apb_' + rtype:
                    for (leaf_name, leaf_desc) in leaves.items():
                        if 'sfrs' in leaf_desc:
                            the_arg = leaf_desc['args'][rtype]
                            sfr_dict = leaf_desc['sfrs']
                            for (sfr_name, sfr_offset) in sfr_dict.items():
                                if type(the_arg.eval_result) is int:
                                    fname = leaf_name
                                else:
                                    # clean up strings to be valid python identifiers
                                    fname = the_arg.as_str().replace('.', '_').strip() # . is not valid in identifiers, nor are trailing spaces
                                    if '[' in fname:
                                        fname = fname.split('[')[0]
                                assert(rtype != 'ar') # we don't handle 'ar' type with SFRCNT idiom
                                fields = [
                                    CSRField(
                                        name= sfr_name,
                                        size= leaf_desc['params']['DW'].eval_result,
                                        description= fname + regdescs[rtype],
                                    )
                                ]
                                if isinstance(sfr_offset, Expr):
                                    offset = sfr_offset.eval_result
                                else:
                                    offset = sfr_offset
                                csrs += [regfuncs[rtype](
                                    name=leaf_name + '_' + sfr_name,
                                    n=int((leaf_desc['params']['A'].eval_result / 4) + offset),
                                    fields=fields,
                                )]
                        else:
                            the_arg = leaf_desc['args'][rtype]
                            fields = []
                            if '{' in the_arg.as_str():
                                base_str = the_arg.as_str().replace('{', '').replace('}', '')
                                bitfields = base_str.split(',')
                                for bf in reversed(bitfields):
                                    bf = bf.strip()
                                    if bf in schema[module]['localparam']:
                                        bitwidth = schema[module]['localparam'][bf]
                                    else:
                                        logging.warning(f"{bf} can't be found, assuming width=1. Manual check is necessary!")
                                        bitwidth = 1
                                    assert(rtype != 'ar') # we don't expect multi-bit ar defs
                                    fields += [
                                        CSRField(
                                            name= bf.replace('.', '_').strip(),
                                            size= bitwidth,
                                            description= bf + regdescs[rtype],
                                        )
                                    ]
                            else:
                                if type(the_arg.eval_result) is int:
                                    fname = leaf_name
                                else:
                                    # clean up strings to be valid python identifiers
                                    fname = the_arg.as_str().replace('.', '_').strip() # . is not valid in identifiers, nor are trailing spaces
                                    if '[' in fname:
                                        fname = fname.split('[')[0]
                                if rtype == 'ar':
                                    if 'DW' in leaf_desc['params']:
                                        ar_size = leaf_desc['params']['DW'].eval_result
                                    else:
                                        ar_size = 32
                                    raw_action = leaf_desc['params']['AR'].eval_result
                                    if type(raw_action) is int:
                                        action = hex(raw_action)
                                    else:
                                        action = raw_action
                                    fields += [
                                        CSRField(
                                            name= fname,
                                            size= ar_size,
                                            description= fname + regdescs[rtype] + action,
                                            pulse=True
                                        )
                                    ]
                                else:
                                    fields += [
                                        CSRField(
                                            name= fname,
                                            size= leaf_desc['params']['DW'].eval_result,
                                            description= fname + regdescs[rtype],
                                        )
                                    ]
                            csrs += [regfuncs[rtype](
                                name=leaf_name,
                                n=int(leaf_desc['params']['A'].eval_result / 4),
                                fields=fields,
                            )]
        doc_soc.csr.regions[module] = SoCCSRRegion(
            ctrl_offset + banks[module] * 0x1000,
            32,
            csrs
        )
    else:
        count = 0
        for rtype in regtypes:
            for (bank, leaves) in schema[module].items():
                if bank == 'apb_' + rtype:
                    count += len(leaves)
        if count != 0:
            logging.warning(f"Registers were discovered that do not have a top-level address mapping: {module}, {count} total orphaned registers")

def main():
    parser = argparse.ArgumentParser(description="Extract SVD from SCE design")
    parser.add_argument(
        "--path", required=False, help="Path to SCE data", type=str, default="./soc_mpw/rtl/crypto")
    parser.add_argument(
        "--loglevel", required=False, help="set logging level (INFO/DEBUG/WARNING/ERROR)", type=str, default="INFO",
    )
    parser.add_argument(
        "--outdir", required=False, help="Path to output files", type=str, default="include/"
    )
    args = parser.parse_args()
    numeric_level = getattr(logging, args.loglevel.upper(), None)
    if not isinstance(numeric_level, int):
        raise ValueError('Invalid log level: %s' % args.loglevel)
    logging.basicConfig(level=numeric_level)

    if not Path(args.path).exists():
        logging.error("Design directory not found. Script should be invoked from project root as python3 ./codegen/sce_to_svd.py!")
        exit(0)

    pp = pprint.PrettyPrinter(indent=2, sort_dicts=False)

    doc_soc = DocSoc()

    sce_path = Path(args.path).glob('**/*')
    sce_files = [x for x in sce_path if x.is_file()]

    ### use only the latest version, as extracted by numerical order
    versioned_files = {}
    for file in sce_files:
        version_matcher = re.match('(.*)_v([0-9].[0-9]).sv$', file.name)
        if version_matcher is None:
            if (file.stem.endswith('.v') or file.stem.endswith('.sv')) and not file.stem.startswith('.'):
                versioned_files[file.stem] = (file, 0.0)  # file path, version. 0 means no version
        else:
            if not file.stem.startswith('.'):
                basename = version_matcher.group(1)
                version = float(version_matcher.group(2))
                if basename not in versioned_files:
                    versioned_files[basename] = (file, version)
                else:
                    (_oldfile, old_version) = versioned_files[basename]
                    if version > old_version:
                        versioned_files[basename] = (file, version)

    versioned_files = dict(sorted(versioned_files.items(), key=lambda x: x[0]))
    logging.debug("Using the following sources based on version numbering:")
    for (k, v) in versioned_files.items():
        logging.debug('  - {}:{}'.format(k, v))

    # ------- extract the general schema of the code ----------
    schema = {}
    for (_file_root, (file, _version)) in versioned_files.items():
        with open(file, "r") as sv_file:
            lines = sv_file.readlines()
            mod_or_pkg = ''
            multi_line_param = ''
            state = 'IDLE'
            for line in lines:
                if state == 'IDLE':
                    # TODO: handle 'typedef enum' case and extract as localparam
                    if line.lstrip().startswith('module') or line.lstrip().startswith('package'):
                        # names are "dirty" if there isn't a space following the mod or package decl
                        # but in practice the ones we care about are well-formed, so we leave this issue hanging.
                        mod_or_pkg = line.split()[1].strip().strip('();#')
                        state = 'ACTIVE'
                        if mod_or_pkg not in schema:
                            schema[mod_or_pkg] = {
                                'localparam' : {},
                                'apb_cr' : {},
                                'apb_sr' : {},
                                'apb_fr' : {},
                                'apb_ar' : {},
                            }
                elif state == 'ACTIVE':
                    if line.lstrip().startswith('endmodule') or line.lstrip().startswith('endpackage'):
                        state = 'IDLE'
                        mod_or_pkg = ''
                    else:
                        code_line = remove_comments(line.strip()).lstrip()
                        if re.match('^apb_[csfa]r', code_line):
                            add_reg(schema, mod_or_pkg, code_line)
                        elif code_line.startswith('localparam'):
                            # simple one line case
                            if code_line.strip().endswith(';'):
                                extract_localparam(schema, mod_or_pkg, code_line)
                            else:
                                state = 'PARAM'
                                multi_line_param += code_line
                        elif code_line.startswith('parameter'):
                            extract_parameter(schema, mod_or_pkg, code_line)
                        elif code_line.startswith('logic') or code_line.startswith('bit'):
                            extract_bitwidth(schema, mod_or_pkg, code_line)
                elif state == 'PARAM':
                    code_line = remove_comments(line.strip()).lstrip()
                    if code_line.strip().endswith(';'):
                        multi_line_param += code_line
                        extract_localparam(schema, mod_or_pkg, multi_line_param)
                        multi_line_param = ''
                        state = 'ACTIVE'
                    else:
                        multi_line_param += code_line

    # --------- extract bank number from the top level file ---------
    (top_file, _version) = versioned_files['sce']
    banks = {}
    with open(top_file, 'r') as top:
        multi_line_expr = ''
        for line in top:
            code_line = remove_comments(line.strip()).lstrip()
            split_at_semi = code_line.strip().split(';')
            if len(split_at_semi) == 1 and 'endgenerate' not in code_line:
                multi_line_expr += code_line
                multi_line_expr += ' '
            else:
                multi_line_expr += split_at_semi[0]
                # process the expression
                multi_line_expr = multi_line_expr.strip()
                apbs_re = re.compile(r"\.apbs(.*?)\(.*?apbs\[([0-9])\]")
                # print(multi_line_expr)
                matches = apbs_re.search(multi_line_expr)
                if matches is not None:
                    # print(multi_line_expr)
                    bank = matches.group(2)
                    name = multi_line_expr.split(' ')[0]
                    # print(f'{name} is at {bank}')
                    banks[name] = int(bank)
                # now reset the search
                if len(split_at_semi) > 1:
                    multi_line_expr = split_at_semi[1]
                else:
                    multi_line_expr = ''
    print("Register banks discovered:")
    pp.pprint(banks)

    # --------- extract SFRCNT from files that contain SFRCNT record ---------
    for (module, leaves) in schema.items():
        sfr_count = 0
        sfr_name = ''
        sfr_module = ''
        for (cr_name, cr_defs) in leaves['apb_cr'].items():
            if 'params' in cr_defs:
                if 'SFRCNT' in cr_defs['params']:
                    sfr_count = cr_defs['params']['SFRCNT'].eval(schema, module)
                    sfr_name = cr_defs['args']['cr'].eval(schema, module)
                    if type(sfr_name) != str: # if a name maps to an expression that is fully evaluated to a number, prefer the expression
                        sfr_name = cr_defs['args']['cr'].expression
                    sfr_module = module

                    print(f"SFR found, {sfr_module}:{sfr_name}[{sfr_count}]")
                    (sfr_file, _version) = versioned_files[sfr_module]
                    re_pattern = sfr_name + '\[(.*)\]'
                    sfr_re = re.compile(re_pattern)

                    # Try to infer the names of the sub-SFRs by searching for the SFR's bracketed references in the code
                    with open(sfr_file, 'r') as sfr_f:
                        cr_defs['sfrs'] = {}
                        for line in sfr_f:
                            matches = sfr_re.search(line)
                            if matches is not None:
                                sfr_item = matches.group(1)
                                if ':' in sfr_item: # bit range in SFR
                                    # might need more parsing if the bit range contains exprsessions, but let's
                                    # see if we can get away for now with just stating the offset is 0
                                    # sfr_item = 'bits' + sfr_item.replace(':', '_')
                                    cr_defs['sfrs']['bits'] = 0
                                else: # symbolic name for SFR
                                    cr_defs['sfrs'][sfr_item] = leaves['localparam'][sfr_item]

    # --------- SPECIAL CASE: check `generate` for SFR ar inside sce_glbsfr ---------
    module = 'sce_glbsfr'
    leaves = schema[module]
    for (sr_name, sr_defs) in leaves['apb_sr'].items():
        if 'params' in sr_defs:
            if 'SFRCNT' in sr_defs['params']:
                sfr_count = sr_defs['params']['SFRCNT'].eval(schema, module)
                sfr_name = sr_defs['args']['sr'].eval(schema, module)
                sfr_module = module
                if sr_name != 'sfr_ffcnt':
                    logging.error("Manual fixup for sfr_ffclr has failed. Check code carefully!")
                sr_defs['sfrs'] = {}
                for i in range(sfr_count):
                    sr_defs['sfrs'][sfr_name + str(i)] = i

    # TODO:
    #  - process segid from localparams

    print("done parsing")
    # Setup the memory region that the CSRs are destined for. This is manually extracted from the top-level docs.
    doc_soc.mem_regions['sce'] = SoCRegion(
        origin=0x4002_8000,
        size=0x8000,
        mode='rw',
        cached=False
    )
    # ----------- print the tree and create CSRs
    for (module, leaves) in schema.items():
        eval_tree(leaves, schema, module, level=2, do_print=False)
        # ctrl_offset is the base of the SCE register set, as extracted from the core documentation
        create_csrs(doc_soc, schema, module, banks, ctrl_offset=doc_soc.mem_regions['sce'].origin)

    # sort the CSR objects according to their 'n' so they appear in the correct locations in the generated files
    from litex.soc.interconnect.csr import _sort_gathered_items
    from litex.soc.interconnect.csr import CSR
    for region in doc_soc.csr.regions.values():
        csr_list = []
        unsorted_csrs = region.obj
        # find max n
        n = 0
        for item in unsorted_csrs:
            if item.n > n:
                n = item.n
        # build a list of "reserved" CSRs
        for i in range(n+1):
            csr_list += [CSR(name=f"reserved{i}")]
        # displace the reserved items with allocated items
        for item in unsorted_csrs:
            csr_list[item.n] = item
        # convert to dictionary
        region.obj = csr_list

    # generate SVD
    with open(args.outdir + 'sce.svd', 'w') as svd_f:
        svd = get_csr_svd(doc_soc, vendor="cramium", name="soc", description="Cramium SoC, product name TBD")
        svd_f.write(svd)

    # generate C header
    with open(args.outdir + 'sce.h', 'w') as header_f:
        reg_header = get_csr_header(doc_soc.csr.regions, doc_soc.constants)
        header_f.write(reg_header)
        mem_header = get_mem_header(doc_soc.mem_regions)
        header_f.write(mem_header)
        const_header = get_soc_header(doc_soc.constants)
        header_f.write(const_header)

    from litex.soc.doc import generate_docs
    doc_dir = args.outdir + 'doc/'
    generate_docs(doc_soc, doc_dir, project_name="Cramium SCE module", author="Cramium, Inc.")

    subprocess.run(['cargo', 'run', '../include/sce.svd' , '../include/sce_generated.rs'], cwd='./svd2utra')
    subprocess.run(['sphinx-build', '-M', 'html', 'include/doc/', 'include/doc/_build'])
    # subprocess.run(['rsync', '-a', '--delete', 'include/doc/_build/html/', 'bunnie@ci.betrusted.io:/var/sce/'])

if __name__ == "__main__":
    main()
    exit(0)

module_schema = {
    'aes' : {
        'localparam' : {
            'AF_ENC': Expr('1'),
            'AF_DEC': Expr('2'),
            # other params
            'PTRID_IV' : 0,
            'PTRID_AKEY' : 0,
        },
        'apb_cr' : {
            'sfr_crfunc' : {
                'params' : {
                    'A' : 0x0,
                    'DW' : 8,
                },
                'args' : {
                    'cr' : 'cr_func',
                    'prdata32' : '',
                }
            },
            'sfr_segptr' : {
                'params' : {
                    'A' : 0x30,
                    'DW' : ['scedma_pkg', 'AW'], # paths are turned into lists
                    'SFRCNT': 4,
                },
                'cr' : 'cr_segptrstart',
                'prdata32' : '',
            }
        },
        # other apb types
        # ...
        # sfr_bank is an entry for banked SFR tables. This is made every time a 'SFRCNT' record is encountered, and the key is the corresponding 'cr' name
        'sfr_bank' : {
            'cr_segptrstart' : {
                ['PTRID_AKEY', 'PTRID_AIB', 'PTRID_IV', 'PTRID_AOB']
            }
        }
    },
    'scedma_pkg' : {
        'localparam' : {
            'SEGID_LKEY', Expr('0x0'),
            'SEGID_KEY', Expr("SEGID_LKEY + 'd1"),
        }
    }
}