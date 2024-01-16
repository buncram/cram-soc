PKE
===

Register Listing for PKE
------------------------

+--------------------------------------------------------------+-----------------------------------------------+
| Register                                                     | Address                                       |
+==============================================================+===============================================+
| :ref:`PKE_SFR_CRFUNC <PKE_SFR_CRFUNC>`                       | :ref:`0x4002c000 <PKE_SFR_CRFUNC>`            |
+--------------------------------------------------------------+-----------------------------------------------+
| :ref:`PKE_SFR_AR <PKE_SFR_AR>`                               | :ref:`0x4002c004 <PKE_SFR_AR>`                |
+--------------------------------------------------------------+-----------------------------------------------+
| :ref:`PKE_SFR_SRMFSM <PKE_SFR_SRMFSM>`                       | :ref:`0x4002c008 <PKE_SFR_SRMFSM>`            |
+--------------------------------------------------------------+-----------------------------------------------+
| :ref:`PKE_SFR_FR <PKE_SFR_FR>`                               | :ref:`0x4002c00c <PKE_SFR_FR>`                |
+--------------------------------------------------------------+-----------------------------------------------+
| :ref:`PKE_SFR_OPTNW <PKE_SFR_OPTNW>`                         | :ref:`0x4002c010 <PKE_SFR_OPTNW>`             |
+--------------------------------------------------------------+-----------------------------------------------+
| :ref:`PKE_SFR_OPTEW <PKE_SFR_OPTEW>`                         | :ref:`0x4002c014 <PKE_SFR_OPTEW>`             |
+--------------------------------------------------------------+-----------------------------------------------+
| :ref:`PKE_SFR_OPTRW <PKE_SFR_OPTRW>`                         | :ref:`0x4002c018 <PKE_SFR_OPTRW>`             |
+--------------------------------------------------------------+-----------------------------------------------+
| :ref:`PKE_SFR_OPTLTX <PKE_SFR_OPTLTX>`                       | :ref:`0x4002c01c <PKE_SFR_OPTLTX>`            |
+--------------------------------------------------------------+-----------------------------------------------+
| :ref:`PKE_SFR_OPTMASK <PKE_SFR_OPTMASK>`                     | :ref:`0x4002c020 <PKE_SFR_OPTMASK>`           |
+--------------------------------------------------------------+-----------------------------------------------+
| :ref:`PKE_SFR_SEGPTR_PTRID_PCON <PKE_SFR_SEGPTR_PTRID_PCON>` | :ref:`0x4002c030 <PKE_SFR_SEGPTR_PTRID_PCON>` |
+--------------------------------------------------------------+-----------------------------------------------+
| :ref:`PKE_SFR_SEGPTR_PTRID_PIB0 <PKE_SFR_SEGPTR_PTRID_PIB0>` | :ref:`0x4002c034 <PKE_SFR_SEGPTR_PTRID_PIB0>` |
+--------------------------------------------------------------+-----------------------------------------------+
| :ref:`PKE_SFR_SEGPTR_PTRID_PIB1 <PKE_SFR_SEGPTR_PTRID_PIB1>` | :ref:`0x4002c038 <PKE_SFR_SEGPTR_PTRID_PIB1>` |
+--------------------------------------------------------------+-----------------------------------------------+
| :ref:`PKE_SFR_SEGPTR_PTRID_PKB <PKE_SFR_SEGPTR_PTRID_PKB>`   | :ref:`0x4002c03c <PKE_SFR_SEGPTR_PTRID_PKB>`  |
+--------------------------------------------------------------+-----------------------------------------------+
| :ref:`PKE_SFR_SEGPTR_PTRID_POB <PKE_SFR_SEGPTR_PTRID_POB>`   | :ref:`0x4002c040 <PKE_SFR_SEGPTR_PTRID_POB>`  |
+--------------------------------------------------------------+-----------------------------------------------+

PKE_SFR_CRFUNC
^^^^^^^^^^^^^^

`Address: 0x4002c000 + 0x0 = 0x4002c000`

    See file:///F:/code/cram-soc/soc-oss/rtl/crypto/pke_v0.3tmp.sv

    .. wavedrom::
        :caption: PKE_SFR_CRFUNC

        {
            "reg": [
                {"name": "cr_func",  "bits": 8},
                {"name": "cr_pcoreir",  "bits": 8},
                {"bits": 16}
            ], "config": {"hspace": 400, "bits": 32, "lanes": 1 }, "options": {"hspace": 400, "bits": 32, "lanes": 1}
        }


+--------+------------+----------------------------------------+
| Field  | Name       | Description                            |
+========+============+========================================+
| [7:0]  | CR_FUNC    | cr_func read/write control register    |
+--------+------------+----------------------------------------+
| [15:8] | CR_PCOREIR | cr_pcoreir read/write control register |
+--------+------------+----------------------------------------+

PKE_SFR_AR
^^^^^^^^^^

`Address: 0x4002c000 + 0x4 = 0x4002c004`

    See file:///F:/code/cram-soc/soc-oss/rtl/crypto/pke_v0.3tmp.sv

    .. wavedrom::
        :caption: PKE_SFR_AR

        {
            "reg": [
                {"name": "sfr_ar",  "type": 4, "bits": 32}
            ], "config": {"hspace": 400, "bits": 32, "lanes": 1 }, "options": {"hspace": 400, "bits": 32, "lanes": 1}
        }


+--------+--------+------------------------------------------------+
| Field  | Name   | Description                                    |
+========+========+================================================+
| [31:0] | SFR_AR | sfr_ar performs action on write of value: 0x5a |
+--------+--------+------------------------------------------------+

PKE_SFR_SRMFSM
^^^^^^^^^^^^^^

`Address: 0x4002c000 + 0x8 = 0x4002c008`

    See file:///F:/code/cram-soc/soc-oss/rtl/crypto/pke_v0.3tmp.sv

    .. wavedrom::
        :caption: PKE_SFR_SRMFSM

        {
            "reg": [
                {"name": "mfsm",  "bits": 8},
                {"name": "modinvready",  "bits": 1},
                {"bits": 23}
            ], "config": {"hspace": 400, "bits": 32, "lanes": 4 }, "options": {"hspace": 400, "bits": 32, "lanes": 4}
        }


+-------+-------------+---------------------------------------+
| Field | Name        | Description                           |
+=======+=============+=======================================+
| [7:0] | MFSM        | mfsm read only status register        |
+-------+-------------+---------------------------------------+
| [8]   | MODINVREADY | modinvready read only status register |
+-------+-------------+---------------------------------------+

PKE_SFR_FR
^^^^^^^^^^

`Address: 0x4002c000 + 0xc = 0x4002c00c`

    See file:///F:/code/cram-soc/soc-oss/rtl/crypto/pke_v0.3tmp.sv

    .. wavedrom::
        :caption: PKE_SFR_FR

        {
            "reg": [
                {"name": "mfsm_done",  "bits": 1},
                {"name": "pcore_done",  "bits": 1},
                {"name": "chnlo_done",  "bits": 1},
                {"name": "chnli_done",  "bits": 1},
                {"name": "chnlx_done",  "bits": 1},
                {"bits": 27}
            ], "config": {"hspace": 400, "bits": 32, "lanes": 4 }, "options": {"hspace": 400, "bits": 32, "lanes": 4}
        }


+-------+------------+----------------------------------------------------------------------------------+
| Field | Name       | Description                                                                      |
+=======+============+==================================================================================+
| [0]   | MFSM_DONE  | mfsm_done flag register. `1` means event happened, write back `1` in respective  |
|       |            | bit position to clear the flag                                                   |
+-------+------------+----------------------------------------------------------------------------------+
| [1]   | PCORE_DONE | pcore_done flag register. `1` means event happened, write back `1` in respective |
|       |            | bit position to clear the flag                                                   |
+-------+------------+----------------------------------------------------------------------------------+
| [2]   | CHNLO_DONE | chnlo_done flag register. `1` means event happened, write back `1` in respective |
|       |            | bit position to clear the flag                                                   |
+-------+------------+----------------------------------------------------------------------------------+
| [3]   | CHNLI_DONE | chnli_done flag register. `1` means event happened, write back `1` in respective |
|       |            | bit position to clear the flag                                                   |
+-------+------------+----------------------------------------------------------------------------------+
| [4]   | CHNLX_DONE | chnlx_done flag register. `1` means event happened, write back `1` in respective |
|       |            | bit position to clear the flag                                                   |
+-------+------------+----------------------------------------------------------------------------------+

PKE_SFR_OPTNW
^^^^^^^^^^^^^

`Address: 0x4002c000 + 0x10 = 0x4002c010`

    See file:///F:/code/cram-soc/soc-oss/rtl/crypto/pke_v0.3tmp.sv

    .. wavedrom::
        :caption: PKE_SFR_OPTNW

        {
            "reg": [
                {"name": "sfr_optnw",  "bits": 13},
                {"bits": 19}
            ], "config": {"hspace": 400, "bits": 32, "lanes": 1 }, "options": {"hspace": 400, "bits": 32, "lanes": 1}
        }


+--------+-----------+---------------------------------------+
| Field  | Name      | Description                           |
+========+===========+=======================================+
| [12:0] | SFR_OPTNW | sfr_optnw read/write control register |
+--------+-----------+---------------------------------------+

PKE_SFR_OPTEW
^^^^^^^^^^^^^

`Address: 0x4002c000 + 0x14 = 0x4002c014`

    See file:///F:/code/cram-soc/soc-oss/rtl/crypto/pke_v0.3tmp.sv

    .. wavedrom::
        :caption: PKE_SFR_OPTEW

        {
            "reg": [
                {"name": "sfr_optew",  "bits": 13},
                {"bits": 19}
            ], "config": {"hspace": 400, "bits": 32, "lanes": 1 }, "options": {"hspace": 400, "bits": 32, "lanes": 1}
        }


+--------+-----------+---------------------------------------+
| Field  | Name      | Description                           |
+========+===========+=======================================+
| [12:0] | SFR_OPTEW | sfr_optew read/write control register |
+--------+-----------+---------------------------------------+

PKE_SFR_OPTRW
^^^^^^^^^^^^^

`Address: 0x4002c000 + 0x18 = 0x4002c018`

    See file:///F:/code/cram-soc/soc-oss/rtl/crypto/pke_v0.3tmp.sv

    .. wavedrom::
        :caption: PKE_SFR_OPTRW

        {
            "reg": [
                {"name": "sfr_optrw",  "bits": 10},
                {"bits": 22}
            ], "config": {"hspace": 400, "bits": 32, "lanes": 1 }, "options": {"hspace": 400, "bits": 32, "lanes": 1}
        }


+-------+-----------+---------------------------------------+
| Field | Name      | Description                           |
+=======+===========+=======================================+
| [9:0] | SFR_OPTRW | sfr_optrw read/write control register |
+-------+-----------+---------------------------------------+

PKE_SFR_OPTLTX
^^^^^^^^^^^^^^

`Address: 0x4002c000 + 0x1c = 0x4002c01c`

    See file:///F:/code/cram-soc/soc-oss/rtl/crypto/pke_v0.3tmp.sv

    .. wavedrom::
        :caption: PKE_SFR_OPTLTX

        {
            "reg": [
                {"name": "sfr_optltx",  "bits": 5},
                {"bits": 27}
            ], "config": {"hspace": 400, "bits": 32, "lanes": 4 }, "options": {"hspace": 400, "bits": 32, "lanes": 4}
        }


+-------+------------+----------------------------------------+
| Field | Name       | Description                            |
+=======+============+========================================+
| [4:0] | SFR_OPTLTX | sfr_optltx read/write control register |
+-------+------------+----------------------------------------+

PKE_SFR_OPTMASK
^^^^^^^^^^^^^^^

`Address: 0x4002c000 + 0x20 = 0x4002c020`

    See file:///F:/code/cram-soc/soc-oss/rtl/crypto/pke_v0.3tmp.sv

    .. wavedrom::
        :caption: PKE_SFR_OPTMASK

        {
            "reg": [
                {"name": "sfr_optmask",  "bits": 16},
                {"bits": 16}
            ], "config": {"hspace": 400, "bits": 32, "lanes": 1 }, "options": {"hspace": 400, "bits": 32, "lanes": 1}
        }


+--------+-------------+-----------------------------------------+
| Field  | Name        | Description                             |
+========+=============+=========================================+
| [15:0] | SFR_OPTMASK | sfr_optmask read/write control register |
+--------+-------------+-----------------------------------------+

PKE_SFR_SEGPTR_PTRID_PCON
^^^^^^^^^^^^^^^^^^^^^^^^^

`Address: 0x4002c000 + 0x30 = 0x4002c030`

    See file:///F:/code/cram-soc/soc-oss/rtl/crypto/pke_v0.3tmp.sv

    .. wavedrom::
        :caption: PKE_SFR_SEGPTR_PTRID_PCON

        {
            "reg": [
                {"name": "PTRID_PCON",  "bits": 12},
                {"bits": 20}
            ], "config": {"hspace": 400, "bits": 32, "lanes": 1 }, "options": {"hspace": 400, "bits": 32, "lanes": 1}
        }


+--------+------------+--------------------------------------------+
| Field  | Name       | Description                                |
+========+============+============================================+
| [11:0] | PTRID_PCON | cr_segptrstart read/write control register |
+--------+------------+--------------------------------------------+

PKE_SFR_SEGPTR_PTRID_PIB0
^^^^^^^^^^^^^^^^^^^^^^^^^

`Address: 0x4002c000 + 0x34 = 0x4002c034`

    See file:///F:/code/cram-soc/soc-oss/rtl/crypto/pke_v0.3tmp.sv

    .. wavedrom::
        :caption: PKE_SFR_SEGPTR_PTRID_PIB0

        {
            "reg": [
                {"name": "PTRID_PIB0",  "bits": 12},
                {"bits": 20}
            ], "config": {"hspace": 400, "bits": 32, "lanes": 1 }, "options": {"hspace": 400, "bits": 32, "lanes": 1}
        }


+--------+------------+--------------------------------------------+
| Field  | Name       | Description                                |
+========+============+============================================+
| [11:0] | PTRID_PIB0 | cr_segptrstart read/write control register |
+--------+------------+--------------------------------------------+

PKE_SFR_SEGPTR_PTRID_PIB1
^^^^^^^^^^^^^^^^^^^^^^^^^

`Address: 0x4002c000 + 0x38 = 0x4002c038`

    See file:///F:/code/cram-soc/soc-oss/rtl/crypto/pke_v0.3tmp.sv

    .. wavedrom::
        :caption: PKE_SFR_SEGPTR_PTRID_PIB1

        {
            "reg": [
                {"name": "PTRID_PIB1",  "bits": 12},
                {"bits": 20}
            ], "config": {"hspace": 400, "bits": 32, "lanes": 1 }, "options": {"hspace": 400, "bits": 32, "lanes": 1}
        }


+--------+------------+--------------------------------------------+
| Field  | Name       | Description                                |
+========+============+============================================+
| [11:0] | PTRID_PIB1 | cr_segptrstart read/write control register |
+--------+------------+--------------------------------------------+

PKE_SFR_SEGPTR_PTRID_PKB
^^^^^^^^^^^^^^^^^^^^^^^^

`Address: 0x4002c000 + 0x3c = 0x4002c03c`

    See file:///F:/code/cram-soc/soc-oss/rtl/crypto/pke_v0.3tmp.sv

    .. wavedrom::
        :caption: PKE_SFR_SEGPTR_PTRID_PKB

        {
            "reg": [
                {"name": "PTRID_PKB",  "bits": 12},
                {"bits": 20}
            ], "config": {"hspace": 400, "bits": 32, "lanes": 1 }, "options": {"hspace": 400, "bits": 32, "lanes": 1}
        }


+--------+-----------+--------------------------------------------+
| Field  | Name      | Description                                |
+========+===========+============================================+
| [11:0] | PTRID_PKB | cr_segptrstart read/write control register |
+--------+-----------+--------------------------------------------+

PKE_SFR_SEGPTR_PTRID_POB
^^^^^^^^^^^^^^^^^^^^^^^^

`Address: 0x4002c000 + 0x40 = 0x4002c040`

    See file:///F:/code/cram-soc/soc-oss/rtl/crypto/pke_v0.3tmp.sv

    .. wavedrom::
        :caption: PKE_SFR_SEGPTR_PTRID_POB

        {
            "reg": [
                {"name": "PTRID_POB",  "bits": 12},
                {"bits": 20}
            ], "config": {"hspace": 400, "bits": 32, "lanes": 1 }, "options": {"hspace": 400, "bits": 32, "lanes": 1}
        }


+--------+-----------+--------------------------------------------+
| Field  | Name      | Description                                |
+========+===========+============================================+
| [11:0] | PTRID_POB | cr_segptrstart read/write control register |
+--------+-----------+--------------------------------------------+

