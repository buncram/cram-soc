TRNG
====

Register Listing for TRNG
-------------------------

+--------------------------------------------+--------------------------------------+
| Register                                   | Address                              |
+============================================+======================================+
| :ref:`TRNG_SFR_CRSRC <TRNG_SFR_CRSRC>`     | :ref:`0x4002e000 <TRNG_SFR_CRSRC>`   |
+--------------------------------------------+--------------------------------------+
| :ref:`TRNG_SFR_AR_STOP <TRNG_SFR_AR_STOP>` | :ref:`0x4002e004 <TRNG_SFR_AR_STOP>` |
+--------------------------------------------+--------------------------------------+
| :ref:`TRNG_SFR_PP <TRNG_SFR_PP>`           | :ref:`0x4002e008 <TRNG_SFR_PP>`      |
+--------------------------------------------+--------------------------------------+
| :ref:`TRNG_SFR_OPT <TRNG_SFR_OPT>`         | :ref:`0x4002e00c <TRNG_SFR_OPT>`     |
+--------------------------------------------+--------------------------------------+
| :ref:`TRNG_SFR_SR <TRNG_SFR_SR>`           | :ref:`0x4002e010 <TRNG_SFR_SR>`      |
+--------------------------------------------+--------------------------------------+

TRNG_SFR_CRSRC
^^^^^^^^^^^^^^

`Address: 0x4002e000 + 0x0 = 0x4002e000`


    .. wavedrom::
        :caption: TRNG_SFR_CRSRC

        {
            "reg": [
                {"name": "sfr_crsrc",  "bits": 12},
                {"bits": 20}
            ], "config": {"hspace": 400, "bits": 32, "lanes": 1 }, "options": {"hspace": 400, "bits": 32, "lanes": 1}
        }


+--------+-----------+---------------------------------------+
| Field  | Name      | Description                           |
+========+===========+=======================================+
| [11:0] | SFR_CRSRC | sfr_crsrc read/write control register |
+--------+-----------+---------------------------------------+

TRNG_SFR_AR_STOP
^^^^^^^^^^^^^^^^

`Address: 0x4002e000 + 0x4 = 0x4002e004`


    .. wavedrom::
        :caption: TRNG_SFR_AR_STOP

        {
            "reg": [
                {"name": "sfr_ar_stop",  "type": 4, "bits": 32}
            ], "config": {"hspace": 400, "bits": 32, "lanes": 1 }, "options": {"hspace": 400, "bits": 32, "lanes": 1}
        }


+--------+-------------+-----------------------------------------------------+
| Field  | Name        | Description                                         |
+========+=============+=====================================================+
| [31:0] | SFR_AR_STOP | sfr_ar_stop performs action on write of value: 0xa5 |
+--------+-------------+-----------------------------------------------------+

TRNG_SFR_PP
^^^^^^^^^^^

`Address: 0x4002e000 + 0x8 = 0x4002e008`


    .. wavedrom::
        :caption: TRNG_SFR_PP

        {
            "reg": [
                {"name": "sfr_pp",  "bits": 17},
                {"bits": 15}
            ], "config": {"hspace": 400, "bits": 32, "lanes": 1 }, "options": {"hspace": 400, "bits": 32, "lanes": 1}
        }


+--------+--------+------------------------------------+
| Field  | Name   | Description                        |
+========+========+====================================+
| [16:0] | SFR_PP | sfr_pp read/write control register |
+--------+--------+------------------------------------+

TRNG_SFR_OPT
^^^^^^^^^^^^

`Address: 0x4002e000 + 0xc = 0x4002e00c`


    .. wavedrom::
        :caption: TRNG_SFR_OPT

        {
            "reg": [
                {"name": "sfr_opt",  "bits": 17},
                {"bits": 15}
            ], "config": {"hspace": 400, "bits": 32, "lanes": 1 }, "options": {"hspace": 400, "bits": 32, "lanes": 1}
        }


+--------+---------+-------------------------------------+
| Field  | Name    | Description                         |
+========+=========+=====================================+
| [16:0] | SFR_OPT | sfr_opt read/write control register |
+--------+---------+-------------------------------------+

TRNG_SFR_SR
^^^^^^^^^^^

`Address: 0x4002e000 + 0x10 = 0x4002e010`


    .. wavedrom::
        :caption: TRNG_SFR_SR

        {
            "reg": [
                {"name": "sr_rng",  "bits": 32}
            ], "config": {"hspace": 400, "bits": 32, "lanes": 1 }, "options": {"hspace": 400, "bits": 32, "lanes": 1}
        }


+--------+--------+----------------------------------+
| Field  | Name   | Description                      |
+========+========+==================================+
| [31:0] | SR_RNG | sr_rng read only status register |
+--------+--------+----------------------------------+

