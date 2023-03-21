SCEDMA
======

Register Listing for SCEDMA
---------------------------

+----------------------------------------------------------+---------------------------------------------+
| Register                                                 | Address                                     |
+==========================================================+=============================================+
| :ref:`SCEDMA_SFR_SCHSTART_AR <SCEDMA_SFR_SCHSTART_AR>`   | :ref:`0x40029000 <SCEDMA_SFR_SCHSTART_AR>`  |
+----------------------------------------------------------+---------------------------------------------+
| :ref:`SCEDMA_RESERVED1 <SCEDMA_RESERVED1>`               | :ref:`0x40029004 <SCEDMA_RESERVED1>`        |
+----------------------------------------------------------+---------------------------------------------+
| :ref:`SCEDMA_RESERVED2 <SCEDMA_RESERVED2>`               | :ref:`0x40029008 <SCEDMA_RESERVED2>`        |
+----------------------------------------------------------+---------------------------------------------+
| :ref:`SCEDMA_RESERVED3 <SCEDMA_RESERVED3>`               | :ref:`0x4002900c <SCEDMA_RESERVED3>`        |
+----------------------------------------------------------+---------------------------------------------+
| :ref:`SCEDMA_SFR_XCH_FUNC <SCEDMA_SFR_XCH_FUNC>`         | :ref:`0x40029010 <SCEDMA_SFR_XCH_FUNC>`     |
+----------------------------------------------------------+---------------------------------------------+
| :ref:`SCEDMA_SFR_XCH_OPT <SCEDMA_SFR_XCH_OPT>`           | :ref:`0x40029014 <SCEDMA_SFR_XCH_OPT>`      |
+----------------------------------------------------------+---------------------------------------------+
| :ref:`SCEDMA_SFR_XCH_AXSTART <SCEDMA_SFR_XCH_AXSTART>`   | :ref:`0x40029018 <SCEDMA_SFR_XCH_AXSTART>`  |
+----------------------------------------------------------+---------------------------------------------+
| :ref:`SCEDMA_SFR_XCH_SEGID <SCEDMA_SFR_XCH_SEGID>`       | :ref:`0x4002901c <SCEDMA_SFR_XCH_SEGID>`    |
+----------------------------------------------------------+---------------------------------------------+
| :ref:`SCEDMA_SFR_XCH_SEGSTART <SCEDMA_SFR_XCH_SEGSTART>` | :ref:`0x40029020 <SCEDMA_SFR_XCH_SEGSTART>` |
+----------------------------------------------------------+---------------------------------------------+
| :ref:`SCEDMA_SFR_XCH_TRANSIZE <SCEDMA_SFR_XCH_TRANSIZE>` | :ref:`0x40029024 <SCEDMA_SFR_XCH_TRANSIZE>` |
+----------------------------------------------------------+---------------------------------------------+
| :ref:`SCEDMA_RESERVED10 <SCEDMA_RESERVED10>`             | :ref:`0x40029028 <SCEDMA_RESERVED10>`       |
+----------------------------------------------------------+---------------------------------------------+
| :ref:`SCEDMA_RESERVED11 <SCEDMA_RESERVED11>`             | :ref:`0x4002902c <SCEDMA_RESERVED11>`       |
+----------------------------------------------------------+---------------------------------------------+
| :ref:`SCEDMA_SFR_SCH_FUNC <SCEDMA_SFR_SCH_FUNC>`         | :ref:`0x40029030 <SCEDMA_SFR_SCH_FUNC>`     |
+----------------------------------------------------------+---------------------------------------------+
| :ref:`SCEDMA_SFR_SCH_OPT <SCEDMA_SFR_SCH_OPT>`           | :ref:`0x40029034 <SCEDMA_SFR_SCH_OPT>`      |
+----------------------------------------------------------+---------------------------------------------+
| :ref:`SCEDMA_SFR_SCH_AXSTART <SCEDMA_SFR_SCH_AXSTART>`   | :ref:`0x40029038 <SCEDMA_SFR_SCH_AXSTART>`  |
+----------------------------------------------------------+---------------------------------------------+
| :ref:`SCEDMA_SFR_SCH_SEGID <SCEDMA_SFR_SCH_SEGID>`       | :ref:`0x4002903c <SCEDMA_SFR_SCH_SEGID>`    |
+----------------------------------------------------------+---------------------------------------------+
| :ref:`SCEDMA_SFR_SCH_SEGSTART <SCEDMA_SFR_SCH_SEGSTART>` | :ref:`0x40029040 <SCEDMA_SFR_SCH_SEGSTART>` |
+----------------------------------------------------------+---------------------------------------------+
| :ref:`SCEDMA_SFR_SCH_TRANSIZE <SCEDMA_SFR_SCH_TRANSIZE>` | :ref:`0x40029044 <SCEDMA_SFR_SCH_TRANSIZE>` |
+----------------------------------------------------------+---------------------------------------------+
| :ref:`SCEDMA_RESERVED18 <SCEDMA_RESERVED18>`             | :ref:`0x40029048 <SCEDMA_RESERVED18>`       |
+----------------------------------------------------------+---------------------------------------------+
| :ref:`SCEDMA_RESERVED19 <SCEDMA_RESERVED19>`             | :ref:`0x4002904c <SCEDMA_RESERVED19>`       |
+----------------------------------------------------------+---------------------------------------------+
| :ref:`SCEDMA_SFR_ICH_OPT <SCEDMA_SFR_ICH_OPT>`           | :ref:`0x40029050 <SCEDMA_SFR_ICH_OPT>`      |
+----------------------------------------------------------+---------------------------------------------+
| :ref:`SCEDMA_SFR_ICH_SEGID <SCEDMA_SFR_ICH_SEGID>`       | :ref:`0x40029054 <SCEDMA_SFR_ICH_SEGID>`    |
+----------------------------------------------------------+---------------------------------------------+
| :ref:`SCEDMA_SFR_ICH_RPSTART <SCEDMA_SFR_ICH_RPSTART>`   | :ref:`0x40029058 <SCEDMA_SFR_ICH_RPSTART>`  |
+----------------------------------------------------------+---------------------------------------------+
| :ref:`SCEDMA_SFR_ICH_WPSTART <SCEDMA_SFR_ICH_WPSTART>`   | :ref:`0x4002905c <SCEDMA_SFR_ICH_WPSTART>`  |
+----------------------------------------------------------+---------------------------------------------+
| :ref:`SCEDMA_SFR_ICH_TRANSIZE <SCEDMA_SFR_ICH_TRANSIZE>` | :ref:`0x40029060 <SCEDMA_SFR_ICH_TRANSIZE>` |
+----------------------------------------------------------+---------------------------------------------+

SCEDMA_SFR_SCHSTART_AR
^^^^^^^^^^^^^^^^^^^^^^

`Address: 0x40029000 + 0x0 = 0x40029000`


    .. wavedrom::
        :caption: SCEDMA_SFR_SCHSTART_AR

        {
            "reg": [
                {"name": "sfr_schstart_ar",  "type": 4, "bits": 32}
            ], "config": {"hspace": 400, "bits": 32, "lanes": 1 }, "options": {"hspace": 400, "bits": 32, "lanes": 1}
        }


+--------+-----------------+---------------------------------------------------------+
| Field  | Name            | Description                                             |
+========+=================+=========================================================+
| [31:0] | SFR_SCHSTART_AR | sfr_schstart_ar performs action on write of value: 0xaa |
+--------+-----------------+---------------------------------------------------------+

SCEDMA_RESERVED1
^^^^^^^^^^^^^^^^

`Address: 0x40029000 + 0x4 = 0x40029004`


    .. wavedrom::
        :caption: SCEDMA_RESERVED1

        {
            "reg": [
                {"name": "reserved1", "bits": 1},
                {"bits": 31},
            ], "config": {"hspace": 400, "bits": 32, "lanes": 4 }, "options": {"hspace": 400, "bits": 32, "lanes": 4}
        }


SCEDMA_RESERVED2
^^^^^^^^^^^^^^^^

`Address: 0x40029000 + 0x8 = 0x40029008`


    .. wavedrom::
        :caption: SCEDMA_RESERVED2

        {
            "reg": [
                {"name": "reserved2", "bits": 1},
                {"bits": 31},
            ], "config": {"hspace": 400, "bits": 32, "lanes": 4 }, "options": {"hspace": 400, "bits": 32, "lanes": 4}
        }


SCEDMA_RESERVED3
^^^^^^^^^^^^^^^^

`Address: 0x40029000 + 0xc = 0x4002900c`


    .. wavedrom::
        :caption: SCEDMA_RESERVED3

        {
            "reg": [
                {"name": "reserved3", "bits": 1},
                {"bits": 31},
            ], "config": {"hspace": 400, "bits": 32, "lanes": 4 }, "options": {"hspace": 400, "bits": 32, "lanes": 4}
        }


SCEDMA_SFR_XCH_FUNC
^^^^^^^^^^^^^^^^^^^

`Address: 0x40029000 + 0x10 = 0x40029010`


    .. wavedrom::
        :caption: SCEDMA_SFR_XCH_FUNC

        {
            "reg": [
                {"name": "sfr_xch_func",  "bits": 1},
                {"bits": 31}
            ], "config": {"hspace": 400, "bits": 32, "lanes": 4 }, "options": {"hspace": 400, "bits": 32, "lanes": 4}
        }


+-------+--------------+------------------------------------------+
| Field | Name         | Description                              |
+=======+==============+==========================================+
| [0]   | SFR_XCH_FUNC | sfr_xch_func read/write control register |
+-------+--------------+------------------------------------------+

SCEDMA_SFR_XCH_OPT
^^^^^^^^^^^^^^^^^^

`Address: 0x40029000 + 0x14 = 0x40029014`


    .. wavedrom::
        :caption: SCEDMA_SFR_XCH_OPT

        {
            "reg": [
                {"name": "sfr_xch_opt",  "bits": 8},
                {"bits": 24}
            ], "config": {"hspace": 400, "bits": 32, "lanes": 1 }, "options": {"hspace": 400, "bits": 32, "lanes": 1}
        }


+-------+-------------+-----------------------------------------+
| Field | Name        | Description                             |
+=======+=============+=========================================+
| [7:0] | SFR_XCH_OPT | sfr_xch_opt read/write control register |
+-------+-------------+-----------------------------------------+

SCEDMA_SFR_XCH_AXSTART
^^^^^^^^^^^^^^^^^^^^^^

`Address: 0x40029000 + 0x18 = 0x40029018`


    .. wavedrom::
        :caption: SCEDMA_SFR_XCH_AXSTART

        {
            "reg": [
                {"name": "sfr_xch_axstart",  "bits": 32}
            ], "config": {"hspace": 400, "bits": 32, "lanes": 1 }, "options": {"hspace": 400, "bits": 32, "lanes": 1}
        }


+--------+-----------------+---------------------------------------------+
| Field  | Name            | Description                                 |
+========+=================+=============================================+
| [31:0] | SFR_XCH_AXSTART | sfr_xch_axstart read/write control register |
+--------+-----------------+---------------------------------------------+

SCEDMA_SFR_XCH_SEGID
^^^^^^^^^^^^^^^^^^^^

`Address: 0x40029000 + 0x1c = 0x4002901c`


    .. wavedrom::
        :caption: SCEDMA_SFR_XCH_SEGID

        {
            "reg": [
                {"name": "sfr_xch_segid",  "bits": 8},
                {"bits": 24}
            ], "config": {"hspace": 400, "bits": 32, "lanes": 1 }, "options": {"hspace": 400, "bits": 32, "lanes": 1}
        }


+-------+---------------+-------------------------------------------+
| Field | Name          | Description                               |
+=======+===============+===========================================+
| [7:0] | SFR_XCH_SEGID | sfr_xch_segid read/write control register |
+-------+---------------+-------------------------------------------+

SCEDMA_SFR_XCH_SEGSTART
^^^^^^^^^^^^^^^^^^^^^^^

`Address: 0x40029000 + 0x20 = 0x40029020`


    .. wavedrom::
        :caption: SCEDMA_SFR_XCH_SEGSTART

        {
            "reg": [
                {"name": "xchcr_segstart",  "bits": 12},
                {"bits": 20}
            ], "config": {"hspace": 400, "bits": 32, "lanes": 1 }, "options": {"hspace": 400, "bits": 32, "lanes": 1}
        }


+--------+----------------+--------------------------------------------+
| Field  | Name           | Description                                |
+========+================+============================================+
| [11:0] | XCHCR_SEGSTART | xchcr_segstart read/write control register |
+--------+----------------+--------------------------------------------+

SCEDMA_SFR_XCH_TRANSIZE
^^^^^^^^^^^^^^^^^^^^^^^

`Address: 0x40029000 + 0x24 = 0x40029024`


    .. wavedrom::
        :caption: SCEDMA_SFR_XCH_TRANSIZE

        {
            "reg": [
                {"name": "xchcr_transize",  "bits": 30},
                {"bits": 2}
            ], "config": {"hspace": 400, "bits": 32, "lanes": 1 }, "options": {"hspace": 400, "bits": 32, "lanes": 1}
        }


+--------+----------------+--------------------------------------------+
| Field  | Name           | Description                                |
+========+================+============================================+
| [29:0] | XCHCR_TRANSIZE | xchcr_transize read/write control register |
+--------+----------------+--------------------------------------------+

SCEDMA_RESERVED10
^^^^^^^^^^^^^^^^^

`Address: 0x40029000 + 0x28 = 0x40029028`


    .. wavedrom::
        :caption: SCEDMA_RESERVED10

        {
            "reg": [
                {"name": "reserved10", "bits": 1},
                {"bits": 31},
            ], "config": {"hspace": 400, "bits": 32, "lanes": 4 }, "options": {"hspace": 400, "bits": 32, "lanes": 4}
        }


SCEDMA_RESERVED11
^^^^^^^^^^^^^^^^^

`Address: 0x40029000 + 0x2c = 0x4002902c`


    .. wavedrom::
        :caption: SCEDMA_RESERVED11

        {
            "reg": [
                {"name": "reserved11", "bits": 1},
                {"bits": 31},
            ], "config": {"hspace": 400, "bits": 32, "lanes": 4 }, "options": {"hspace": 400, "bits": 32, "lanes": 4}
        }


SCEDMA_SFR_SCH_FUNC
^^^^^^^^^^^^^^^^^^^

`Address: 0x40029000 + 0x30 = 0x40029030`


    .. wavedrom::
        :caption: SCEDMA_SFR_SCH_FUNC

        {
            "reg": [
                {"name": "sfr_sch_func",  "bits": 1},
                {"bits": 31}
            ], "config": {"hspace": 400, "bits": 32, "lanes": 4 }, "options": {"hspace": 400, "bits": 32, "lanes": 4}
        }


+-------+--------------+------------------------------------------+
| Field | Name         | Description                              |
+=======+==============+==========================================+
| [0]   | SFR_SCH_FUNC | sfr_sch_func read/write control register |
+-------+--------------+------------------------------------------+

SCEDMA_SFR_SCH_OPT
^^^^^^^^^^^^^^^^^^

`Address: 0x40029000 + 0x34 = 0x40029034`


    .. wavedrom::
        :caption: SCEDMA_SFR_SCH_OPT

        {
            "reg": [
                {"name": "sfr_sch_opt",  "bits": 8},
                {"bits": 24}
            ], "config": {"hspace": 400, "bits": 32, "lanes": 1 }, "options": {"hspace": 400, "bits": 32, "lanes": 1}
        }


+-------+-------------+-----------------------------------------+
| Field | Name        | Description                             |
+=======+=============+=========================================+
| [7:0] | SFR_SCH_OPT | sfr_sch_opt read/write control register |
+-------+-------------+-----------------------------------------+

SCEDMA_SFR_SCH_AXSTART
^^^^^^^^^^^^^^^^^^^^^^

`Address: 0x40029000 + 0x38 = 0x40029038`


    .. wavedrom::
        :caption: SCEDMA_SFR_SCH_AXSTART

        {
            "reg": [
                {"name": "sfr_sch_axstart",  "bits": 32}
            ], "config": {"hspace": 400, "bits": 32, "lanes": 1 }, "options": {"hspace": 400, "bits": 32, "lanes": 1}
        }


+--------+-----------------+---------------------------------------------+
| Field  | Name            | Description                                 |
+========+=================+=============================================+
| [31:0] | SFR_SCH_AXSTART | sfr_sch_axstart read/write control register |
+--------+-----------------+---------------------------------------------+

SCEDMA_SFR_SCH_SEGID
^^^^^^^^^^^^^^^^^^^^

`Address: 0x40029000 + 0x3c = 0x4002903c`


    .. wavedrom::
        :caption: SCEDMA_SFR_SCH_SEGID

        {
            "reg": [
                {"name": "sfr_sch_segid",  "bits": 8},
                {"bits": 24}
            ], "config": {"hspace": 400, "bits": 32, "lanes": 1 }, "options": {"hspace": 400, "bits": 32, "lanes": 1}
        }


+-------+---------------+-------------------------------------------+
| Field | Name          | Description                               |
+=======+===============+===========================================+
| [7:0] | SFR_SCH_SEGID | sfr_sch_segid read/write control register |
+-------+---------------+-------------------------------------------+

SCEDMA_SFR_SCH_SEGSTART
^^^^^^^^^^^^^^^^^^^^^^^

`Address: 0x40029000 + 0x40 = 0x40029040`


    .. wavedrom::
        :caption: SCEDMA_SFR_SCH_SEGSTART

        {
            "reg": [
                {"name": "schcr_segstart",  "bits": 12},
                {"bits": 20}
            ], "config": {"hspace": 400, "bits": 32, "lanes": 1 }, "options": {"hspace": 400, "bits": 32, "lanes": 1}
        }


+--------+----------------+--------------------------------------------+
| Field  | Name           | Description                                |
+========+================+============================================+
| [11:0] | SCHCR_SEGSTART | schcr_segstart read/write control register |
+--------+----------------+--------------------------------------------+

SCEDMA_SFR_SCH_TRANSIZE
^^^^^^^^^^^^^^^^^^^^^^^

`Address: 0x40029000 + 0x44 = 0x40029044`


    .. wavedrom::
        :caption: SCEDMA_SFR_SCH_TRANSIZE

        {
            "reg": [
                {"name": "schcr_transize",  "bits": 30},
                {"bits": 2}
            ], "config": {"hspace": 400, "bits": 32, "lanes": 1 }, "options": {"hspace": 400, "bits": 32, "lanes": 1}
        }


+--------+----------------+--------------------------------------------+
| Field  | Name           | Description                                |
+========+================+============================================+
| [29:0] | SCHCR_TRANSIZE | schcr_transize read/write control register |
+--------+----------------+--------------------------------------------+

SCEDMA_RESERVED18
^^^^^^^^^^^^^^^^^

`Address: 0x40029000 + 0x48 = 0x40029048`


    .. wavedrom::
        :caption: SCEDMA_RESERVED18

        {
            "reg": [
                {"name": "reserved18", "bits": 1},
                {"bits": 31},
            ], "config": {"hspace": 400, "bits": 32, "lanes": 4 }, "options": {"hspace": 400, "bits": 32, "lanes": 4}
        }


SCEDMA_RESERVED19
^^^^^^^^^^^^^^^^^

`Address: 0x40029000 + 0x4c = 0x4002904c`


    .. wavedrom::
        :caption: SCEDMA_RESERVED19

        {
            "reg": [
                {"name": "reserved19", "bits": 1},
                {"bits": 31},
            ], "config": {"hspace": 400, "bits": 32, "lanes": 4 }, "options": {"hspace": 400, "bits": 32, "lanes": 4}
        }


SCEDMA_SFR_ICH_OPT
^^^^^^^^^^^^^^^^^^

`Address: 0x40029000 + 0x50 = 0x40029050`


    .. wavedrom::
        :caption: SCEDMA_SFR_ICH_OPT

        {
            "reg": [
                {"name": "sfr_ich_opt",  "bits": 4},
                {"bits": 28}
            ], "config": {"hspace": 400, "bits": 32, "lanes": 4 }, "options": {"hspace": 400, "bits": 32, "lanes": 4}
        }


+-------+-------------+-----------------------------------------+
| Field | Name        | Description                             |
+=======+=============+=========================================+
| [3:0] | SFR_ICH_OPT | sfr_ich_opt read/write control register |
+-------+-------------+-----------------------------------------+

SCEDMA_SFR_ICH_SEGID
^^^^^^^^^^^^^^^^^^^^

`Address: 0x40029000 + 0x54 = 0x40029054`


    .. wavedrom::
        :caption: SCEDMA_SFR_ICH_SEGID

        {
            "reg": [
                {"name": "sfr_ich_segid",  "bits": 16},
                {"bits": 16}
            ], "config": {"hspace": 400, "bits": 32, "lanes": 1 }, "options": {"hspace": 400, "bits": 32, "lanes": 1}
        }


+--------+---------------+-------------------------------------------+
| Field  | Name          | Description                               |
+========+===============+===========================================+
| [15:0] | SFR_ICH_SEGID | sfr_ich_segid read/write control register |
+--------+---------------+-------------------------------------------+

SCEDMA_SFR_ICH_RPSTART
^^^^^^^^^^^^^^^^^^^^^^

`Address: 0x40029000 + 0x58 = 0x40029058`


    .. wavedrom::
        :caption: SCEDMA_SFR_ICH_RPSTART

        {
            "reg": [
                {"name": "ichcr_rpstart",  "bits": 12},
                {"bits": 20}
            ], "config": {"hspace": 400, "bits": 32, "lanes": 1 }, "options": {"hspace": 400, "bits": 32, "lanes": 1}
        }


+--------+---------------+-------------------------------------------+
| Field  | Name          | Description                               |
+========+===============+===========================================+
| [11:0] | ICHCR_RPSTART | ichcr_rpstart read/write control register |
+--------+---------------+-------------------------------------------+

SCEDMA_SFR_ICH_WPSTART
^^^^^^^^^^^^^^^^^^^^^^

`Address: 0x40029000 + 0x5c = 0x4002905c`


    .. wavedrom::
        :caption: SCEDMA_SFR_ICH_WPSTART

        {
            "reg": [
                {"name": "ichcr_wpstart",  "bits": 12},
                {"bits": 20}
            ], "config": {"hspace": 400, "bits": 32, "lanes": 1 }, "options": {"hspace": 400, "bits": 32, "lanes": 1}
        }


+--------+---------------+-------------------------------------------+
| Field  | Name          | Description                               |
+========+===============+===========================================+
| [11:0] | ICHCR_WPSTART | ichcr_wpstart read/write control register |
+--------+---------------+-------------------------------------------+

SCEDMA_SFR_ICH_TRANSIZE
^^^^^^^^^^^^^^^^^^^^^^^

`Address: 0x40029000 + 0x60 = 0x40029060`


    .. wavedrom::
        :caption: SCEDMA_SFR_ICH_TRANSIZE

        {
            "reg": [
                {"name": "ichcr_transize",  "bits": 12},
                {"bits": 20}
            ], "config": {"hspace": 400, "bits": 32, "lanes": 1 }, "options": {"hspace": 400, "bits": 32, "lanes": 1}
        }


+--------+----------------+--------------------------------------------+
| Field  | Name           | Description                                |
+========+================+============================================+
| [11:0] | ICHCR_TRANSIZE | ichcr_transize read/write control register |
+--------+----------------+--------------------------------------------+

