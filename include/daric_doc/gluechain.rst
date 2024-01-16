GLUECHAIN
=========

Register Listing for GLUECHAIN
------------------------------

+----------------------------------------------------+------------------------------------------+
| Register                                           | Address                                  |
+====================================================+==========================================+
| :ref:`GLUECHAIN_SFR_GCMASK <GLUECHAIN_SFR_GCMASK>` | :ref:`0x40054000 <GLUECHAIN_SFR_GCMASK>` |
+----------------------------------------------------+------------------------------------------+
| :ref:`GLUECHAIN_SFR_GCSR <GLUECHAIN_SFR_GCSR>`     | :ref:`0x40054004 <GLUECHAIN_SFR_GCSR>`   |
+----------------------------------------------------+------------------------------------------+
| :ref:`GLUECHAIN_SFR_GCRST <GLUECHAIN_SFR_GCRST>`   | :ref:`0x40054008 <GLUECHAIN_SFR_GCRST>`  |
+----------------------------------------------------+------------------------------------------+
| :ref:`GLUECHAIN_SFR_GCTEST <GLUECHAIN_SFR_GCTEST>` | :ref:`0x4005400c <GLUECHAIN_SFR_GCTEST>` |
+----------------------------------------------------+------------------------------------------+

GLUECHAIN_SFR_GCMASK
^^^^^^^^^^^^^^^^^^^^

`Address: 0x40054000 + 0x0 = 0x40054000`

    See file:///F:/code/cram-soc/soc-oss/rtl/sec/gluechain_v0.1.sv

    .. wavedrom::
        :caption: GLUECHAIN_SFR_GCMASK

        {
            "reg": [
                {"name": "cr_gcmask",  "bits": 32}
            ], "config": {"hspace": 400, "bits": 32, "lanes": 1 }, "options": {"hspace": 400, "bits": 32, "lanes": 1}
        }


+--------+-----------+---------------------------------------+
| Field  | Name      | Description                           |
+========+===========+=======================================+
| [31:0] | CR_GCMASK | cr_gcmask read/write control register |
+--------+-----------+---------------------------------------+

GLUECHAIN_SFR_GCSR
^^^^^^^^^^^^^^^^^^

`Address: 0x40054000 + 0x4 = 0x40054004`

    See file:///F:/code/cram-soc/soc-oss/rtl/sec/gluechain_v0.1.sv

    .. wavedrom::
        :caption: GLUECHAIN_SFR_GCSR

        {
            "reg": [
                {"name": "gluereg",  "bits": 32}
            ], "config": {"hspace": 400, "bits": 32, "lanes": 1 }, "options": {"hspace": 400, "bits": 32, "lanes": 1}
        }


+--------+---------+-----------------------------------+
| Field  | Name    | Description                       |
+========+=========+===================================+
| [31:0] | GLUEREG | gluereg read only status register |
+--------+---------+-----------------------------------+

GLUECHAIN_SFR_GCRST
^^^^^^^^^^^^^^^^^^^

`Address: 0x40054000 + 0x8 = 0x40054008`

    See file:///F:/code/cram-soc/soc-oss/rtl/sec/gluechain_v0.1.sv

    .. wavedrom::
        :caption: GLUECHAIN_SFR_GCRST

        {
            "reg": [
                {"name": "gluerst",  "bits": 32}
            ], "config": {"hspace": 400, "bits": 32, "lanes": 1 }, "options": {"hspace": 400, "bits": 32, "lanes": 1}
        }


+--------+---------+-------------------------------------+
| Field  | Name    | Description                         |
+========+=========+=====================================+
| [31:0] | GLUERST | gluerst read/write control register |
+--------+---------+-------------------------------------+

GLUECHAIN_SFR_GCTEST
^^^^^^^^^^^^^^^^^^^^

`Address: 0x40054000 + 0xc = 0x4005400c`

    See file:///F:/code/cram-soc/soc-oss/rtl/sec/gluechain_v0.1.sv

    .. wavedrom::
        :caption: GLUECHAIN_SFR_GCTEST

        {
            "reg": [
                {"name": "gluetest",  "bits": 32}
            ], "config": {"hspace": 400, "bits": 32, "lanes": 1 }, "options": {"hspace": 400, "bits": 32, "lanes": 1}
        }


+--------+----------+--------------------------------------+
| Field  | Name     | Description                          |
+========+==========+======================================+
| [31:0] | GLUETEST | gluetest read/write control register |
+--------+----------+--------------------------------------+

