import re
from pathlib import Path
from datetime import datetime

def add_ports_to_module(file_path, module_ports_map):
    """
    Parses a Verilog file, finds module instantiations, and adds specified ports to them.

    :param file_path: Path to the Verilog file.
    :param module_ports_map: Dictionary where keys are module names and values are the ports to add.
                             e.g., {"Ram_1w_1rs": ".rbif_ramtype1"}
    """
    with open(file_path, 'r') as file:
        lines = file.read()

    olines = f'// Modified by bist_insert.py on {str(datetime.now())}\n\n'
    module_pattern = re.compile(
        # Captures everything up until the very last "endmodule" statement; trailing comments
        # & whitespace need to be separately handled.
        # group 1 is anything before the module (comments, headers, whitespace)
        # group 2 is the module declaration
        # group 3 is the module name itself - not to be re-inserted in the stream
        # group 4 is the parameter list
        # group 5 is the module body, until the last endmodule statement
        rf"(.*?)(module\s*([\S]*?)\s*\(.?)([^;]*\;)(.*?endmodule)",
        re.DOTALL
    )
    end_matter = re.compile(
        # group 2 is everything that module_pattern couldn't capture
        rf"(.*endmodule)(.*)",
        re.DOTALL
    )

    def add_port_to_instance(match):
        if match.group(2) == ramname:
            if new_port in ports_added:
                count = ports_added[new_port] + 1
            else:
                count = 0
            ports_added[new_port] = count
            modified_instance = match.group(1) + match.group(3) + f"\t\t{new_port}[{count}],\n" + match.group(4) + ";"
        else:
            modified_instance = match.group(1) + match.group(3) + match.group(4) + ";"
        return modified_instance

    modules = {}
    for mod in module_pattern.finditer(lines):
        module_name = mod.group(3)

        # Iterate through each module and its additional port
        bisted_code = mod.group(5)
        ports_added = {}
        for specifier, new_port in module_ports_map.items():
            (ram_module, ramname) = specifier
            # Regular expression to locate RAM instantiations
            ram_pattern = re.compile(
                # group 2 is the module name
                # group 4 is the parameter list
                rf"(\s*{ram_module}\s#\([^;]*ramname\(\"(\S*)\"\)[^;]*?.\s*\)\s*[\S]*\s)(\(.)([^;]*)\;",
                re.DOTALL
            )
            # Apply the transformation to all matching instances
            bisted_code = ram_pattern.sub(add_port_to_instance, bisted_code)

        bist_params = ''
        for port, count in ports_added.items():
            bist_params += f'\trbif.slave\t{port}[{count+1}],\n'
        modules[module_name] = [mod.group(1), mod.group(2), bist_params, mod.group(4), bisted_code]
        for c in modules[module_name]:
            olines += c

    # extract the end matter and append it
    em = end_matter.search(lines)
    olines += em.group(2)

    # Write the modified Verilog content back to the file
    with open(file_path.parent / Path(file_path.stem + '.sv'), 'w') as file:
        file.write(olines)

    print(f"Modified file saved to {file_path.stem + '.sv'}; added {ports_added}")


if __name__ == "__main__":
    # the ordering of these files is important. The lower file in the hierarchy must come
    # first so its BIST ports are inserted and accounted for.
    verilog_files = [
        Path("./candidate/libs/VexRiscv_CramSoC.v"),
        Path("./candidate/cram_axi.v"),
    ]
    module_ports_to_add = {
        ("Ram_1w_1rs", "RAM_DP_1024_32") : ".rbif_rdram1kx32",
        ("Ram_1w_1rs", "RAM_DP_512_64") : ".rbif_rdram512x64",
        ("Ram_1w_1rs", "RAM_DP_128_22") : ".rbif_rdram128x22",
        ("Ram_1w_1rs", "RAM_DP_32_16_WM") : ".rbif_rdram32x16",
        ("Ram_1w_1rs", "RAM_DP_32_16_MM") : ".rbif_rdram32x16",
    }

    for file in verilog_files:
        add_ports_to_module(file, module_ports_to_add)
