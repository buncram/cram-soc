Search.setIndex({"docnames": ["coreuser", "cpu", "csrtest", "index", "interrupts", "irqarray0", "irqarray1", "irqarray10", "irqarray11", "irqarray12", "irqarray13", "irqarray14", "irqarray15", "irqarray16", "irqarray17", "irqarray18", "irqarray19", "irqarray2", "irqarray3", "irqarray4", "irqarray5", "irqarray6", "irqarray7", "irqarray8", "irqarray9", "mailbox", "resetvalue", "ticktimer"], "filenames": ["coreuser.rst", "cpu.rst", "csrtest.rst", "index.rst", "interrupts.rst", "irqarray0.rst", "irqarray1.rst", "irqarray10.rst", "irqarray11.rst", "irqarray12.rst", "irqarray13.rst", "irqarray14.rst", "irqarray15.rst", "irqarray16.rst", "irqarray17.rst", "irqarray18.rst", "irqarray19.rst", "irqarray2.rst", "irqarray3.rst", "irqarray4.rst", "irqarray5.rst", "irqarray6.rst", "irqarray7.rst", "irqarray8.rst", "irqarray9.rst", "mailbox.rst", "resetvalue.rst", "ticktimer.rst"], "titles": ["COREUSER", "CPU", "CSRTEST", "Documentation for Cramium SoC (RISC-V Core Complex)", "Interrupt Controller", "IRQARRAY0", "IRQARRAY1", "IRQARRAY10", "IRQARRAY11", "IRQARRAY12", "IRQARRAY13", "IRQARRAY14", "IRQARRAY15", "IRQARRAY16", "IRQARRAY17", "IRQARRAY18", "IRQARRAY19", "IRQARRAY2", "IRQARRAY3", "IRQARRAY4", "IRQARRAY5", "IRQARRAY6", "IRQARRAY7", "IRQARRAY8", "IRQARRAY9", "MAILBOX", "RESETVALUE", "TICKTIMER"], "terms": {"i": [0, 1, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26, 27], "hardwar": [0, 25, 27], "signal": [0, 1, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 27], "indic": [0, 25], "code": [0, 25], "execut": 0, "highli": 0, "trust": 0, "piec": 0, "thi": [0, 1, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26, 27], "determin": [0, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24], "examin": 0, "configur": 0, "combin": 0, "satp": [0, 1], "": [0, 25], "asid": 0, "ppn": 0, "valu": [0, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 26, 27], "allow": [0, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25], "o": [0, 1, 27], "target": [0, 27], "certain": 0, "virtual": [0, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24], "memori": [0, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25], "space": [0, 25], "more": [0, 25], "than": [0, 27], "other": [0, 25, 27], "can": [0, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26, 27], "onli": [0, 1], "comput": 0, "when": [0, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 27], "risc": 0, "v": 0, "core": [0, 1], "sv32": 0, "mode": 0, "ha": [0, 4, 25, 27], "been": [0, 25], "enabl": [0, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26, 27], "specifi": [0, 25, 26], "two": [0, 25], "window": 0, "ar": [0, 1, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26], "provid": [0, 1, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25], "b": 0, "The": [0, 1, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26, 27], "independ": 0, "OR": 0, "d": [0, 1], "togeth": 0, "should": [0, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25], "non": [0, 1], "overlap": 0, "If": [0, 27], "thei": [0, 25], "poorli": 0, "behavior": 0, "guarante": [0, 25], "intent": 0, "have": [0, 25, 27], "so": [0, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 27], "process": [0, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25], "rather": 0, "design": 0, "alloc": 0, "all": [0, 1, 25, 27], "within": [0, 25], "singl": [0, 25], "rang": 0, "protect": 0, "altern": 0, "scratch": 0, "re": [0, 25], "organ": 0, "shuffl": 0, "around": [0, 25, 27], "higher": 0, "level": [0, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 27], "cycl": [0, 25, 27], "precis": 0, "assert": [0, 25], "roughli": 0, "2": [0, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25], "after": [0, 25], "updat": 0, "furthermor": 0, "field": [0, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 27], "an": [0, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 27], "advisori": [0, 25], "isn": 0, "t": 0, "us": [0, 1, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25], "cpu": [0, 3, 4, 27], "enforc": [0, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24], "page": [0, 3, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24], "access": 0, "you": [0, 4], "think": 0, "kernel": 0, "control": [0, 3, 25], "context": 0, "we": [0, 25, 26], "swap": 0, "fortun": 0, "ani": [0, 1, 25], "follow": [0, 1, 4, 25], "sfenc": 0, "instruct": [0, 1], "invalid": 0, "tlb": [0, 1], "map": 0, "etc": 0, "which": [0, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 27], "give": [0, 27], "time": [0, 25, 27], "propag": 0, "through": [0, 1, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25], "pipelin": [0, 1, 25], "thu": [0, 25], "practic": [0, 25], "first": [0, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25], "user": 0, "run": [0, 1, 25, 27], "set": [0, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 27], "properli": 0, "howev": [0, 25], "from": [0, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26, 27], "secur": 0, "audit": 0, "perspect": 0, "import": 0, "keep": [0, 27], "mind": 0, "race": [0, 25], "condit": [0, 25], "between": [0, 25, 27], "address": [0, 2, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26, 27], "0x58000000": 0, "0x58000004": 0, "0x58000008": 0, "0x5800000c": 0, "0x58000010": 0, "0x58000014": 0, "0x58000018": 0, "0x5800001c": 0, "0x58000020": 0, "0x0": [0, 2, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26, 27], "name": [0, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 27], "descript": [0, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 27], "8": [0, 1, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 27], "0": [0, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 27], "write": [0, 2, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 27], "commit": 0, "9": [0, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25], "1": [0, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 27], "0x4": [0, 2, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 27], "read": [0, 1, 2, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26, 27], "back": [0, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26], "0x8": [0, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 27], "correspond": [0, 25, 27], "get_asid_addr": 0, "mean": [0, 25], "0xc": [0, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 27], "appli": 0, "clear": [0, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 27], "alwai": [0, 25, 27], "valid": [0, 25], "default": 0, "requir": [0, 25, 27], "ppn_a": 0, "3": [0, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25], "ppn_b": 0, "0x10": [0, 25, 27], "bit": [0, 1, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26, 27], "prevent": [0, 25], "further": [0, 25], "statu": [0, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24], "revers": 0, "system": [0, 4, 25, 27], "reset": [0, 26, 27], "0x14": [0, 25, 27], "21": [0, 4, 25], "match": 0, "lower": 0, "bound": 0, "greater": 0, "equal": [0, 27], "0x18": [0, 25, 27], "upper": 0, "less": [0, 27], "255": 0, "would": [0, 25, 26, 27], "everyth": 0, "result": [0, 25], "256": 0, "total": 0, "locat": 0, "0x1c": [0, 25, 27], "0x20": 0, "vexriscv": 1, "bu": 1, "interfac": [1, 25], "64": [1, 27], "axi": 1, "4": [1, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24], "cach": 1, "data": [1, 2, 27], "crossbar": 1, "0x60000000": 1, "7fffffff": 1, "32": [1, 25, 27], "r": 1, "w": 1, "0x40000000": 1, "4fffffff": 1, "lite": 1, "peripher": 1, "uncach": 1, "buss": 1, "aclk": [1, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24], "speed": [1, 27], "wfi": 1, "broken": 1, "out": [1, 25, 27], "wfi_act": 1, "coreus": [1, 3], "interpret": 1, "itself": 1, "contain": [1, 27], "featur": 1, "simpl": 1, "order": [1, 25], "rv32": 1, "imac": 1, "static": 1, "branch": 1, "predict": 1, "4k": 1, "wai": [1, 25], "mmu": 1, "entri": 1, "ae": 1, "extens": 1, "region": 1, "5fffffff": 1, "0xa0000000": 1, "ffffffff": 1, "rout": [1, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 27], "intern": 1, "block": [1, 25, 27], "0x58001000": 2, "0x58001004": 2, "test": 2, "here": [2, 25], "interrupt": [3, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 27], "csrtest": 3, "irqarray0": [3, 4], "irqarray1": [3, 4], "irqarray10": [3, 4], "irqarray11": [3, 4], "irqarray12": [3, 4], "irqarray13": [3, 4], "irqarray14": [3, 4], "irqarray15": [3, 4], "irqarray16": [3, 4], "irqarray17": [3, 4], "irqarray18": [3, 4], "irqarray19": [3, 4], "irqarray2": [3, 4], "irqarray3": [3, 4], "irqarray4": [3, 4], "irqarray5": [3, 4], "irqarray6": [3, 4], "irqarray7": [3, 4], "irqarray8": [3, 4], "irqarray9": [3, 4], "mailbox": [3, 4], "resetvalu": 3, "ticktim": [3, 4], "index": 3, "search": 3, "devic": [4, 25], "eventmanag": 4, "base": 4, "individu": [4, 27], "modul": 4, "gener": [4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25], "event": [4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26, 27], "wire": [4, 25], "central": 4, "occur": [4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 27], "look": 4, "number": [4, 25], "up": [4, 25, 27], "specif": [4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 27], "tabl": 4, "call": [4, 25], "relev": 4, "10": [4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25], "11": [4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24], "12": [4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24], "13": [4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24], "14": [4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24], "15": [4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25], "16": [4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25], "17": [4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24], "18": [4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24], "19": [4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25], "5": [4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24], "6": [4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24], "7": [4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24], "20": [4, 25], "irqarrai": [5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24], "larg": [5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24], "bank": [5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24], "soc": [5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24], "integr": [5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24], "It": [5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25], "differ": [5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24], "e": [5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24], "g": [5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24], "nvic": [5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24], "clint": [5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24], "structur": [5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24], "along": [5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24], "boundari": [5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24], "handler": [5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25], "csr": [5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25], "own": [5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24], "instead": [5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 27], "bounc": [5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24], "common": [5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25], "forc": [5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24], "inter": [5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24], "messag": [5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25], "final": [5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24], "destin": [5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24], "incom": [5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25], "assum": [5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24], "synchron": [5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 27], "prioriti": [5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 27], "entir": [5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24], "softwar": [5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24], "must": [5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25], "pend": [5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25], "decid": [5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25], "ones": [5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24], "handl": [5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25], "eventsourc": [5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24], "eventsourceflex": [5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24], "puls": [5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 27], "well": [5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24], "trigger": [5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 27], "latch": [5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26], "goe": [5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24], "high": [5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 27], "stai": [5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25], "until": [5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 27], "take": [5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 27], "preced": [5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24], "over": [5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 27], "sourc": [5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 27], "prior": [5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24], "again": [5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24], "reflect": [5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24], "instantan": [5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24], "A": [5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25], "separ": [5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24], "input": [5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24], "line": [5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25], "induc": [5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24], "soft": [5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24], "0x58002000": 5, "0x58002004": 5, "0x58002008": 5, "0x5800200c": 5, "persist": [5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24], "wa": [5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 27], "respons": [5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25], "repeat": [5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24], "without": [5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25], "still": [5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25], "function": [5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 27], "source19": [5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24], "form": [5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25], "source0": [5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24], "source1": [5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24], "source2": [5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24], "source3": [5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24], "source4": [5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24], "source5": [5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24], "source6": [5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24], "source7": [5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24], "source8": [5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24], "source9": [5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24], "source10": [5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24], "source11": [5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24], "source12": [5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24], "source13": [5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24], "source14": [5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24], "source15": [5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24], "source16": [5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24], "source17": [5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24], "source18": [5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24], "0x58003000": 6, "0x58003004": 6, "0x58003008": 6, "0x5800300c": 6, "0x58004000": 7, "0x58004004": 7, "0x58004008": 7, "0x5800400c": 7, "0x58005000": 8, "0x58005004": 8, "0x58005008": 8, "0x5800500c": 8, "0x58006000": 9, "0x58006004": 9, "0x58006008": 9, "0x5800600c": 9, "0x58007000": 10, "0x58007004": 10, "0x58007008": 10, "0x5800700c": 10, "0x58008000": 11, "0x58008004": 11, "0x58008008": 11, "0x5800800c": 11, "0x58009000": 12, "0x58009004": 12, "0x58009008": 12, "0x5800900c": 12, "0x5800a000": 13, "0x5800a004": 13, "0x5800a008": 13, "0x5800a00c": 13, "0x5800b000": 14, "0x5800b004": 14, "0x5800b008": 14, "0x5800b00c": 14, "0x5800c000": 15, "0x5800c004": 15, "0x5800c008": 15, "0x5800c00c": 15, "0x5800d000": 16, "0x5800d004": 16, "0x5800d008": 16, "0x5800d00c": 16, "0x5800e000": 17, "0x5800e004": 17, "0x5800e008": 17, "0x5800e00c": 17, "0x5800f000": 18, "0x5800f004": 18, "0x5800f008": 18, "0x5800f00c": 18, "0x58010000": 19, "0x58010004": 19, "0x58010008": 19, "0x5801000c": 19, "0x58011000": 20, "0x58011004": 20, "0x58011008": 20, "0x5801100c": 20, "0x58012000": 21, "0x58012004": 21, "0x58012008": 21, "0x5801200c": 21, "0x58013000": 22, "0x58013004": 22, "0x58013008": 22, "0x5801300c": 22, "0x58014000": 23, "0x58014004": 23, "0x58014008": 23, "0x5801400c": 23, "0x58015000": 24, "0x58015004": 24, "0x58015008": 24, "0x5801500c": 24, "bi": 25, "direct": 25, "deliv": 25, "share": 25, "consist": 25, "packet": 25, "1024": 25, "word": 25, "long": 25, "where": [25, 26], "each": [25, 27], "length": 25, "both": 25, "consid": 25, "peer": 25, "initi": 25, "one": 25, "mac": 25, "app": 25, "fulli": 25, "implement": [25, 27], "manag": 25, "oper": 25, "just": 25, "mani": [25, 27], "commun": 25, "help": 25, "ground": 25, "framework": 25, "some": [25, 27], "detail": 25, "impact": 25, "especi": 25, "conflict": 25, "avoid": 25, "channel": 25, "dat": 25, "avail": 25, "readi": 25, "fifo": 25, "exclus": 25, "state": 25, "machin": 25, "host": 25, "awar": 25, "mainli": 25, "exist": 25, "overflow": [25, 27], "case": 25, "multipl": 25, "There": [25, 27], "addit": 25, "done": 25, "exactli": 25, "sender": 25, "finish": 25, "given": 25, "doe": 25, "need": 25, "busi": 25, "monitor": 25, "depth": 25, "send": 25, "four": 25, "abov": 25, "exampl": 25, "show": 25, "being": [25, 27], "transmit": 25, "extra": 25, "acknowledg": 25, "remain": [25, 27], "three": 25, "immedi": 25, "accept": 25, "could": [25, 26, 27], "come": [25, 26], "earli": 25, "simultan": 25, "last": 25, "coupl": 25, "later": 25, "sinc": [25, 27], "symmetr": 25, "across": 25, "recov": 25, "known": 25, "empti": 25, "idl": 25, "accomplish": 25, "cross": [25, 27], "w_abort": 25, "r_abort": 25, "either": [25, 26], "At": 25, "conclus": 25, "normal": 25, "In": 25, "diagram": 25, "w_": 25, "r_": 25, "issu": 25, "held": 25, "while": 25, "receiv": 25, "refus": 25, "render": 25, "r_abort_int": 25, "sticki": 25, "link": 25, "main": 25, "loop": 25, "irq": 25, "its": 25, "note": [25, 27], "written": 25, "progress": 25, "truli": 25, "interact": 25, "probabl": 25, "disabl": [25, 27], "point": 25, "fire": [25, 27], "insid": 25, "side": 25, "effect": [25, 27], "variabl": 25, "resum": 25, "now": [25, 27], "check": 25, "residu": 25, "clean": 25, "drop": 25, "return": 25, "abort_don": 25, "mai": 25, "mask": [25, 27], "poll": 25, "prefer": 25, "make": 25, "work": 25, "attempt": 25, "same": 25, "act": 25, "thing": 25, "request": [25, 27], "edg": 25, "rare": [25, 27], "perfect": 25, "transit": 25, "req": 25, "go": 25, "ack": 25, "semi": 25, "prepar": 25, "perhap": 25, "late": 25, "naiv": 25, "respond": 25, "ping": 25, "pong": 25, "forth": 25, "infinit": 25, "break": 25, "abort_ack": 25, "respect": 25, "perfectli": 25, "align": 25, "typic": 25, "shall": 25, "previous": 25, "storm": 25, "wrap": 25, "format": 25, "31": [25, 27], "30": 25, "sequenc": 25, "tag": 25, "exclud": 25, "encod": 25, "intend": 25, "ascrib": 25, "As": 25, "rpc": 25, "desir": 25, "subsequ": 25, "argument": 25, "definit": 25, "extend": 25, "sent": 25, "were": 25, "0x58016000": 25, "0x58016004": 25, "0x58016008": 25, "0x5801600c": 25, "0x58016010": 25, "0x58016014": 25, "0x58016018": 25, "0x5801601c": 25, "outgo": 25, "tx_err": 25, "rx_err": 25, "abort_init": 25, "error": 25, "current": [25, 27], "rx_word": 25, "tx_word": 25, "free": 25, "tx_avail": 25, "abort_in_progress": 25, "imperfect": 25, "new": [25, 27], "also": 25, "ignor": 25, "22": 25, "becaus": [25, 26, 27], "wrote": 25, "too": 25, "much": [25, 27], "23": 25, "underflow": 25, "full": 25, "load": 25, "captur": 26, "actual": 26, "present": 26, "reason": 26, "necessari": 26, "built": 26, "silicon": 26, "trim": 26, "program": 26, "via": 26, "reram": 26, "vector": 26, "confirm": 26, "fact": 26, "expect": 26, "default_valu": 26, "what": 26, "trimming_reset": 26, "trimming_reset_ena": 26, "0x58017000": 26, "pc": 26, "timer0": 27, "resolut": 27, "sysclk": 27, "veri": 27, "quickli": 27, "overhead": 27, "convert": 27, "usabl": 27, "count": 27, "off": 27, "paramet": 27, "divisor": 27, "1000": 27, "increment": 27, "tick": 27, "1m": 27, "2000": 27, "5m": 27, "self": 27, "substanti": 27, "area": 27, "save": 27, "hand": 27, "smarter": 27, "about": 27, "domain": 27, "right": 27, "chip": 27, "eaten": 27, "1100": 27, "clock": 27, "move": 27, "slightli": 27, "method": 27, "creat": 27, "lock": 27, "false_path": 27, "rule": 27, "datapath": 27, "place": 27, "get": 27, "distract": 27, "roll": 27, "292471208": 27, "68": 27, "year": 27, "xou": 27, "add": 27, "aid": 27, "server": 27, "elaps": 27, "activ": 27, "slight": 27, "slip": 27, "200n": 27, "befor": 27, "transfer": 27, "slower": 27, "rate": 27, "0x58018000": 27, "0x58018004": 27, "0x58018008": 27, "0x5801800c": 27, "0x58018010": 27, "0x58018014": 27, "0x58018018": 27, "0x5801801c": 27, "63": 27, "ticktimer_tim": 27, "ticktimer_msleep_target": 27, "raw": 27, "alarm": 27, "To": 27}, "objects": {}, "objtypes": {}, "objnames": {}, "titleterms": {"coreus": 0, "regist": [0, 2, 3, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26, 27], "list": [0, 2, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26, 27], "coreuser_set_asid": 0, "coreuser_get_asid_addr": 0, "coreuser_get_asid_valu": 0, "coreuser_control": 0, "coreuser_protect": 0, "coreuser_window_": 0, "coreuser_window_ah": 0, "coreuser_window_bl": 0, "coreuser_window_bh": 0, "cpu": [1, 25], "csrtest": 2, "csrtest_wtest": 2, "csrtest_rtest": 2, "document": 3, "cramium": 3, "soc": 3, "risc": 3, "v": 3, "core": 3, "complex": 3, "modul": 3, "group": 3, "indic": 3, "tabl": 3, "interrupt": 4, "control": 4, "assign": 4, "irqarray0": 5, "irqarray0_ev_soft": 5, "irqarray0_ev_statu": 5, "irqarray0_ev_pend": 5, "irqarray0_ev_en": 5, "irqarray1": 6, "irqarray1_ev_soft": 6, "irqarray1_ev_statu": 6, "irqarray1_ev_pend": 6, "irqarray1_ev_en": 6, "irqarray10": 7, "irqarray10_ev_soft": 7, "irqarray10_ev_statu": 7, "irqarray10_ev_pend": 7, "irqarray10_ev_en": 7, "irqarray11": 8, "irqarray11_ev_soft": 8, "irqarray11_ev_statu": 8, "irqarray11_ev_pend": 8, "irqarray11_ev_en": 8, "irqarray12": 9, "irqarray12_ev_soft": 9, "irqarray12_ev_statu": 9, "irqarray12_ev_pend": 9, "irqarray12_ev_en": 9, "irqarray13": 10, "irqarray13_ev_soft": 10, "irqarray13_ev_statu": 10, "irqarray13_ev_pend": 10, "irqarray13_ev_en": 10, "irqarray14": 11, "irqarray14_ev_soft": 11, "irqarray14_ev_statu": 11, "irqarray14_ev_pend": 11, "irqarray14_ev_en": 11, "irqarray15": 12, "irqarray15_ev_soft": 12, "irqarray15_ev_statu": 12, "irqarray15_ev_pend": 12, "irqarray15_ev_en": 12, "irqarray16": 13, "irqarray16_ev_soft": 13, "irqarray16_ev_statu": 13, "irqarray16_ev_pend": 13, "irqarray16_ev_en": 13, "irqarray17": 14, "irqarray17_ev_soft": 14, "irqarray17_ev_statu": 14, "irqarray17_ev_pend": 14, "irqarray17_ev_en": 14, "irqarray18": 15, "irqarray18_ev_soft": 15, "irqarray18_ev_statu": 15, "irqarray18_ev_pend": 15, "irqarray18_ev_en": 15, "irqarray19": 16, "irqarray19_ev_soft": 16, "irqarray19_ev_statu": 16, "irqarray19_ev_pend": 16, "irqarray19_ev_en": 16, "irqarray2": 17, "irqarray2_ev_soft": 17, "irqarray2_ev_statu": 17, "irqarray2_ev_pend": 17, "irqarray2_ev_en": 17, "irqarray3": 18, "irqarray3_ev_soft": 18, "irqarray3_ev_statu": 18, "irqarray3_ev_pend": 18, "irqarray3_ev_en": 18, "irqarray4": 19, "irqarray4_ev_soft": 19, "irqarray4_ev_statu": 19, "irqarray4_ev_pend": 19, "irqarray4_ev_en": 19, "irqarray5": 20, "irqarray5_ev_soft": 20, "irqarray5_ev_statu": 20, "irqarray5_ev_pend": 20, "irqarray5_ev_en": 20, "irqarray6": 21, "irqarray6_ev_soft": 21, "irqarray6_ev_statu": 21, "irqarray6_ev_pend": 21, "irqarray6_ev_en": 21, "irqarray7": 22, "irqarray7_ev_soft": 22, "irqarray7_ev_statu": 22, "irqarray7_ev_pend": 22, "irqarray7_ev_en": 22, "irqarray8": 23, "irqarray8_ev_soft": 23, "irqarray8_ev_statu": 23, "irqarray8_ev_pend": 23, "irqarray8_ev_en": 23, "irqarray9": 24, "irqarray9_ev_soft": 24, "irqarray9_ev_statu": 24, "irqarray9_ev_pend": 24, "irqarray9_ev_en": 24, "mailbox": 25, "an": 25, "inter": 25, "data": 25, "transfer": 25, "protocol": 25, "abort": 25, "applic": 25, "mailbox_wdata": 25, "mailbox_rdata": 25, "mailbox_ev_statu": 25, "mailbox_ev_pend": 25, "mailbox_ev_en": 25, "mailbox_statu": 25, "mailbox_control": 25, "mailbox_don": 25, "resetvalu": 26, "resetvalue_pc": 26, "ticktim": 27, "A": 27, "practic": 27, "systick": 27, "timer": 27, "configur": 27, "msleep": 27, "extens": 27, "ticktimer_control": 27, "ticktimer_time1": 27, "ticktimer_time0": 27, "ticktimer_msleep_target1": 27, "ticktimer_msleep_target0": 27, "ticktimer_ev_statu": 27, "ticktimer_ev_pend": 27, "ticktimer_ev_en": 27}, "envversion": {"sphinx.domains.c": 2, "sphinx.domains.changeset": 1, "sphinx.domains.citation": 1, "sphinx.domains.cpp": 8, "sphinx.domains.index": 1, "sphinx.domains.javascript": 2, "sphinx.domains.math": 2, "sphinx.domains.python": 3, "sphinx.domains.rst": 2, "sphinx.domains.std": 2, "sphinx": 57}, "alltitles": {"COREUSER": [[0, "coreuser"]], "Register Listing for COREUSER": [[0, "register-listing-for-coreuser"]], "COREUSER_SET_ASID": [[0, "coreuser-set-asid"]], "COREUSER_GET_ASID_ADDR": [[0, "coreuser-get-asid-addr"]], "COREUSER_GET_ASID_VALUE": [[0, "coreuser-get-asid-value"]], "COREUSER_CONTROL": [[0, "coreuser-control"]], "COREUSER_PROTECT": [[0, "coreuser-protect"]], "COREUSER_WINDOW_AL": [[0, "coreuser-window-al"]], "COREUSER_WINDOW_AH": [[0, "coreuser-window-ah"]], "COREUSER_WINDOW_BL": [[0, "coreuser-window-bl"]], "COREUSER_WINDOW_BH": [[0, "coreuser-window-bh"]], "CPU": [[1, "cpu"]], "CSRTEST": [[2, "csrtest"]], "Register Listing for CSRTEST": [[2, "register-listing-for-csrtest"]], "CSRTEST_WTEST": [[2, "csrtest-wtest"]], "CSRTEST_RTEST": [[2, "csrtest-rtest"]], "Documentation for Cramium SoC (RISC-V Core Complex)": [[3, "documentation-for-cramium-soc-risc-v-core-complex"]], "Modules": [[3, "modules"]], "Register Groups": [[3, "register-groups"]], "Indices and tables": [[3, "indices-and-tables"]], "Interrupt Controller": [[4, "interrupt-controller"]], "Assigned Interrupts": [[4, "assigned-interrupts"]], "IRQARRAY0": [[5, "irqarray0"]], "Register Listing for IRQARRAY0": [[5, "register-listing-for-irqarray0"]], "IRQARRAY0_EV_SOFT": [[5, "irqarray0-ev-soft"]], "IRQARRAY0_EV_STATUS": [[5, "irqarray0-ev-status"]], "IRQARRAY0_EV_PENDING": [[5, "irqarray0-ev-pending"]], "IRQARRAY0_EV_ENABLE": [[5, "irqarray0-ev-enable"]], "IRQARRAY1": [[6, "irqarray1"]], "Register Listing for IRQARRAY1": [[6, "register-listing-for-irqarray1"]], "IRQARRAY1_EV_SOFT": [[6, "irqarray1-ev-soft"]], "IRQARRAY1_EV_STATUS": [[6, "irqarray1-ev-status"]], "IRQARRAY1_EV_PENDING": [[6, "irqarray1-ev-pending"]], "IRQARRAY1_EV_ENABLE": [[6, "irqarray1-ev-enable"]], "IRQARRAY10": [[7, "irqarray10"]], "Register Listing for IRQARRAY10": [[7, "register-listing-for-irqarray10"]], "IRQARRAY10_EV_SOFT": [[7, "irqarray10-ev-soft"]], "IRQARRAY10_EV_STATUS": [[7, "irqarray10-ev-status"]], "IRQARRAY10_EV_PENDING": [[7, "irqarray10-ev-pending"]], "IRQARRAY10_EV_ENABLE": [[7, "irqarray10-ev-enable"]], "IRQARRAY11": [[8, "irqarray11"]], "Register Listing for IRQARRAY11": [[8, "register-listing-for-irqarray11"]], "IRQARRAY11_EV_SOFT": [[8, "irqarray11-ev-soft"]], "IRQARRAY11_EV_STATUS": [[8, "irqarray11-ev-status"]], "IRQARRAY11_EV_PENDING": [[8, "irqarray11-ev-pending"]], "IRQARRAY11_EV_ENABLE": [[8, "irqarray11-ev-enable"]], "IRQARRAY12": [[9, "irqarray12"]], "Register Listing for IRQARRAY12": [[9, "register-listing-for-irqarray12"]], "IRQARRAY12_EV_SOFT": [[9, "irqarray12-ev-soft"]], "IRQARRAY12_EV_STATUS": [[9, "irqarray12-ev-status"]], "IRQARRAY12_EV_PENDING": [[9, "irqarray12-ev-pending"]], "IRQARRAY12_EV_ENABLE": [[9, "irqarray12-ev-enable"]], "IRQARRAY13": [[10, "irqarray13"]], "Register Listing for IRQARRAY13": [[10, "register-listing-for-irqarray13"]], "IRQARRAY13_EV_SOFT": [[10, "irqarray13-ev-soft"]], "IRQARRAY13_EV_STATUS": [[10, "irqarray13-ev-status"]], "IRQARRAY13_EV_PENDING": [[10, "irqarray13-ev-pending"]], "IRQARRAY13_EV_ENABLE": [[10, "irqarray13-ev-enable"]], "IRQARRAY14": [[11, "irqarray14"]], "Register Listing for IRQARRAY14": [[11, "register-listing-for-irqarray14"]], "IRQARRAY14_EV_SOFT": [[11, "irqarray14-ev-soft"]], "IRQARRAY14_EV_STATUS": [[11, "irqarray14-ev-status"]], "IRQARRAY14_EV_PENDING": [[11, "irqarray14-ev-pending"]], "IRQARRAY14_EV_ENABLE": [[11, "irqarray14-ev-enable"]], "IRQARRAY15": [[12, "irqarray15"]], "Register Listing for IRQARRAY15": [[12, "register-listing-for-irqarray15"]], "IRQARRAY15_EV_SOFT": [[12, "irqarray15-ev-soft"]], "IRQARRAY15_EV_STATUS": [[12, "irqarray15-ev-status"]], "IRQARRAY15_EV_PENDING": [[12, "irqarray15-ev-pending"]], "IRQARRAY15_EV_ENABLE": [[12, "irqarray15-ev-enable"]], "IRQARRAY16": [[13, "irqarray16"]], "Register Listing for IRQARRAY16": [[13, "register-listing-for-irqarray16"]], "IRQARRAY16_EV_SOFT": [[13, "irqarray16-ev-soft"]], "IRQARRAY16_EV_STATUS": [[13, "irqarray16-ev-status"]], "IRQARRAY16_EV_PENDING": [[13, "irqarray16-ev-pending"]], "IRQARRAY16_EV_ENABLE": [[13, "irqarray16-ev-enable"]], "IRQARRAY17": [[14, "irqarray17"]], "Register Listing for IRQARRAY17": [[14, "register-listing-for-irqarray17"]], "IRQARRAY17_EV_SOFT": [[14, "irqarray17-ev-soft"]], "IRQARRAY17_EV_STATUS": [[14, "irqarray17-ev-status"]], "IRQARRAY17_EV_PENDING": [[14, "irqarray17-ev-pending"]], "IRQARRAY17_EV_ENABLE": [[14, "irqarray17-ev-enable"]], "IRQARRAY18": [[15, "irqarray18"]], "Register Listing for IRQARRAY18": [[15, "register-listing-for-irqarray18"]], "IRQARRAY18_EV_SOFT": [[15, "irqarray18-ev-soft"]], "IRQARRAY18_EV_STATUS": [[15, "irqarray18-ev-status"]], "IRQARRAY18_EV_PENDING": [[15, "irqarray18-ev-pending"]], "IRQARRAY18_EV_ENABLE": [[15, "irqarray18-ev-enable"]], "IRQARRAY19": [[16, "irqarray19"]], "Register Listing for IRQARRAY19": [[16, "register-listing-for-irqarray19"]], "IRQARRAY19_EV_SOFT": [[16, "irqarray19-ev-soft"]], "IRQARRAY19_EV_STATUS": [[16, "irqarray19-ev-status"]], "IRQARRAY19_EV_PENDING": [[16, "irqarray19-ev-pending"]], "IRQARRAY19_EV_ENABLE": [[16, "irqarray19-ev-enable"]], "IRQARRAY2": [[17, "irqarray2"]], "Register Listing for IRQARRAY2": [[17, "register-listing-for-irqarray2"]], "IRQARRAY2_EV_SOFT": [[17, "irqarray2-ev-soft"]], "IRQARRAY2_EV_STATUS": [[17, "irqarray2-ev-status"]], "IRQARRAY2_EV_PENDING": [[17, "irqarray2-ev-pending"]], "IRQARRAY2_EV_ENABLE": [[17, "irqarray2-ev-enable"]], "IRQARRAY3": [[18, "irqarray3"]], "Register Listing for IRQARRAY3": [[18, "register-listing-for-irqarray3"]], "IRQARRAY3_EV_SOFT": [[18, "irqarray3-ev-soft"]], "IRQARRAY3_EV_STATUS": [[18, "irqarray3-ev-status"]], "IRQARRAY3_EV_PENDING": [[18, "irqarray3-ev-pending"]], "IRQARRAY3_EV_ENABLE": [[18, "irqarray3-ev-enable"]], "IRQARRAY4": [[19, "irqarray4"]], "Register Listing for IRQARRAY4": [[19, "register-listing-for-irqarray4"]], "IRQARRAY4_EV_SOFT": [[19, "irqarray4-ev-soft"]], "IRQARRAY4_EV_STATUS": [[19, "irqarray4-ev-status"]], "IRQARRAY4_EV_PENDING": [[19, "irqarray4-ev-pending"]], "IRQARRAY4_EV_ENABLE": [[19, "irqarray4-ev-enable"]], "IRQARRAY5": [[20, "irqarray5"]], "Register Listing for IRQARRAY5": [[20, "register-listing-for-irqarray5"]], "IRQARRAY5_EV_SOFT": [[20, "irqarray5-ev-soft"]], "IRQARRAY5_EV_STATUS": [[20, "irqarray5-ev-status"]], "IRQARRAY5_EV_PENDING": [[20, "irqarray5-ev-pending"]], "IRQARRAY5_EV_ENABLE": [[20, "irqarray5-ev-enable"]], "IRQARRAY6": [[21, "irqarray6"]], "Register Listing for IRQARRAY6": [[21, "register-listing-for-irqarray6"]], "IRQARRAY6_EV_SOFT": [[21, "irqarray6-ev-soft"]], "IRQARRAY6_EV_STATUS": [[21, "irqarray6-ev-status"]], "IRQARRAY6_EV_PENDING": [[21, "irqarray6-ev-pending"]], "IRQARRAY6_EV_ENABLE": [[21, "irqarray6-ev-enable"]], "IRQARRAY7": [[22, "irqarray7"]], "Register Listing for IRQARRAY7": [[22, "register-listing-for-irqarray7"]], "IRQARRAY7_EV_SOFT": [[22, "irqarray7-ev-soft"]], "IRQARRAY7_EV_STATUS": [[22, "irqarray7-ev-status"]], "IRQARRAY7_EV_PENDING": [[22, "irqarray7-ev-pending"]], "IRQARRAY7_EV_ENABLE": [[22, "irqarray7-ev-enable"]], "IRQARRAY8": [[23, "irqarray8"]], "Register Listing for IRQARRAY8": [[23, "register-listing-for-irqarray8"]], "IRQARRAY8_EV_SOFT": [[23, "irqarray8-ev-soft"]], "IRQARRAY8_EV_STATUS": [[23, "irqarray8-ev-status"]], "IRQARRAY8_EV_PENDING": [[23, "irqarray8-ev-pending"]], "IRQARRAY8_EV_ENABLE": [[23, "irqarray8-ev-enable"]], "IRQARRAY9": [[24, "irqarray9"]], "Register Listing for IRQARRAY9": [[24, "register-listing-for-irqarray9"]], "IRQARRAY9_EV_SOFT": [[24, "irqarray9-ev-soft"]], "IRQARRAY9_EV_STATUS": [[24, "irqarray9-ev-status"]], "IRQARRAY9_EV_PENDING": [[24, "irqarray9-ev-pending"]], "IRQARRAY9_EV_ENABLE": [[24, "irqarray9-ev-enable"]], "MAILBOX": [[25, "mailbox"]], "Mailbox: An inter-CPU mailbox": [[25, "mailbox-an-inter-cpu-mailbox"]], "Data Transfer Protocol": [[25, "data-transfer-protocol"]], "Abort Protocol": [[25, "abort-protocol"]], "Application Protocol": [[25, "application-protocol"]], "Register Listing for MAILBOX": [[25, "register-listing-for-mailbox"]], "MAILBOX_WDATA": [[25, "mailbox-wdata"]], "MAILBOX_RDATA": [[25, "mailbox-rdata"]], "MAILBOX_EV_STATUS": [[25, "mailbox-ev-status"]], "MAILBOX_EV_PENDING": [[25, "mailbox-ev-pending"]], "MAILBOX_EV_ENABLE": [[25, "mailbox-ev-enable"]], "MAILBOX_STATUS": [[25, "mailbox-status"]], "MAILBOX_CONTROL": [[25, "mailbox-control"]], "MAILBOX_DONE": [[25, "mailbox-done"]], "RESETVALUE": [[26, "resetvalue"]], "Register Listing for RESETVALUE": [[26, "register-listing-for-resetvalue"]], "RESETVALUE_PC": [[26, "resetvalue-pc"]], "TICKTIMER": [[27, "ticktimer"]], "TickTimer: A practical systick timer.": [[27, "ticktimer-a-practical-systick-timer"]], "Configuration": [[27, "configuration"]], "msleep extension": [[27, "msleep-extension"]], "Register Listing for TICKTIMER": [[27, "register-listing-for-ticktimer"]], "TICKTIMER_CONTROL": [[27, "ticktimer-control"]], "TICKTIMER_TIME1": [[27, "ticktimer-time1"]], "TICKTIMER_TIME0": [[27, "ticktimer-time0"]], "TICKTIMER_MSLEEP_TARGET1": [[27, "ticktimer-msleep-target1"]], "TICKTIMER_MSLEEP_TARGET0": [[27, "ticktimer-msleep-target0"]], "TICKTIMER_EV_STATUS": [[27, "ticktimer-ev-status"]], "TICKTIMER_EV_PENDING": [[27, "ticktimer-ev-pending"]], "TICKTIMER_EV_ENABLE": [[27, "ticktimer-ev-enable"]]}, "indexentries": {}})