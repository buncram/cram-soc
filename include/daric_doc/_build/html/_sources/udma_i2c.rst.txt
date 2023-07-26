UDMA_I2C
========

Register Listing for UDMA_I2C
-----------------------------

+------------------------------------------------------------+----------------------------------------------+
| Register                                                   | Address                                      |
+============================================================+==============================================+
| :ref:`UDMA_I2C_REG_RX_SADDR <UDMA_I2C_REG_RX_SADDR>`       | :ref:`0x50109000 <UDMA_I2C_REG_RX_SADDR>`    |
+------------------------------------------------------------+----------------------------------------------+
| :ref:`UDMA_I2C_REG_RX_SIZE <UDMA_I2C_REG_RX_SIZE>`         | :ref:`0x50109004 <UDMA_I2C_REG_RX_SIZE>`     |
+------------------------------------------------------------+----------------------------------------------+
| :ref:`UDMA_I2C_REG_RX_CFG <UDMA_I2C_REG_RX_CFG>`           | :ref:`0x50109008 <UDMA_I2C_REG_RX_CFG>`      |
+------------------------------------------------------------+----------------------------------------------+
| :ref:`UDMA_I2C_REG_RX_INTCFG <UDMA_I2C_REG_RX_INTCFG>`     | :ref:`0x5010900c <UDMA_I2C_REG_RX_INTCFG>`   |
+------------------------------------------------------------+----------------------------------------------+
| :ref:`UDMA_I2C_REG_TX_SADDR <UDMA_I2C_REG_TX_SADDR>`       | :ref:`0x50109010 <UDMA_I2C_REG_TX_SADDR>`    |
+------------------------------------------------------------+----------------------------------------------+
| :ref:`UDMA_I2C_REG_TX_SIZE <UDMA_I2C_REG_TX_SIZE>`         | :ref:`0x50109014 <UDMA_I2C_REG_TX_SIZE>`     |
+------------------------------------------------------------+----------------------------------------------+
| :ref:`UDMA_I2C_REG_TX_CFG <UDMA_I2C_REG_TX_CFG>`           | :ref:`0x50109018 <UDMA_I2C_REG_TX_CFG>`      |
+------------------------------------------------------------+----------------------------------------------+
| :ref:`UDMA_I2C_REG_TX_INTCFG <UDMA_I2C_REG_TX_INTCFG>`     | :ref:`0x5010901c <UDMA_I2C_REG_TX_INTCFG>`   |
+------------------------------------------------------------+----------------------------------------------+
| :ref:`UDMA_I2C_REG_STATUS <UDMA_I2C_REG_STATUS>`           | :ref:`0x50109020 <UDMA_I2C_REG_STATUS>`      |
+------------------------------------------------------------+----------------------------------------------+
| :ref:`UDMA_I2C_REG_SCIF_SETUP <UDMA_I2C_REG_SCIF_SETUP>`   | :ref:`0x50109024 <UDMA_I2C_REG_SCIF_SETUP>`  |
+------------------------------------------------------------+----------------------------------------------+
| :ref:`UDMA_I2C_REG_ERROR <UDMA_I2C_REG_ERROR>`             | :ref:`0x50109028 <UDMA_I2C_REG_ERROR>`       |
+------------------------------------------------------------+----------------------------------------------+
| :ref:`UDMA_I2C_REG_IRQ_EN <UDMA_I2C_REG_IRQ_EN>`           | :ref:`0x5010902c <UDMA_I2C_REG_IRQ_EN>`      |
+------------------------------------------------------------+----------------------------------------------+
| :ref:`UDMA_I2C_REG_VALID <UDMA_I2C_REG_VALID>`             | :ref:`0x50109030 <UDMA_I2C_REG_VALID>`       |
+------------------------------------------------------------+----------------------------------------------+
| :ref:`UDMA_I2C_REG_DATA <UDMA_I2C_REG_DATA>`               | :ref:`0x50109034 <UDMA_I2C_REG_DATA>`        |
+------------------------------------------------------------+----------------------------------------------+
| :ref:`UDMA_I2C_REG_SCIF_ETU <UDMA_I2C_REG_SCIF_ETU>`       | :ref:`0x50109038 <UDMA_I2C_REG_SCIF_ETU>`    |
+------------------------------------------------------------+----------------------------------------------+
| :ref:`UDMA_I2C_REG_AU_CFG <UDMA_I2C_REG_AU_CFG>`           | :ref:`0x5010903c <UDMA_I2C_REG_AU_CFG>`      |
+------------------------------------------------------------+----------------------------------------------+
| :ref:`UDMA_I2C_REG_AU_REG0 <UDMA_I2C_REG_AU_REG0>`         | :ref:`0x50109040 <UDMA_I2C_REG_AU_REG0>`     |
+------------------------------------------------------------+----------------------------------------------+
| :ref:`UDMA_I2C_REG_AU_REG1 <UDMA_I2C_REG_AU_REG1>`         | :ref:`0x50109044 <UDMA_I2C_REG_AU_REG1>`     |
+------------------------------------------------------------+----------------------------------------------+
| :ref:`UDMA_I2C_REG_BINCU_TH <UDMA_I2C_REG_BINCU_TH>`       | :ref:`0x50109048 <UDMA_I2C_REG_BINCU_TH>`    |
+------------------------------------------------------------+----------------------------------------------+
| :ref:`UDMA_I2C_REG_BINCU_CNT <UDMA_I2C_REG_BINCU_CNT>`     | :ref:`0x5010904c <UDMA_I2C_REG_BINCU_CNT>`   |
+------------------------------------------------------------+----------------------------------------------+
| :ref:`UDMA_I2C_REG_BINCU_SETUP <UDMA_I2C_REG_BINCU_SETUP>` | :ref:`0x50109050 <UDMA_I2C_REG_BINCU_SETUP>` |
+------------------------------------------------------------+----------------------------------------------+
| :ref:`UDMA_I2C_REG_BINCU_VAL <UDMA_I2C_REG_BINCU_VAL>`     | :ref:`0x50109054 <UDMA_I2C_REG_BINCU_VAL>`   |
+------------------------------------------------------------+----------------------------------------------+
| :ref:`UDMA_I2C_REG_FILT <UDMA_I2C_REG_FILT>`               | :ref:`0x50109058 <UDMA_I2C_REG_FILT>`        |
+------------------------------------------------------------+----------------------------------------------+
| :ref:`UDMA_I2C_REG_FILT_CMD <UDMA_I2C_REG_FILT_CMD>`       | :ref:`0x5010905c <UDMA_I2C_REG_FILT_CMD>`    |
+------------------------------------------------------------+----------------------------------------------+
| :ref:`UDMA_I2C_REG_STATUS <UDMA_I2C_REG_STATUS>`           | :ref:`0x50109060 <UDMA_I2C_REG_STATUS>`      |
+------------------------------------------------------------+----------------------------------------------+

UDMA_I2C_REG_RX_SADDR
^^^^^^^^^^^^^^^^^^^^^

`Address: 0x50109000 + 0x0 = 0x50109000`


    .. wavedrom::
        :caption: UDMA_I2C_REG_RX_SADDR

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

UDMA_I2C_REG_RX_SIZE
^^^^^^^^^^^^^^^^^^^^

`Address: 0x50109000 + 0x4 = 0x50109004`


    .. wavedrom::
        :caption: UDMA_I2C_REG_RX_SIZE

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

UDMA_I2C_REG_RX_CFG
^^^^^^^^^^^^^^^^^^^

`Address: 0x50109000 + 0x8 = 0x50109008`


    .. wavedrom::
        :caption: UDMA_I2C_REG_RX_CFG

        {
            "reg": [
                {"name": "r_rx_continuous",  "bits": 1},
                {"bits": 31}
            ], "config": {"hspace": 400, "bits": 32, "lanes": 4 }, "options": {"hspace": 400, "bits": 32, "lanes": 4}
        }


+-------+-----------------+-----------------+
| Field | Name            | Description     |
+=======+=================+=================+
| [0]   | R_RX_CONTINUOUS | r_rx_continuous |
+-------+-----------------+-----------------+

UDMA_I2C_REG_RX_INTCFG
^^^^^^^^^^^^^^^^^^^^^^

`Address: 0x50109000 + 0xc = 0x5010900c`


    .. wavedrom::
        :caption: UDMA_I2C_REG_RX_INTCFG

        {
            "reg": [
                {"name": "reg_rx_intcfg", "bits": 1},
                {"bits": 31},
            ], "config": {"hspace": 400, "bits": 32, "lanes": 4 }, "options": {"hspace": 400, "bits": 32, "lanes": 4}
        }


UDMA_I2C_REG_TX_SADDR
^^^^^^^^^^^^^^^^^^^^^

`Address: 0x50109000 + 0x10 = 0x50109010`


    .. wavedrom::
        :caption: UDMA_I2C_REG_TX_SADDR

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

UDMA_I2C_REG_TX_SIZE
^^^^^^^^^^^^^^^^^^^^

`Address: 0x50109000 + 0x14 = 0x50109014`


    .. wavedrom::
        :caption: UDMA_I2C_REG_TX_SIZE

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

UDMA_I2C_REG_TX_CFG
^^^^^^^^^^^^^^^^^^^

`Address: 0x50109000 + 0x18 = 0x50109018`


    .. wavedrom::
        :caption: UDMA_I2C_REG_TX_CFG

        {
            "reg": [
                {"name": "r_tx_continuous",  "bits": 1},
                {"bits": 31}
            ], "config": {"hspace": 400, "bits": 32, "lanes": 4 }, "options": {"hspace": 400, "bits": 32, "lanes": 4}
        }


+-------+-----------------+-----------------+
| Field | Name            | Description     |
+=======+=================+=================+
| [0]   | R_TX_CONTINUOUS | r_tx_continuous |
+-------+-----------------+-----------------+

UDMA_I2C_REG_TX_INTCFG
^^^^^^^^^^^^^^^^^^^^^^

`Address: 0x50109000 + 0x1c = 0x5010901c`


    .. wavedrom::
        :caption: UDMA_I2C_REG_TX_INTCFG

        {
            "reg": [
                {"name": "reg_tx_intcfg", "bits": 1},
                {"bits": 31},
            ], "config": {"hspace": 400, "bits": 32, "lanes": 4 }, "options": {"hspace": 400, "bits": 32, "lanes": 4}
        }


UDMA_I2C_REG_STATUS
^^^^^^^^^^^^^^^^^^^

`Address: 0x50109000 + 0x20 = 0x50109020`


    .. wavedrom::
        :caption: UDMA_I2C_REG_STATUS

        {
            "reg": [
                {"name": "reg_status", "bits": 1},
                {"bits": 31},
            ], "config": {"hspace": 400, "bits": 32, "lanes": 4 }, "options": {"hspace": 400, "bits": 32, "lanes": 4}
        }


UDMA_I2C_REG_SCIF_SETUP
^^^^^^^^^^^^^^^^^^^^^^^

`Address: 0x50109000 + 0x24 = 0x50109024`


    .. wavedrom::
        :caption: UDMA_I2C_REG_SCIF_SETUP

        {
            "reg": [
                {"name": "r_scif_div",  "bits": 16},
                {"name": "r_scif_clksel",  "bits": 2},
                {"name": "r_scif_en_rx",  "bits": 1},
                {"name": "r_scif_en_tx",  "bits": 1},
                {"name": "r_scif_rx_clean_fifo",  "bits": 1},
                {"name": "r_scif_rx_polling_en",  "bits": 1},
                {"name": "r_scif_stop_bits",  "bits": 1},
                {"name": "r_scif_bits",  "bits": 2},
                {"name": "r_scif_parity_en",  "bits": 1},
                {"bits": 6}
            ], "config": {"hspace": 400, "bits": 32, "lanes": 4 }, "options": {"hspace": 400, "bits": 32, "lanes": 4}
        }


+---------+----------------------+----------------------+
| Field   | Name                 | Description          |
+=========+======================+======================+
| [15:0]  | R_SCIF_DIV           | r_scif_div           |
+---------+----------------------+----------------------+
| [17:16] | R_SCIF_CLKSEL        | r_scif_clksel        |
+---------+----------------------+----------------------+
| [18]    | R_SCIF_EN_RX         | r_scif_en_rx         |
+---------+----------------------+----------------------+
| [19]    | R_SCIF_EN_TX         | r_scif_en_tx         |
+---------+----------------------+----------------------+
| [20]    | R_SCIF_RX_CLEAN_FIFO | r_scif_rx_clean_fifo |
+---------+----------------------+----------------------+
| [21]    | R_SCIF_RX_POLLING_EN | r_scif_rx_polling_en |
+---------+----------------------+----------------------+
| [22]    | R_SCIF_STOP_BITS     | r_scif_stop_bits     |
+---------+----------------------+----------------------+
| [24:23] | R_SCIF_BITS          | r_scif_bits          |
+---------+----------------------+----------------------+
| [25]    | R_SCIF_PARITY_EN     | r_scif_parity_en     |
+---------+----------------------+----------------------+

UDMA_I2C_REG_ERROR
^^^^^^^^^^^^^^^^^^

`Address: 0x50109000 + 0x28 = 0x50109028`


    .. wavedrom::
        :caption: UDMA_I2C_REG_ERROR

        {
            "reg": [
                {"name": "reg_error", "bits": 1},
                {"bits": 31},
            ], "config": {"hspace": 400, "bits": 32, "lanes": 4 }, "options": {"hspace": 400, "bits": 32, "lanes": 4}
        }


UDMA_I2C_REG_IRQ_EN
^^^^^^^^^^^^^^^^^^^

`Address: 0x50109000 + 0x2c = 0x5010902c`


    .. wavedrom::
        :caption: UDMA_I2C_REG_IRQ_EN

        {
            "reg": [
                {"name": "reg_irq_en", "bits": 1},
                {"bits": 31},
            ], "config": {"hspace": 400, "bits": 32, "lanes": 4 }, "options": {"hspace": 400, "bits": 32, "lanes": 4}
        }


UDMA_I2C_REG_VALID
^^^^^^^^^^^^^^^^^^

`Address: 0x50109000 + 0x30 = 0x50109030`


    .. wavedrom::
        :caption: UDMA_I2C_REG_VALID

        {
            "reg": [
                {"name": "reg_valid", "bits": 1},
                {"bits": 31},
            ], "config": {"hspace": 400, "bits": 32, "lanes": 4 }, "options": {"hspace": 400, "bits": 32, "lanes": 4}
        }


UDMA_I2C_REG_DATA
^^^^^^^^^^^^^^^^^

`Address: 0x50109000 + 0x34 = 0x50109034`


    .. wavedrom::
        :caption: UDMA_I2C_REG_DATA

        {
            "reg": [
                {"name": "reg_data", "bits": 1},
                {"bits": 31},
            ], "config": {"hspace": 400, "bits": 32, "lanes": 4 }, "options": {"hspace": 400, "bits": 32, "lanes": 4}
        }


UDMA_I2C_REG_SCIF_ETU
^^^^^^^^^^^^^^^^^^^^^

`Address: 0x50109000 + 0x38 = 0x50109038`


    .. wavedrom::
        :caption: UDMA_I2C_REG_SCIF_ETU

        {
            "reg": [
                {"name": "r_scif_etu",  "bits": 16},
                {"name": "r_scif_err_irq_en",  "bits": 1},
                {"name": "r_scif_rx_irq_en",  "bits": 1},
                {"bits": 14}
            ], "config": {"hspace": 400, "bits": 32, "lanes": 4 }, "options": {"hspace": 400, "bits": 32, "lanes": 4}
        }


+--------+-------------------+-------------------+
| Field  | Name              | Description       |
+========+===================+===================+
| [15:0] | R_SCIF_ETU        | r_scif_etu        |
+--------+-------------------+-------------------+
| [16]   | R_SCIF_ERR_IRQ_EN | r_scif_err_irq_en |
+--------+-------------------+-------------------+
| [17]   | R_SCIF_RX_IRQ_EN  | r_scif_rx_irq_en  |
+--------+-------------------+-------------------+

UDMA_I2C_REG_AU_CFG
^^^^^^^^^^^^^^^^^^^

`Address: 0x50109000 + 0x3c = 0x5010903c`


    .. wavedrom::
        :caption: UDMA_I2C_REG_AU_CFG

        {
            "reg": [
                {"name": "r_au_use_signed",  "bits": 1},
                {"name": "r_au_bypass",  "bits": 1},
                {"name": "r_au_mode",  "bits": 4},
                {"name": "r_au_shift",  "bits": 5},
                {"bits": 21}
            ], "config": {"hspace": 400, "bits": 32, "lanes": 4 }, "options": {"hspace": 400, "bits": 32, "lanes": 4}
        }


+--------+-----------------+-----------------+
| Field  | Name            | Description     |
+========+=================+=================+
| [0]    | R_AU_USE_SIGNED | r_au_use_signed |
+--------+-----------------+-----------------+
| [1]    | R_AU_BYPASS     | r_au_bypass     |
+--------+-----------------+-----------------+
| [5:2]  | R_AU_MODE       | r_au_mode       |
+--------+-----------------+-----------------+
| [10:6] | R_AU_SHIFT      | r_au_shift      |
+--------+-----------------+-----------------+

UDMA_I2C_REG_AU_REG0
^^^^^^^^^^^^^^^^^^^^

`Address: 0x50109000 + 0x40 = 0x50109040`


    .. wavedrom::
        :caption: UDMA_I2C_REG_AU_REG0

        {
            "reg": [
                {"name": "r_au_reg0",  "bits": 32}
            ], "config": {"hspace": 400, "bits": 32, "lanes": 1 }, "options": {"hspace": 400, "bits": 32, "lanes": 1}
        }


+--------+-----------+-------------+
| Field  | Name      | Description |
+========+===========+=============+
| [31:0] | R_AU_REG0 | r_au_reg0   |
+--------+-----------+-------------+

UDMA_I2C_REG_AU_REG1
^^^^^^^^^^^^^^^^^^^^

`Address: 0x50109000 + 0x44 = 0x50109044`


    .. wavedrom::
        :caption: UDMA_I2C_REG_AU_REG1

        {
            "reg": [
                {"name": "r_au_reg1",  "bits": 32}
            ], "config": {"hspace": 400, "bits": 32, "lanes": 1 }, "options": {"hspace": 400, "bits": 32, "lanes": 1}
        }


+--------+-----------+-------------+
| Field  | Name      | Description |
+========+===========+=============+
| [31:0] | R_AU_REG1 | r_au_reg1   |
+--------+-----------+-------------+

UDMA_I2C_REG_BINCU_TH
^^^^^^^^^^^^^^^^^^^^^

`Address: 0x50109000 + 0x48 = 0x50109048`


    .. wavedrom::
        :caption: UDMA_I2C_REG_BINCU_TH

        {
            "reg": [
                {"name": "r_bincu_threshold",  "bits": 32}
            ], "config": {"hspace": 400, "bits": 32, "lanes": 1 }, "options": {"hspace": 400, "bits": 32, "lanes": 1}
        }


+--------+-------------------+-------------------+
| Field  | Name              | Description       |
+========+===================+===================+
| [31:0] | R_BINCU_THRESHOLD | r_bincu_threshold |
+--------+-------------------+-------------------+

UDMA_I2C_REG_BINCU_CNT
^^^^^^^^^^^^^^^^^^^^^^

`Address: 0x50109000 + 0x4c = 0x5010904c`


    .. wavedrom::
        :caption: UDMA_I2C_REG_BINCU_CNT

        {
            "reg": [
                {"name": "r_bincu_counter",  "bits": 15},
                {"name": "r_bincu_en_counter",  "bits": 1},
                {"bits": 16}
            ], "config": {"hspace": 400, "bits": 32, "lanes": 4 }, "options": {"hspace": 400, "bits": 32, "lanes": 4}
        }


+--------+--------------------+--------------------+
| Field  | Name               | Description        |
+========+====================+====================+
| [14:0] | R_BINCU_COUNTER    | r_bincu_counter    |
+--------+--------------------+--------------------+
| [15]   | R_BINCU_EN_COUNTER | r_bincu_en_counter |
+--------+--------------------+--------------------+

UDMA_I2C_REG_BINCU_SETUP
^^^^^^^^^^^^^^^^^^^^^^^^

`Address: 0x50109000 + 0x50 = 0x50109050`


    .. wavedrom::
        :caption: UDMA_I2C_REG_BINCU_SETUP

        {
            "reg": [
                {"name": "r_bincu_datasize",  "bits": 2},
                {"bits": 30}
            ], "config": {"hspace": 400, "bits": 32, "lanes": 4 }, "options": {"hspace": 400, "bits": 32, "lanes": 4}
        }


+-------+------------------+------------------+
| Field | Name             | Description      |
+=======+==================+==================+
| [1:0] | R_BINCU_DATASIZE | r_bincu_datasize |
+-------+------------------+------------------+

UDMA_I2C_REG_BINCU_VAL
^^^^^^^^^^^^^^^^^^^^^^

`Address: 0x50109000 + 0x54 = 0x50109054`


    .. wavedrom::
        :caption: UDMA_I2C_REG_BINCU_VAL

        {
            "reg": [
                {"name": "reg_bincu_val", "bits": 1},
                {"bits": 31},
            ], "config": {"hspace": 400, "bits": 32, "lanes": 4 }, "options": {"hspace": 400, "bits": 32, "lanes": 4}
        }


UDMA_I2C_REG_FILT
^^^^^^^^^^^^^^^^^

`Address: 0x50109000 + 0x58 = 0x50109058`


    .. wavedrom::
        :caption: UDMA_I2C_REG_FILT

        {
            "reg": [
                {"name": "r_filter_mode",  "bits": 4},
                {"bits": 28}
            ], "config": {"hspace": 400, "bits": 32, "lanes": 4 }, "options": {"hspace": 400, "bits": 32, "lanes": 4}
        }


+-------+---------------+---------------+
| Field | Name          | Description   |
+=======+===============+===============+
| [3:0] | R_FILTER_MODE | r_filter_mode |
+-------+---------------+---------------+

UDMA_I2C_REG_FILT_CMD
^^^^^^^^^^^^^^^^^^^^^

`Address: 0x50109000 + 0x5c = 0x5010905c`


    .. wavedrom::
        :caption: UDMA_I2C_REG_FILT_CMD

        {
            "reg": [
                {"name": "reg_filt_cmd", "bits": 1},
                {"bits": 31},
            ], "config": {"hspace": 400, "bits": 32, "lanes": 4 }, "options": {"hspace": 400, "bits": 32, "lanes": 4}
        }


UDMA_I2C_REG_STATUS
^^^^^^^^^^^^^^^^^^^

`Address: 0x50109000 + 0x60 = 0x50109060`


    .. wavedrom::
        :caption: UDMA_I2C_REG_STATUS

        {
            "reg": [
                {"name": "r_filter_done",  "bits": 32}
            ], "config": {"hspace": 400, "bits": 32, "lanes": 1 }, "options": {"hspace": 400, "bits": 32, "lanes": 1}
        }


+--------+---------------+---------------+
| Field  | Name          | Description   |
+========+===============+===============+
| [31:0] | R_FILTER_DONE | r_filter_done |
+--------+---------------+---------------+

