UDMA_UART_6
===========

Register Listing for UDMA_UART_6
--------------------------------

+----------------------------------------------------------------+------------------------------------------------+
| Register                                                       | Address                                        |
+================================================================+================================================+
| :ref:`UDMA_UART_6_REG_RX_SADDR <UDMA_UART_6_REG_RX_SADDR>`     | :ref:`0x5010d000 <UDMA_UART_6_REG_RX_SADDR>`   |
+----------------------------------------------------------------+------------------------------------------------+
| :ref:`UDMA_UART_6_REG_RX_SIZE <UDMA_UART_6_REG_RX_SIZE>`       | :ref:`0x5010d004 <UDMA_UART_6_REG_RX_SIZE>`    |
+----------------------------------------------------------------+------------------------------------------------+
| :ref:`UDMA_UART_6_REG_RX_CFG <UDMA_UART_6_REG_RX_CFG>`         | :ref:`0x5010d008 <UDMA_UART_6_REG_RX_CFG>`     |
+----------------------------------------------------------------+------------------------------------------------+
| :ref:`UDMA_UART_6_REG_RX_INTCFG <UDMA_UART_6_REG_RX_INTCFG>`   | :ref:`0x5010d00c <UDMA_UART_6_REG_RX_INTCFG>`  |
+----------------------------------------------------------------+------------------------------------------------+
| :ref:`UDMA_UART_6_REG_TX_SADDR <UDMA_UART_6_REG_TX_SADDR>`     | :ref:`0x5010d010 <UDMA_UART_6_REG_TX_SADDR>`   |
+----------------------------------------------------------------+------------------------------------------------+
| :ref:`UDMA_UART_6_REG_TX_SIZE <UDMA_UART_6_REG_TX_SIZE>`       | :ref:`0x5010d014 <UDMA_UART_6_REG_TX_SIZE>`    |
+----------------------------------------------------------------+------------------------------------------------+
| :ref:`UDMA_UART_6_REG_TX_CFG <UDMA_UART_6_REG_TX_CFG>`         | :ref:`0x5010d018 <UDMA_UART_6_REG_TX_CFG>`     |
+----------------------------------------------------------------+------------------------------------------------+
| :ref:`UDMA_UART_6_REG_TX_INTCFG <UDMA_UART_6_REG_TX_INTCFG>`   | :ref:`0x5010d01c <UDMA_UART_6_REG_TX_INTCFG>`  |
+----------------------------------------------------------------+------------------------------------------------+
| :ref:`UDMA_UART_6_REG_STATUS <UDMA_UART_6_REG_STATUS>`         | :ref:`0x5010d020 <UDMA_UART_6_REG_STATUS>`     |
+----------------------------------------------------------------+------------------------------------------------+
| :ref:`UDMA_UART_6_REG_UART_SETUP <UDMA_UART_6_REG_UART_SETUP>` | :ref:`0x5010d024 <UDMA_UART_6_REG_UART_SETUP>` |
+----------------------------------------------------------------+------------------------------------------------+
| :ref:`UDMA_UART_6_REG_ERROR <UDMA_UART_6_REG_ERROR>`           | :ref:`0x5010d028 <UDMA_UART_6_REG_ERROR>`      |
+----------------------------------------------------------------+------------------------------------------------+
| :ref:`UDMA_UART_6_REG_IRQ_EN <UDMA_UART_6_REG_IRQ_EN>`         | :ref:`0x5010d02c <UDMA_UART_6_REG_IRQ_EN>`     |
+----------------------------------------------------------------+------------------------------------------------+
| :ref:`UDMA_UART_6_REG_VALID <UDMA_UART_6_REG_VALID>`           | :ref:`0x5010d030 <UDMA_UART_6_REG_VALID>`      |
+----------------------------------------------------------------+------------------------------------------------+
| :ref:`UDMA_UART_6_REG_DATA <UDMA_UART_6_REG_DATA>`             | :ref:`0x5010d034 <UDMA_UART_6_REG_DATA>`       |
+----------------------------------------------------------------+------------------------------------------------+

UDMA_UART_6_REG_RX_SADDR
^^^^^^^^^^^^^^^^^^^^^^^^

`Address: 0x5010d000 + 0x0 = 0x5010d000`


    .. wavedrom::
        :caption: UDMA_UART_6_REG_RX_SADDR

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

UDMA_UART_6_REG_RX_SIZE
^^^^^^^^^^^^^^^^^^^^^^^

`Address: 0x5010d000 + 0x4 = 0x5010d004`


    .. wavedrom::
        :caption: UDMA_UART_6_REG_RX_SIZE

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

UDMA_UART_6_REG_RX_CFG
^^^^^^^^^^^^^^^^^^^^^^

`Address: 0x5010d000 + 0x8 = 0x5010d008`


    .. wavedrom::
        :caption: UDMA_UART_6_REG_RX_CFG

        {
            "reg": [
                {"name": "r_rx_clr",  "bits": 1},
                {"name": "r_rx_en",  "bits": 1},
                {"name": "r_rx_continuous",  "bits": 1},
                {"bits": 29}
            ], "config": {"hspace": 400, "bits": 32, "lanes": 4 }, "options": {"hspace": 400, "bits": 32, "lanes": 4}
        }


+-------+-----------------+-----------------+
| Field | Name            | Description     |
+=======+=================+=================+
| [0]   | R_RX_CLR        | r_rx_clr        |
+-------+-----------------+-----------------+
| [1]   | R_RX_EN         | r_rx_en         |
+-------+-----------------+-----------------+
| [2]   | R_RX_CONTINUOUS | r_rx_continuous |
+-------+-----------------+-----------------+

UDMA_UART_6_REG_RX_INTCFG
^^^^^^^^^^^^^^^^^^^^^^^^^

`Address: 0x5010d000 + 0xc = 0x5010d00c`


    .. wavedrom::
        :caption: UDMA_UART_6_REG_RX_INTCFG

        {
            "reg": [
                {"name": "reg_rx_intcfg", "bits": 1},
                {"bits": 31},
            ], "config": {"hspace": 400, "bits": 32, "lanes": 4 }, "options": {"hspace": 400, "bits": 32, "lanes": 4}
        }


UDMA_UART_6_REG_TX_SADDR
^^^^^^^^^^^^^^^^^^^^^^^^

`Address: 0x5010d000 + 0x10 = 0x5010d010`


    .. wavedrom::
        :caption: UDMA_UART_6_REG_TX_SADDR

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

UDMA_UART_6_REG_TX_SIZE
^^^^^^^^^^^^^^^^^^^^^^^

`Address: 0x5010d000 + 0x14 = 0x5010d014`


    .. wavedrom::
        :caption: UDMA_UART_6_REG_TX_SIZE

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

UDMA_UART_6_REG_TX_CFG
^^^^^^^^^^^^^^^^^^^^^^

`Address: 0x5010d000 + 0x18 = 0x5010d018`


    .. wavedrom::
        :caption: UDMA_UART_6_REG_TX_CFG

        {
            "reg": [
                {"name": "r_tx_clr",  "bits": 1},
                {"name": "r_tx_en",  "bits": 1},
                {"name": "r_tx_continuous",  "bits": 1},
                {"bits": 29}
            ], "config": {"hspace": 400, "bits": 32, "lanes": 4 }, "options": {"hspace": 400, "bits": 32, "lanes": 4}
        }


+-------+-----------------+-----------------+
| Field | Name            | Description     |
+=======+=================+=================+
| [0]   | R_TX_CLR        | r_tx_clr        |
+-------+-----------------+-----------------+
| [1]   | R_TX_EN         | r_tx_en         |
+-------+-----------------+-----------------+
| [2]   | R_TX_CONTINUOUS | r_tx_continuous |
+-------+-----------------+-----------------+

UDMA_UART_6_REG_TX_INTCFG
^^^^^^^^^^^^^^^^^^^^^^^^^

`Address: 0x5010d000 + 0x1c = 0x5010d01c`


    .. wavedrom::
        :caption: UDMA_UART_6_REG_TX_INTCFG

        {
            "reg": [
                {"name": "reg_tx_intcfg", "bits": 1},
                {"bits": 31},
            ], "config": {"hspace": 400, "bits": 32, "lanes": 4 }, "options": {"hspace": 400, "bits": 32, "lanes": 4}
        }


UDMA_UART_6_REG_STATUS
^^^^^^^^^^^^^^^^^^^^^^

`Address: 0x5010d000 + 0x20 = 0x5010d020`


    .. wavedrom::
        :caption: UDMA_UART_6_REG_STATUS

        {
            "reg": [
                {"name": "reg_status", "bits": 1},
                {"bits": 31},
            ], "config": {"hspace": 400, "bits": 32, "lanes": 4 }, "options": {"hspace": 400, "bits": 32, "lanes": 4}
        }


UDMA_UART_6_REG_UART_SETUP
^^^^^^^^^^^^^^^^^^^^^^^^^^

`Address: 0x5010d000 + 0x24 = 0x5010d024`


    .. wavedrom::
        :caption: UDMA_UART_6_REG_UART_SETUP

        {
            "reg": [
                {"name": "r_uart_div",  "bits": 16},
                {"name": "r_uart_en_rx",  "bits": 1},
                {"name": "r_uart_en_tx",  "bits": 1},
                {"name": "r_uart_rx_clean_fifo",  "bits": 1},
                {"name": "r_uart_rx_polling_en",  "bits": 1},
                {"name": "r_uart_stop_bits",  "bits": 1},
                {"name": "r_uart_bits",  "bits": 2},
                {"name": "r_uart_parity_en",  "bits": 1},
                {"bits": 8}
            ], "config": {"hspace": 400, "bits": 32, "lanes": 4 }, "options": {"hspace": 400, "bits": 32, "lanes": 4}
        }


+---------+----------------------+----------------------+
| Field   | Name                 | Description          |
+=========+======================+======================+
| [15:0]  | R_UART_DIV           | r_uart_div           |
+---------+----------------------+----------------------+
| [16]    | R_UART_EN_RX         | r_uart_en_rx         |
+---------+----------------------+----------------------+
| [17]    | R_UART_EN_TX         | r_uart_en_tx         |
+---------+----------------------+----------------------+
| [18]    | R_UART_RX_CLEAN_FIFO | r_uart_rx_clean_fifo |
+---------+----------------------+----------------------+
| [19]    | R_UART_RX_POLLING_EN | r_uart_rx_polling_en |
+---------+----------------------+----------------------+
| [20]    | R_UART_STOP_BITS     | r_uart_stop_bits     |
+---------+----------------------+----------------------+
| [22:21] | R_UART_BITS          | r_uart_bits          |
+---------+----------------------+----------------------+
| [23]    | R_UART_PARITY_EN     | r_uart_parity_en     |
+---------+----------------------+----------------------+

UDMA_UART_6_REG_ERROR
^^^^^^^^^^^^^^^^^^^^^

`Address: 0x5010d000 + 0x28 = 0x5010d028`


    .. wavedrom::
        :caption: UDMA_UART_6_REG_ERROR

        {
            "reg": [
                {"name": "reg_error", "bits": 1},
                {"bits": 31},
            ], "config": {"hspace": 400, "bits": 32, "lanes": 4 }, "options": {"hspace": 400, "bits": 32, "lanes": 4}
        }


UDMA_UART_6_REG_IRQ_EN
^^^^^^^^^^^^^^^^^^^^^^

`Address: 0x5010d000 + 0x2c = 0x5010d02c`


    .. wavedrom::
        :caption: UDMA_UART_6_REG_IRQ_EN

        {
            "reg": [
                {"name": "r_uart_err_irq_en",  "bits": 1},
                {"name": "r_uart_rx_irq_en",  "bits": 1},
                {"bits": 30}
            ], "config": {"hspace": 400, "bits": 32, "lanes": 4 }, "options": {"hspace": 400, "bits": 32, "lanes": 4}
        }


+-------+-------------------+-------------------+
| Field | Name              | Description       |
+=======+===================+===================+
| [0]   | R_UART_ERR_IRQ_EN | r_uart_err_irq_en |
+-------+-------------------+-------------------+
| [1]   | R_UART_RX_IRQ_EN  | r_uart_rx_irq_en  |
+-------+-------------------+-------------------+

UDMA_UART_6_REG_VALID
^^^^^^^^^^^^^^^^^^^^^

`Address: 0x5010d000 + 0x30 = 0x5010d030`


    .. wavedrom::
        :caption: UDMA_UART_6_REG_VALID

        {
            "reg": [
                {"name": "reg_valid", "bits": 1},
                {"bits": 31},
            ], "config": {"hspace": 400, "bits": 32, "lanes": 4 }, "options": {"hspace": 400, "bits": 32, "lanes": 4}
        }


UDMA_UART_6_REG_DATA
^^^^^^^^^^^^^^^^^^^^

`Address: 0x5010d000 + 0x34 = 0x5010d034`


    .. wavedrom::
        :caption: UDMA_UART_6_REG_DATA

        {
            "reg": [
                {"name": "reg_data", "bits": 1},
                {"bits": 31},
            ], "config": {"hspace": 400, "bits": 32, "lanes": 4 }, "options": {"hspace": 400, "bits": 32, "lanes": 4}
        }


