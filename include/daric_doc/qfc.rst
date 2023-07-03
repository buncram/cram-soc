QFC
===

Register Listing for QFC
------------------------

+--------------------------------------------------+-----------------------------------------+
| Register                                         | Address                                 |
+==================================================+=========================================+
| :ref:`QFC_SFR_IO <QFC_SFR_IO>`                   | :ref:`0x40000000 <QFC_SFR_IO>`          |
+--------------------------------------------------+-----------------------------------------+
| :ref:`QFC_SFR_AR <QFC_SFR_AR>`                   | :ref:`0x40000004 <QFC_SFR_AR>`          |
+--------------------------------------------------+-----------------------------------------+
| :ref:`QFC_SFR_IODRV <QFC_SFR_IODRV>`             | :ref:`0x40000008 <QFC_SFR_IODRV>`       |
+--------------------------------------------------+-----------------------------------------+
| :ref:`QFC_CR_XIP_ADDRMODE <QFC_CR_XIP_ADDRMODE>` | :ref:`0x40000010 <QFC_CR_XIP_ADDRMODE>` |
+--------------------------------------------------+-----------------------------------------+
| :ref:`QFC_CR_XIP_OPCODE <QFC_CR_XIP_OPCODE>`     | :ref:`0x40000014 <QFC_CR_XIP_OPCODE>`   |
+--------------------------------------------------+-----------------------------------------+
| :ref:`QFC_CR_XIP_WIDTH <QFC_CR_XIP_WIDTH>`       | :ref:`0x40000018 <QFC_CR_XIP_WIDTH>`    |
+--------------------------------------------------+-----------------------------------------+
| :ref:`QFC_CR_XIP_SSEL <QFC_CR_XIP_SSEL>`         | :ref:`0x4000001c <QFC_CR_XIP_SSEL>`     |
+--------------------------------------------------+-----------------------------------------+
| :ref:`QFC_CR_XIP_DUMCYC <QFC_CR_XIP_DUMCYC>`     | :ref:`0x40000020 <QFC_CR_XIP_DUMCYC>`   |
+--------------------------------------------------+-----------------------------------------+
| :ref:`QFC_CR_XIP_CFG <QFC_CR_XIP_CFG>`           | :ref:`0x40000024 <QFC_CR_XIP_CFG>`      |
+--------------------------------------------------+-----------------------------------------+

QFC_SFR_IO
^^^^^^^^^^

`Address: 0x40000000 + 0x0 = 0x40000000`


    .. wavedrom::
        :caption: QFC_SFR_IO

        {
            "reg": [
                {"name": "sfr_io",  "bits": 8},
                {"bits": 24}
            ], "config": {"hspace": 400, "bits": 32, "lanes": 1 }, "options": {"hspace": 400, "bits": 32, "lanes": 1}
        }


+-------+--------+------------------------------------+
| Field | Name   | Description                        |
+=======+========+====================================+
| [7:0] | SFR_IO | sfr_io read/write control register |
+-------+--------+------------------------------------+

QFC_SFR_AR
^^^^^^^^^^

`Address: 0x40000000 + 0x4 = 0x40000004`


    .. wavedrom::
        :caption: QFC_SFR_AR

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

QFC_SFR_IODRV
^^^^^^^^^^^^^

`Address: 0x40000000 + 0x8 = 0x40000008`


    .. wavedrom::
        :caption: QFC_SFR_IODRV

        {
            "reg": [
                {"name": "paddrvsel",  "bits": 12},
                {"bits": 20}
            ], "config": {"hspace": 400, "bits": 32, "lanes": 1 }, "options": {"hspace": 400, "bits": 32, "lanes": 1}
        }


+--------+-----------+---------------------------------------+
| Field  | Name      | Description                           |
+========+===========+=======================================+
| [11:0] | PADDRVSEL | paddrvsel read/write control register |
+--------+-----------+---------------------------------------+

QFC_CR_XIP_ADDRMODE
^^^^^^^^^^^^^^^^^^^

`Address: 0x40000000 + 0x10 = 0x40000010`


    .. wavedrom::
        :caption: QFC_CR_XIP_ADDRMODE

        {
            "reg": [
                {"name": "cr_xip_addrmode",  "bits": 2},
                {"bits": 30}
            ], "config": {"hspace": 400, "bits": 32, "lanes": 4 }, "options": {"hspace": 400, "bits": 32, "lanes": 4}
        }


+-------+-----------------+---------------------------------------------+
| Field | Name            | Description                                 |
+=======+=================+=============================================+
| [1:0] | CR_XIP_ADDRMODE | cr_xip_addrmode read/write control register |
+-------+-----------------+---------------------------------------------+

QFC_CR_XIP_OPCODE
^^^^^^^^^^^^^^^^^

`Address: 0x40000000 + 0x14 = 0x40000014`


    .. wavedrom::
        :caption: QFC_CR_XIP_OPCODE

        {
            "reg": [
                {"name": "cr_xip_opcode",  "bits": 32}
            ], "config": {"hspace": 400, "bits": 32, "lanes": 1 }, "options": {"hspace": 400, "bits": 32, "lanes": 1}
        }


+--------+---------------+-------------------------------------------+
| Field  | Name          | Description                               |
+========+===============+===========================================+
| [31:0] | CR_XIP_OPCODE | cr_xip_opcode read/write control register |
+--------+---------------+-------------------------------------------+

QFC_CR_XIP_WIDTH
^^^^^^^^^^^^^^^^

`Address: 0x40000000 + 0x18 = 0x40000018`


    .. wavedrom::
        :caption: QFC_CR_XIP_WIDTH

        {
            "reg": [
                {"name": "cr_xip_width",  "bits": 6},
                {"bits": 26}
            ], "config": {"hspace": 400, "bits": 32, "lanes": 4 }, "options": {"hspace": 400, "bits": 32, "lanes": 4}
        }


+-------+--------------+------------------------------------------+
| Field | Name         | Description                              |
+=======+==============+==========================================+
| [5:0] | CR_XIP_WIDTH | cr_xip_width read/write control register |
+-------+--------------+------------------------------------------+

QFC_CR_XIP_SSEL
^^^^^^^^^^^^^^^

`Address: 0x40000000 + 0x1c = 0x4000001c`


    .. wavedrom::
        :caption: QFC_CR_XIP_SSEL

        {
            "reg": [
                {"name": "cr_xip_ssel",  "bits": 7},
                {"bits": 25}
            ], "config": {"hspace": 400, "bits": 32, "lanes": 4 }, "options": {"hspace": 400, "bits": 32, "lanes": 4}
        }


+-------+-------------+-----------------------------------------+
| Field | Name        | Description                             |
+=======+=============+=========================================+
| [6:0] | CR_XIP_SSEL | cr_xip_ssel read/write control register |
+-------+-------------+-----------------------------------------+

QFC_CR_XIP_DUMCYC
^^^^^^^^^^^^^^^^^

`Address: 0x40000000 + 0x20 = 0x40000020`


    .. wavedrom::
        :caption: QFC_CR_XIP_DUMCYC

        {
            "reg": [
                {"name": "cr_xip_dumcyc",  "bits": 16},
                {"bits": 16}
            ], "config": {"hspace": 400, "bits": 32, "lanes": 1 }, "options": {"hspace": 400, "bits": 32, "lanes": 1}
        }


+--------+---------------+-------------------------------------------+
| Field  | Name          | Description                               |
+========+===============+===========================================+
| [15:0] | CR_XIP_DUMCYC | cr_xip_dumcyc read/write control register |
+--------+---------------+-------------------------------------------+

QFC_CR_XIP_CFG
^^^^^^^^^^^^^^

`Address: 0x40000000 + 0x24 = 0x40000024`


    .. wavedrom::
        :caption: QFC_CR_XIP_CFG

        {
            "reg": [
                {"name": "cr_xip_cfg",  "bits": 14},
                {"bits": 18}
            ], "config": {"hspace": 400, "bits": 32, "lanes": 1 }, "options": {"hspace": 400, "bits": 32, "lanes": 1}
        }


+--------+------------+----------------------------------------+
| Field  | Name       | Description                            |
+========+============+========================================+
| [13:0] | CR_XIP_CFG | cr_xip_cfg read/write control register |
+--------+------------+----------------------------------------+

