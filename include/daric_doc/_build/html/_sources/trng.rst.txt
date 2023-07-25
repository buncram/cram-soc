TRNG
====

Register Listing for TRNG
-------------------------

+--------------------------------------------------------------------------+-----------------------------------------------------+
| Register                                                                 | Address                                             |
+==========================================================================+=====================================================+
| :ref:`TRNG_SFR_CRFUNC <TRNG_SFR_CRFUNC>`                                 | :ref:`0x4002e000 <TRNG_SFR_CRFUNC>`                 |
+--------------------------------------------------------------------------+-----------------------------------------------------+
| :ref:`TRNG_SFR_AR <TRNG_SFR_AR>`                                         | :ref:`0x4002e004 <TRNG_SFR_AR>`                     |
+--------------------------------------------------------------------------+-----------------------------------------------------+
| :ref:`TRNG_SFR_SRMFSM <TRNG_SFR_SRMFSM>`                                 | :ref:`0x4002e008 <TRNG_SFR_SRMFSM>`                 |
+--------------------------------------------------------------------------+-----------------------------------------------------+
| :ref:`TRNG_SFR_FR <TRNG_SFR_FR>`                                         | :ref:`0x4002e00c <TRNG_SFR_FR>`                     |
+--------------------------------------------------------------------------+-----------------------------------------------------+
| :ref:`TRNG_SFR_OPTNW <TRNG_SFR_OPTNW>`                                   | :ref:`0x4002e010 <TRNG_SFR_OPTNW>`                  |
+--------------------------------------------------------------------------+-----------------------------------------------------+
| :ref:`TRNG_SFR_OPTEW <TRNG_SFR_OPTEW>`                                   | :ref:`0x4002e014 <TRNG_SFR_OPTEW>`                  |
+--------------------------------------------------------------------------+-----------------------------------------------------+
| :ref:`TRNG_SFR_OPTMASK <TRNG_SFR_OPTMASK>`                               | :ref:`0x4002e020 <TRNG_SFR_OPTMASK>`                |
+--------------------------------------------------------------------------+-----------------------------------------------------+
| :ref:`TRNG_SFR_SEGPTR_CR_SEGPTRSTART0 <TRNG_SFR_SEGPTR_CR_SEGPTRSTART0>` | :ref:`0x4002e030 <TRNG_SFR_SEGPTR_CR_SEGPTRSTART0>` |
+--------------------------------------------------------------------------+-----------------------------------------------------+
| :ref:`TRNG_SFR_SEGPTR_CR_SEGPTRSTART1 <TRNG_SFR_SEGPTR_CR_SEGPTRSTART1>` | :ref:`0x4002e034 <TRNG_SFR_SEGPTR_CR_SEGPTRSTART1>` |
+--------------------------------------------------------------------------+-----------------------------------------------------+
| :ref:`TRNG_SFR_SEGPTR_CR_SEGPTRSTART2 <TRNG_SFR_SEGPTR_CR_SEGPTRSTART2>` | :ref:`0x4002e038 <TRNG_SFR_SEGPTR_CR_SEGPTRSTART2>` |
+--------------------------------------------------------------------------+-----------------------------------------------------+
| :ref:`TRNG_SFR_SEGPTR_CR_SEGPTRSTART3 <TRNG_SFR_SEGPTR_CR_SEGPTRSTART3>` | :ref:`0x4002e03c <TRNG_SFR_SEGPTR_CR_SEGPTRSTART3>` |
+--------------------------------------------------------------------------+-----------------------------------------------------+
| :ref:`TRNG_SFR_SEGPTR_CR_SEGPTRSTART4 <TRNG_SFR_SEGPTR_CR_SEGPTRSTART4>` | :ref:`0x4002e040 <TRNG_SFR_SEGPTR_CR_SEGPTRSTART4>` |
+--------------------------------------------------------------------------+-----------------------------------------------------+

TRNG_SFR_CRFUNC
^^^^^^^^^^^^^^^

`Address: 0x4002e000 + 0x0 = 0x4002e000`


    .. wavedrom::
        :caption: TRNG_SFR_CRFUNC

        {
            "reg": [
                {"name": "cr_func",  "bits": 8},
                {"bits": 24}
            ], "config": {"hspace": 400, "bits": 32, "lanes": 1 }, "options": {"hspace": 400, "bits": 32, "lanes": 1}
        }


+-------+---------+-------------------------------------+
| Field | Name    | Description                         |
+=======+=========+=====================================+
| [7:0] | CR_FUNC | cr_func read/write control register |
+-------+---------+-------------------------------------+

TRNG_SFR_AR
^^^^^^^^^^^

`Address: 0x4002e000 + 0x4 = 0x4002e004`


    .. wavedrom::
        :caption: TRNG_SFR_AR

        {
            "reg": [
                {"name": "start",  "type": 4, "bits": 32}
            ], "config": {"hspace": 400, "bits": 32, "lanes": 1 }, "options": {"hspace": 400, "bits": 32, "lanes": 1}
        }


+--------+-------+-----------------------------------------------+
| Field  | Name  | Description                                   |
+========+=======+===============================================+
| [31:0] | START | start performs action on write of value: 0x5a |
+--------+-------+-----------------------------------------------+

TRNG_SFR_SRMFSM
^^^^^^^^^^^^^^^

`Address: 0x4002e000 + 0x8 = 0x4002e008`


    .. wavedrom::
        :caption: TRNG_SFR_SRMFSM

        {
            "reg": [
                {"name": "mfsm",  "bits": 1},
                {"name": "modinvready",  "bits": 1},
                {"bits": 30}
            ], "config": {"hspace": 400, "bits": 32, "lanes": 4 }, "options": {"hspace": 400, "bits": 32, "lanes": 4}
        }


+-------+-------------+---------------------------------------+
| Field | Name        | Description                           |
+=======+=============+=======================================+
| [0]   | MFSM        | mfsm read only status register        |
+-------+-------------+---------------------------------------+
| [1]   | MODINVREADY | modinvready read only status register |
+-------+-------------+---------------------------------------+

TRNG_SFR_FR
^^^^^^^^^^^

`Address: 0x4002e000 + 0xc = 0x4002e00c`


    .. wavedrom::
        :caption: TRNG_SFR_FR

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

TRNG_SFR_OPTNW
^^^^^^^^^^^^^^

`Address: 0x4002e000 + 0x10 = 0x4002e010`


    .. wavedrom::
        :caption: TRNG_SFR_OPTNW

        {
            "reg": [
                {"name": "opt_nw",  "bits": 13},
                {"bits": 19}
            ], "config": {"hspace": 400, "bits": 32, "lanes": 1 }, "options": {"hspace": 400, "bits": 32, "lanes": 1}
        }


+--------+--------+------------------------------------+
| Field  | Name   | Description                        |
+========+========+====================================+
| [12:0] | OPT_NW | opt_nw read/write control register |
+--------+--------+------------------------------------+

TRNG_SFR_OPTEW
^^^^^^^^^^^^^^

`Address: 0x4002e000 + 0x14 = 0x4002e014`


    .. wavedrom::
        :caption: TRNG_SFR_OPTEW

        {
            "reg": [
                {"name": "opt_ew",  "bits": 13},
                {"bits": 19}
            ], "config": {"hspace": 400, "bits": 32, "lanes": 1 }, "options": {"hspace": 400, "bits": 32, "lanes": 1}
        }


+--------+--------+------------------------------------+
| Field  | Name   | Description                        |
+========+========+====================================+
| [12:0] | OPT_EW | opt_ew read/write control register |
+--------+--------+------------------------------------+

TRNG_SFR_OPTMASK
^^^^^^^^^^^^^^^^

`Address: 0x4002e000 + 0x20 = 0x4002e020`


    .. wavedrom::
        :caption: TRNG_SFR_OPTMASK

        {
            "reg": [
                {"name": "opt_mask",  "bits": 16},
                {"bits": 16}
            ], "config": {"hspace": 400, "bits": 32, "lanes": 1 }, "options": {"hspace": 400, "bits": 32, "lanes": 1}
        }


+--------+----------+--------------------------------------+
| Field  | Name     | Description                          |
+========+==========+======================================+
| [15:0] | OPT_MASK | opt_mask read/write control register |
+--------+----------+--------------------------------------+

TRNG_SFR_SEGPTR_CR_SEGPTRSTART0
^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

`Address: 0x4002e000 + 0x30 = 0x4002e030`


    .. wavedrom::
        :caption: TRNG_SFR_SEGPTR_CR_SEGPTRSTART0

        {
            "reg": [
                {"name": "cr_segptrstart0",  "bits": 12},
                {"bits": 20}
            ], "config": {"hspace": 400, "bits": 32, "lanes": 1 }, "options": {"hspace": 400, "bits": 32, "lanes": 1}
        }


+--------+-----------------+--------------------------------------------+
| Field  | Name            | Description                                |
+========+=================+============================================+
| [11:0] | CR_SEGPTRSTART0 | cr_segptrstart read/write control register |
+--------+-----------------+--------------------------------------------+

TRNG_SFR_SEGPTR_CR_SEGPTRSTART1
^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

`Address: 0x4002e000 + 0x34 = 0x4002e034`


    .. wavedrom::
        :caption: TRNG_SFR_SEGPTR_CR_SEGPTRSTART1

        {
            "reg": [
                {"name": "cr_segptrstart1",  "bits": 12},
                {"bits": 20}
            ], "config": {"hspace": 400, "bits": 32, "lanes": 1 }, "options": {"hspace": 400, "bits": 32, "lanes": 1}
        }


+--------+-----------------+--------------------------------------------+
| Field  | Name            | Description                                |
+========+=================+============================================+
| [11:0] | CR_SEGPTRSTART1 | cr_segptrstart read/write control register |
+--------+-----------------+--------------------------------------------+

TRNG_SFR_SEGPTR_CR_SEGPTRSTART2
^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

`Address: 0x4002e000 + 0x38 = 0x4002e038`


    .. wavedrom::
        :caption: TRNG_SFR_SEGPTR_CR_SEGPTRSTART2

        {
            "reg": [
                {"name": "cr_segptrstart2",  "bits": 12},
                {"bits": 20}
            ], "config": {"hspace": 400, "bits": 32, "lanes": 1 }, "options": {"hspace": 400, "bits": 32, "lanes": 1}
        }


+--------+-----------------+--------------------------------------------+
| Field  | Name            | Description                                |
+========+=================+============================================+
| [11:0] | CR_SEGPTRSTART2 | cr_segptrstart read/write control register |
+--------+-----------------+--------------------------------------------+

TRNG_SFR_SEGPTR_CR_SEGPTRSTART3
^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

`Address: 0x4002e000 + 0x3c = 0x4002e03c`


    .. wavedrom::
        :caption: TRNG_SFR_SEGPTR_CR_SEGPTRSTART3

        {
            "reg": [
                {"name": "cr_segptrstart3",  "bits": 12},
                {"bits": 20}
            ], "config": {"hspace": 400, "bits": 32, "lanes": 1 }, "options": {"hspace": 400, "bits": 32, "lanes": 1}
        }


+--------+-----------------+--------------------------------------------+
| Field  | Name            | Description                                |
+========+=================+============================================+
| [11:0] | CR_SEGPTRSTART3 | cr_segptrstart read/write control register |
+--------+-----------------+--------------------------------------------+

TRNG_SFR_SEGPTR_CR_SEGPTRSTART4
^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

`Address: 0x4002e000 + 0x40 = 0x4002e040`


    .. wavedrom::
        :caption: TRNG_SFR_SEGPTR_CR_SEGPTRSTART4

        {
            "reg": [
                {"name": "cr_segptrstart4",  "bits": 12},
                {"bits": 20}
            ], "config": {"hspace": 400, "bits": 32, "lanes": 1 }, "options": {"hspace": 400, "bits": 32, "lanes": 1}
        }


+--------+-----------------+--------------------------------------------+
| Field  | Name            | Description                                |
+========+=================+============================================+
| [11:0] | CR_SEGPTRSTART4 | cr_segptrstart read/write control register |
+--------+-----------------+--------------------------------------------+

