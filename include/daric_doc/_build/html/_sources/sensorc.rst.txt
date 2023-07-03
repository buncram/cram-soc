SENSORC
=======

Register Listing for SENSORC
----------------------------

+------------------------------------------------------------------+-------------------------------------------------+
| Register                                                         | Address                                         |
+==================================================================+=================================================+
| :ref:`SENSORC_SFR_VDMASK0 <SENSORC_SFR_VDMASK0>`                 | :ref:`0x40053000 <SENSORC_SFR_VDMASK0>`         |
+------------------------------------------------------------------+-------------------------------------------------+
| :ref:`SENSORC_SFR_VDMASK1 <SENSORC_SFR_VDMASK1>`                 | :ref:`0x40053004 <SENSORC_SFR_VDMASK1>`         |
+------------------------------------------------------------------+-------------------------------------------------+
| :ref:`SENSORC_SFR_VDSR <SENSORC_SFR_VDSR>`                       | :ref:`0x40053008 <SENSORC_SFR_VDSR>`            |
+------------------------------------------------------------------+-------------------------------------------------+
| :ref:`SENSORC_SFR_LDMASK <SENSORC_SFR_LDMASK>`                   | :ref:`0x40053010 <SENSORC_SFR_LDMASK>`          |
+------------------------------------------------------------------+-------------------------------------------------+
| :ref:`SENSORC_SFR_LDSR <SENSORC_SFR_LDSR>`                       | :ref:`0x40053014 <SENSORC_SFR_LDSR>`            |
+------------------------------------------------------------------+-------------------------------------------------+
| :ref:`SENSORC_SFR_LDCFG <SENSORC_SFR_LDCFG>`                     | :ref:`0x40053018 <SENSORC_SFR_LDCFG>`           |
+------------------------------------------------------------------+-------------------------------------------------+
| :ref:`SENSORC_SFR_VDCFG_CR_VDCFG0 <SENSORC_SFR_VDCFG_CR_VDCFG0>` | :ref:`0x40053020 <SENSORC_SFR_VDCFG_CR_VDCFG0>` |
+------------------------------------------------------------------+-------------------------------------------------+
| :ref:`SENSORC_SFR_VDCFG_CR_VDCFG1 <SENSORC_SFR_VDCFG_CR_VDCFG1>` | :ref:`0x40053024 <SENSORC_SFR_VDCFG_CR_VDCFG1>` |
+------------------------------------------------------------------+-------------------------------------------------+
| :ref:`SENSORC_SFR_VDCFG_CR_VDCFG2 <SENSORC_SFR_VDCFG_CR_VDCFG2>` | :ref:`0x40053028 <SENSORC_SFR_VDCFG_CR_VDCFG2>` |
+------------------------------------------------------------------+-------------------------------------------------+
| :ref:`SENSORC_SFR_VDCFG_CR_VDCFG3 <SENSORC_SFR_VDCFG_CR_VDCFG3>` | :ref:`0x4005302c <SENSORC_SFR_VDCFG_CR_VDCFG3>` |
+------------------------------------------------------------------+-------------------------------------------------+
| :ref:`SENSORC_SFR_VDCFG_CR_VDCFG4 <SENSORC_SFR_VDCFG_CR_VDCFG4>` | :ref:`0x40053030 <SENSORC_SFR_VDCFG_CR_VDCFG4>` |
+------------------------------------------------------------------+-------------------------------------------------+
| :ref:`SENSORC_SFR_VDCFG_CR_VDCFG5 <SENSORC_SFR_VDCFG_CR_VDCFG5>` | :ref:`0x40053034 <SENSORC_SFR_VDCFG_CR_VDCFG5>` |
+------------------------------------------------------------------+-------------------------------------------------+
| :ref:`SENSORC_SFR_VDCFG_CR_VDCFG6 <SENSORC_SFR_VDCFG_CR_VDCFG6>` | :ref:`0x40053038 <SENSORC_SFR_VDCFG_CR_VDCFG6>` |
+------------------------------------------------------------------+-------------------------------------------------+
| :ref:`SENSORC_SFR_VDCFG_CR_VDCFG7 <SENSORC_SFR_VDCFG_CR_VDCFG7>` | :ref:`0x4005303c <SENSORC_SFR_VDCFG_CR_VDCFG7>` |
+------------------------------------------------------------------+-------------------------------------------------+

SENSORC_SFR_VDMASK0
^^^^^^^^^^^^^^^^^^^

`Address: 0x40053000 + 0x0 = 0x40053000`


    .. wavedrom::
        :caption: SENSORC_SFR_VDMASK0

        {
            "reg": [
                {"name": "cr_vdmask0",  "bits": 8},
                {"bits": 24}
            ], "config": {"hspace": 400, "bits": 32, "lanes": 1 }, "options": {"hspace": 400, "bits": 32, "lanes": 1}
        }


+-------+------------+----------------------------------------+
| Field | Name       | Description                            |
+=======+============+========================================+
| [7:0] | CR_VDMASK0 | cr_vdmask0 read/write control register |
+-------+------------+----------------------------------------+

SENSORC_SFR_VDMASK1
^^^^^^^^^^^^^^^^^^^

`Address: 0x40053000 + 0x4 = 0x40053004`


    .. wavedrom::
        :caption: SENSORC_SFR_VDMASK1

        {
            "reg": [
                {"name": "cr_vdmask1",  "bits": 8},
                {"bits": 24}
            ], "config": {"hspace": 400, "bits": 32, "lanes": 1 }, "options": {"hspace": 400, "bits": 32, "lanes": 1}
        }


+-------+------------+----------------------------------------+
| Field | Name       | Description                            |
+=======+============+========================================+
| [7:0] | CR_VDMASK1 | cr_vdmask1 read/write control register |
+-------+------------+----------------------------------------+

SENSORC_SFR_VDSR
^^^^^^^^^^^^^^^^

`Address: 0x40053000 + 0x8 = 0x40053008`


    .. wavedrom::
        :caption: SENSORC_SFR_VDSR

        {
            "reg": [
                {"name": "sr_vdsr",  "bits": 8},
                {"bits": 24}
            ], "config": {"hspace": 400, "bits": 32, "lanes": 1 }, "options": {"hspace": 400, "bits": 32, "lanes": 1}
        }


+-------+---------+-----------------------------------+
| Field | Name    | Description                       |
+=======+=========+===================================+
| [7:0] | SR_VDSR | sr_vdsr read only status register |
+-------+---------+-----------------------------------+

SENSORC_SFR_LDMASK
^^^^^^^^^^^^^^^^^^

`Address: 0x40053000 + 0x10 = 0x40053010`


    .. wavedrom::
        :caption: SENSORC_SFR_LDMASK

        {
            "reg": [
                {"name": "cr_ldmask",  "bits": 4},
                {"bits": 28}
            ], "config": {"hspace": 400, "bits": 32, "lanes": 4 }, "options": {"hspace": 400, "bits": 32, "lanes": 4}
        }


+-------+-----------+---------------------------------------+
| Field | Name      | Description                           |
+=======+===========+=======================================+
| [3:0] | CR_LDMASK | cr_ldmask read/write control register |
+-------+-----------+---------------------------------------+

SENSORC_SFR_LDSR
^^^^^^^^^^^^^^^^

`Address: 0x40053000 + 0x14 = 0x40053014`


    .. wavedrom::
        :caption: SENSORC_SFR_LDSR

        {
            "reg": [
                {"name": "sr_ldsr",  "bits": 4},
                {"bits": 28}
            ], "config": {"hspace": 400, "bits": 32, "lanes": 4 }, "options": {"hspace": 400, "bits": 32, "lanes": 4}
        }


+-------+---------+-----------------------------------+
| Field | Name    | Description                       |
+=======+=========+===================================+
| [3:0] | SR_LDSR | sr_ldsr read only status register |
+-------+---------+-----------------------------------+

SENSORC_SFR_LDCFG
^^^^^^^^^^^^^^^^^

`Address: 0x40053000 + 0x18 = 0x40053018`


    .. wavedrom::
        :caption: SENSORC_SFR_LDCFG

        {
            "reg": [
                {"name": "sfr_ldcfg",  "bits": 4},
                {"bits": 28}
            ], "config": {"hspace": 400, "bits": 32, "lanes": 4 }, "options": {"hspace": 400, "bits": 32, "lanes": 4}
        }


+-------+-----------+---------------------------------------+
| Field | Name      | Description                           |
+=======+===========+=======================================+
| [3:0] | SFR_LDCFG | sfr_ldcfg read/write control register |
+-------+-----------+---------------------------------------+

SENSORC_SFR_VDCFG_CR_VDCFG0
^^^^^^^^^^^^^^^^^^^^^^^^^^^

`Address: 0x40053000 + 0x20 = 0x40053020`


    .. wavedrom::
        :caption: SENSORC_SFR_VDCFG_CR_VDCFG0

        {
            "reg": [
                {"name": "cr_vdcfg0",  "bits": 4},
                {"bits": 28}
            ], "config": {"hspace": 400, "bits": 32, "lanes": 4 }, "options": {"hspace": 400, "bits": 32, "lanes": 4}
        }


+-------+-----------+--------------------------------------+
| Field | Name      | Description                          |
+=======+===========+======================================+
| [3:0] | CR_VDCFG0 | cr_vdcfg read/write control register |
+-------+-----------+--------------------------------------+

SENSORC_SFR_VDCFG_CR_VDCFG1
^^^^^^^^^^^^^^^^^^^^^^^^^^^

`Address: 0x40053000 + 0x24 = 0x40053024`


    .. wavedrom::
        :caption: SENSORC_SFR_VDCFG_CR_VDCFG1

        {
            "reg": [
                {"name": "cr_vdcfg1",  "bits": 4},
                {"bits": 28}
            ], "config": {"hspace": 400, "bits": 32, "lanes": 4 }, "options": {"hspace": 400, "bits": 32, "lanes": 4}
        }


+-------+-----------+--------------------------------------+
| Field | Name      | Description                          |
+=======+===========+======================================+
| [3:0] | CR_VDCFG1 | cr_vdcfg read/write control register |
+-------+-----------+--------------------------------------+

SENSORC_SFR_VDCFG_CR_VDCFG2
^^^^^^^^^^^^^^^^^^^^^^^^^^^

`Address: 0x40053000 + 0x28 = 0x40053028`


    .. wavedrom::
        :caption: SENSORC_SFR_VDCFG_CR_VDCFG2

        {
            "reg": [
                {"name": "cr_vdcfg2",  "bits": 4},
                {"bits": 28}
            ], "config": {"hspace": 400, "bits": 32, "lanes": 4 }, "options": {"hspace": 400, "bits": 32, "lanes": 4}
        }


+-------+-----------+--------------------------------------+
| Field | Name      | Description                          |
+=======+===========+======================================+
| [3:0] | CR_VDCFG2 | cr_vdcfg read/write control register |
+-------+-----------+--------------------------------------+

SENSORC_SFR_VDCFG_CR_VDCFG3
^^^^^^^^^^^^^^^^^^^^^^^^^^^

`Address: 0x40053000 + 0x2c = 0x4005302c`


    .. wavedrom::
        :caption: SENSORC_SFR_VDCFG_CR_VDCFG3

        {
            "reg": [
                {"name": "cr_vdcfg3",  "bits": 4},
                {"bits": 28}
            ], "config": {"hspace": 400, "bits": 32, "lanes": 4 }, "options": {"hspace": 400, "bits": 32, "lanes": 4}
        }


+-------+-----------+--------------------------------------+
| Field | Name      | Description                          |
+=======+===========+======================================+
| [3:0] | CR_VDCFG3 | cr_vdcfg read/write control register |
+-------+-----------+--------------------------------------+

SENSORC_SFR_VDCFG_CR_VDCFG4
^^^^^^^^^^^^^^^^^^^^^^^^^^^

`Address: 0x40053000 + 0x30 = 0x40053030`


    .. wavedrom::
        :caption: SENSORC_SFR_VDCFG_CR_VDCFG4

        {
            "reg": [
                {"name": "cr_vdcfg4",  "bits": 4},
                {"bits": 28}
            ], "config": {"hspace": 400, "bits": 32, "lanes": 4 }, "options": {"hspace": 400, "bits": 32, "lanes": 4}
        }


+-------+-----------+--------------------------------------+
| Field | Name      | Description                          |
+=======+===========+======================================+
| [3:0] | CR_VDCFG4 | cr_vdcfg read/write control register |
+-------+-----------+--------------------------------------+

SENSORC_SFR_VDCFG_CR_VDCFG5
^^^^^^^^^^^^^^^^^^^^^^^^^^^

`Address: 0x40053000 + 0x34 = 0x40053034`


    .. wavedrom::
        :caption: SENSORC_SFR_VDCFG_CR_VDCFG5

        {
            "reg": [
                {"name": "cr_vdcfg5",  "bits": 4},
                {"bits": 28}
            ], "config": {"hspace": 400, "bits": 32, "lanes": 4 }, "options": {"hspace": 400, "bits": 32, "lanes": 4}
        }


+-------+-----------+--------------------------------------+
| Field | Name      | Description                          |
+=======+===========+======================================+
| [3:0] | CR_VDCFG5 | cr_vdcfg read/write control register |
+-------+-----------+--------------------------------------+

SENSORC_SFR_VDCFG_CR_VDCFG6
^^^^^^^^^^^^^^^^^^^^^^^^^^^

`Address: 0x40053000 + 0x38 = 0x40053038`


    .. wavedrom::
        :caption: SENSORC_SFR_VDCFG_CR_VDCFG6

        {
            "reg": [
                {"name": "cr_vdcfg6",  "bits": 4},
                {"bits": 28}
            ], "config": {"hspace": 400, "bits": 32, "lanes": 4 }, "options": {"hspace": 400, "bits": 32, "lanes": 4}
        }


+-------+-----------+--------------------------------------+
| Field | Name      | Description                          |
+=======+===========+======================================+
| [3:0] | CR_VDCFG6 | cr_vdcfg read/write control register |
+-------+-----------+--------------------------------------+

SENSORC_SFR_VDCFG_CR_VDCFG7
^^^^^^^^^^^^^^^^^^^^^^^^^^^

`Address: 0x40053000 + 0x3c = 0x4005303c`


    .. wavedrom::
        :caption: SENSORC_SFR_VDCFG_CR_VDCFG7

        {
            "reg": [
                {"name": "cr_vdcfg7",  "bits": 4},
                {"bits": 28}
            ], "config": {"hspace": 400, "bits": 32, "lanes": 4 }, "options": {"hspace": 400, "bits": 32, "lanes": 4}
        }


+-------+-----------+--------------------------------------+
| Field | Name      | Description                          |
+=======+===========+======================================+
| [3:0] | CR_VDCFG7 | cr_vdcfg read/write control register |
+-------+-----------+--------------------------------------+

