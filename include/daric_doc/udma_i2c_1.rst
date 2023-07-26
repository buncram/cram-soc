UDMA_I2C_1
==========

Register Listing for UDMA_I2C_1
-------------------------------

+------------------------------------------------------------+----------------------------------------------+
| Register                                                   | Address                                      |
+============================================================+==============================================+
| :ref:`UDMA_I2C_1_REG_RX_SADDR <UDMA_I2C_1_REG_RX_SADDR>`   | :ref:`0x5010a000 <UDMA_I2C_1_REG_RX_SADDR>`  |
+------------------------------------------------------------+----------------------------------------------+
| :ref:`UDMA_I2C_1_REG_RX_SIZE <UDMA_I2C_1_REG_RX_SIZE>`     | :ref:`0x5010a004 <UDMA_I2C_1_REG_RX_SIZE>`   |
+------------------------------------------------------------+----------------------------------------------+
| :ref:`UDMA_I2C_1_REG_RX_CFG <UDMA_I2C_1_REG_RX_CFG>`       | :ref:`0x5010a008 <UDMA_I2C_1_REG_RX_CFG>`    |
+------------------------------------------------------------+----------------------------------------------+
| :ref:`UDMA_I2C_1_REG_TX_SADDR <UDMA_I2C_1_REG_TX_SADDR>`   | :ref:`0x5010a010 <UDMA_I2C_1_REG_TX_SADDR>`  |
+------------------------------------------------------------+----------------------------------------------+
| :ref:`UDMA_I2C_1_REG_TX_SIZE <UDMA_I2C_1_REG_TX_SIZE>`     | :ref:`0x5010a014 <UDMA_I2C_1_REG_TX_SIZE>`   |
+------------------------------------------------------------+----------------------------------------------+
| :ref:`UDMA_I2C_1_REG_TX_CFG <UDMA_I2C_1_REG_TX_CFG>`       | :ref:`0x5010a018 <UDMA_I2C_1_REG_TX_CFG>`    |
+------------------------------------------------------------+----------------------------------------------+
| :ref:`UDMA_I2C_1_REG_CMD_SADDR <UDMA_I2C_1_REG_CMD_SADDR>` | :ref:`0x5010a020 <UDMA_I2C_1_REG_CMD_SADDR>` |
+------------------------------------------------------------+----------------------------------------------+
| :ref:`UDMA_I2C_1_REG_CMD_SIZE <UDMA_I2C_1_REG_CMD_SIZE>`   | :ref:`0x5010a024 <UDMA_I2C_1_REG_CMD_SIZE>`  |
+------------------------------------------------------------+----------------------------------------------+
| :ref:`UDMA_I2C_1_REG_CMD_CFG <UDMA_I2C_1_REG_CMD_CFG>`     | :ref:`0x5010a028 <UDMA_I2C_1_REG_CMD_CFG>`   |
+------------------------------------------------------------+----------------------------------------------+
| :ref:`UDMA_I2C_1_REG_STATUS <UDMA_I2C_1_REG_STATUS>`       | :ref:`0x5010a030 <UDMA_I2C_1_REG_STATUS>`    |
+------------------------------------------------------------+----------------------------------------------+
| :ref:`UDMA_I2C_1_REG_SETUP <UDMA_I2C_1_REG_SETUP>`         | :ref:`0x5010a034 <UDMA_I2C_1_REG_SETUP>`     |
+------------------------------------------------------------+----------------------------------------------+
| :ref:`UDMA_I2C_1_REG_ACK <UDMA_I2C_1_REG_ACK>`             | :ref:`0x5010a038 <UDMA_I2C_1_REG_ACK>`       |
+------------------------------------------------------------+----------------------------------------------+

UDMA_I2C_1_REG_RX_SADDR
^^^^^^^^^^^^^^^^^^^^^^^

`Address: 0x5010a000 + 0x0 = 0x5010a000`


    .. wavedrom::
        :caption: UDMA_I2C_1_REG_RX_SADDR

        {
            "reg": [
                {"name": "r_rx_startaddr",  "bits": 12},
                {"bits": 20}
            ], "config": {"hspace": 400, "bits": 32, "lanes": 1 }, "options": {"hspace": 400, "bits": 32, "lanes": 1}
        }


+--------+----------------+----------------+
| Field  | Name           | Description    |
+========+================+================+
| [11:0] | R_RX_STARTADDR | r_rx_startaddr |
+--------+----------------+----------------+

UDMA_I2C_1_REG_RX_SIZE
^^^^^^^^^^^^^^^^^^^^^^

`Address: 0x5010a000 + 0x4 = 0x5010a004`


    .. wavedrom::
        :caption: UDMA_I2C_1_REG_RX_SIZE

        {
            "reg": [
                {"name": "r_rx_size",  "bits": 16},
                {"bits": 16}
            ], "config": {"hspace": 400, "bits": 32, "lanes": 1 }, "options": {"hspace": 400, "bits": 32, "lanes": 1}
        }


+--------+-----------+-------------+
| Field  | Name      | Description |
+========+===========+=============+
| [15:0] | R_RX_SIZE | r_rx_size   |
+--------+-----------+-------------+

UDMA_I2C_1_REG_RX_CFG
^^^^^^^^^^^^^^^^^^^^^

`Address: 0x5010a000 + 0x8 = 0x5010a008`


    .. wavedrom::
        :caption: UDMA_I2C_1_REG_RX_CFG

        {
            "reg": [
                {"name": "r_rx_continuous",  "bits": 1},
                {"bits": 3},
                {"name": "r_rx_en",  "bits": 1},
                {"bits": 1},
                {"name": "r_rx_clr",  "bits": 1},
                {"bits": 25}
            ], "config": {"hspace": 400, "bits": 32, "lanes": 4 }, "options": {"hspace": 400, "bits": 32, "lanes": 4}
        }


+-------+-----------------+-----------------+
| Field | Name            | Description     |
+=======+=================+=================+
| [0]   | R_RX_CONTINUOUS | r_rx_continuous |
+-------+-----------------+-----------------+
| [4]   | R_RX_EN         | r_rx_en         |
+-------+-----------------+-----------------+
| [6]   | R_RX_CLR        | r_rx_clr        |
+-------+-----------------+-----------------+

UDMA_I2C_1_REG_TX_SADDR
^^^^^^^^^^^^^^^^^^^^^^^

`Address: 0x5010a000 + 0x10 = 0x5010a010`


    .. wavedrom::
        :caption: UDMA_I2C_1_REG_TX_SADDR

        {
            "reg": [
                {"name": "r_tx_startaddr",  "bits": 12},
                {"bits": 20}
            ], "config": {"hspace": 400, "bits": 32, "lanes": 1 }, "options": {"hspace": 400, "bits": 32, "lanes": 1}
        }


+--------+----------------+----------------+
| Field  | Name           | Description    |
+========+================+================+
| [11:0] | R_TX_STARTADDR | r_tx_startaddr |
+--------+----------------+----------------+

UDMA_I2C_1_REG_TX_SIZE
^^^^^^^^^^^^^^^^^^^^^^

`Address: 0x5010a000 + 0x14 = 0x5010a014`


    .. wavedrom::
        :caption: UDMA_I2C_1_REG_TX_SIZE

        {
            "reg": [
                {"name": "r_tx_size",  "bits": 16},
                {"bits": 16}
            ], "config": {"hspace": 400, "bits": 32, "lanes": 1 }, "options": {"hspace": 400, "bits": 32, "lanes": 1}
        }


+--------+-----------+-------------+
| Field  | Name      | Description |
+========+===========+=============+
| [15:0] | R_TX_SIZE | r_tx_size   |
+--------+-----------+-------------+

UDMA_I2C_1_REG_TX_CFG
^^^^^^^^^^^^^^^^^^^^^

`Address: 0x5010a000 + 0x18 = 0x5010a018`


    .. wavedrom::
        :caption: UDMA_I2C_1_REG_TX_CFG

        {
            "reg": [
                {"name": "r_tx_continuous",  "bits": 1},
                {"bits": 3},
                {"name": "r_tx_en",  "bits": 1},
                {"bits": 1},
                {"name": "r_tx_clr",  "bits": 1},
                {"bits": 25}
            ], "config": {"hspace": 400, "bits": 32, "lanes": 4 }, "options": {"hspace": 400, "bits": 32, "lanes": 4}
        }


+-------+-----------------+-----------------+
| Field | Name            | Description     |
+=======+=================+=================+
| [0]   | R_TX_CONTINUOUS | r_tx_continuous |
+-------+-----------------+-----------------+
| [4]   | R_TX_EN         | r_tx_en         |
+-------+-----------------+-----------------+
| [6]   | R_TX_CLR        | r_tx_clr        |
+-------+-----------------+-----------------+

UDMA_I2C_1_REG_CMD_SADDR
^^^^^^^^^^^^^^^^^^^^^^^^

`Address: 0x5010a000 + 0x20 = 0x5010a020`


    .. wavedrom::
        :caption: UDMA_I2C_1_REG_CMD_SADDR

        {
            "reg": [
                {"name": "r_cmd_startaddr",  "bits": 12},
                {"bits": 20}
            ], "config": {"hspace": 400, "bits": 32, "lanes": 1 }, "options": {"hspace": 400, "bits": 32, "lanes": 1}
        }


+--------+-----------------+-----------------+
| Field  | Name            | Description     |
+========+=================+=================+
| [11:0] | R_CMD_STARTADDR | r_cmd_startaddr |
+--------+-----------------+-----------------+

UDMA_I2C_1_REG_CMD_SIZE
^^^^^^^^^^^^^^^^^^^^^^^

`Address: 0x5010a000 + 0x24 = 0x5010a024`


    .. wavedrom::
        :caption: UDMA_I2C_1_REG_CMD_SIZE

        {
            "reg": [
                {"name": "r_cmd_size",  "bits": 16},
                {"bits": 16}
            ], "config": {"hspace": 400, "bits": 32, "lanes": 1 }, "options": {"hspace": 400, "bits": 32, "lanes": 1}
        }


+--------+------------+-------------+
| Field  | Name       | Description |
+========+============+=============+
| [15:0] | R_CMD_SIZE | r_cmd_size  |
+--------+------------+-------------+

UDMA_I2C_1_REG_CMD_CFG
^^^^^^^^^^^^^^^^^^^^^^

`Address: 0x5010a000 + 0x28 = 0x5010a028`


    .. wavedrom::
        :caption: UDMA_I2C_1_REG_CMD_CFG

        {
            "reg": [
                {"name": "r_cmd_continuous",  "bits": 1},
                {"bits": 3},
                {"name": "r_cmd_en",  "bits": 1},
                {"bits": 1},
                {"name": "r_cmd_clr",  "bits": 1},
                {"bits": 25}
            ], "config": {"hspace": 400, "bits": 32, "lanes": 4 }, "options": {"hspace": 400, "bits": 32, "lanes": 4}
        }


+-------+------------------+------------------+
| Field | Name             | Description      |
+=======+==================+==================+
| [0]   | R_CMD_CONTINUOUS | r_cmd_continuous |
+-------+------------------+------------------+
| [4]   | R_CMD_EN         | r_cmd_en         |
+-------+------------------+------------------+
| [6]   | R_CMD_CLR        | r_cmd_clr        |
+-------+------------------+------------------+

UDMA_I2C_1_REG_STATUS
^^^^^^^^^^^^^^^^^^^^^

`Address: 0x5010a000 + 0x30 = 0x5010a030`


    .. wavedrom::
        :caption: UDMA_I2C_1_REG_STATUS

        {
            "reg": [
                {"name": "r_busy",  "bits": 1},
                {"name": "r_al",  "bits": 1},
                {"bits": 30}
            ], "config": {"hspace": 400, "bits": 32, "lanes": 4 }, "options": {"hspace": 400, "bits": 32, "lanes": 4}
        }


+-------+--------+-------------+
| Field | Name   | Description |
+=======+========+=============+
| [0]   | R_BUSY | r_busy      |
+-------+--------+-------------+
| [1]   | R_AL   | r_al        |
+-------+--------+-------------+

UDMA_I2C_1_REG_SETUP
^^^^^^^^^^^^^^^^^^^^

`Address: 0x5010a000 + 0x34 = 0x5010a034`


    .. wavedrom::
        :caption: UDMA_I2C_1_REG_SETUP

        {
            "reg": [
                {"name": "r_do_rst",  "bits": 1},
                {"bits": 31}
            ], "config": {"hspace": 400, "bits": 32, "lanes": 4 }, "options": {"hspace": 400, "bits": 32, "lanes": 4}
        }


+-------+----------+-------------+
| Field | Name     | Description |
+=======+==========+=============+
| [0]   | R_DO_RST | r_do_rst    |
+-------+----------+-------------+

UDMA_I2C_1_REG_ACK
^^^^^^^^^^^^^^^^^^

`Address: 0x5010a000 + 0x38 = 0x5010a038`


    .. wavedrom::
        :caption: UDMA_I2C_1_REG_ACK

        {
            "reg": [
                {"name": "r_nack",  "bits": 1},
                {"bits": 31}
            ], "config": {"hspace": 400, "bits": 32, "lanes": 4 }, "options": {"hspace": 400, "bits": 32, "lanes": 4}
        }


+-------+--------+-------------+
| Field | Name   | Description |
+=======+========+=============+
| [0]   | R_NACK | r_nack      |
+-------+--------+-------------+

