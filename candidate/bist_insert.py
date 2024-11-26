import re
from pathlib import Path
from datetime import datetime

def add_ports_to_module(file_path, module_ports_map):
    """
    Parses a Verilog file, finds module instantiations, and adds specified ports to them.

    :param file_path: Path to the Verilog file.
    :param module_ports_map: Dictionary where keys are (module names, ram name) and values are the ports to add, *or*.
                             module name : { port_name : count }
                             e.g., {("Ram_1w_1rs", "RAM_1024_32"): ".rbif_ramtype1"} *or*
                            {"InstructionCache": {".rbif_ramtype1": 4} }
    """
    with open(file_path, 'r') as file:
        lines = file.read()

    olines = f'// Post-processing pass by bist_insert.py on {str(datetime.now())}\n'
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

    def add_port_to_ramleaf(match):
        if match.group(2) == ramname:
            if new_port in ports_added:
                count = ports_added[new_port] + 1
            else:
                count = 1
            ports_added[new_port] = count
            modified_instance = match.group(1) + match.group(3) + f"\t\t{new_port}[{count - 1}],\n" + match.group(4) + ";"
        else:
            modified_instance = match.group(1) + match.group(3) + match.group(4) + ";"
        return modified_instance

    def add_port_to_instance(match):
        if match.group(2) == specifier:
            modified_instance = match.group(1) + match.group(2) + match.group(3) + match.group(4)
            for port, count in new_port.items():
                if port in ports_added:
                    start = ports_added[port]
                else:
                    start = 0
                ports_added[port] = start + count
                modified_instance += f"\t\t{port}\t\t({port.lstrip('.')}[{start + count - 1}:{start}]),\n"
            modified_instance += match.group(5)
        else:
            modified_instance = match.group(1) + match.group(2) + match.group(3) + match.group(4) + match.group(5)
        return modified_instance

    modules = {}
    module_ports = {}
    for mod in module_pattern.finditer(lines):
        module_name = mod.group(3)

        # Iterate through each module and its additional port
        bisted_code = mod.group(5)
        ports_added = {}
        for specifier, new_port in module_ports_map.items():
            if type(specifier) is tuple:
                # specifier for leaf RAM modules
                (ram_module, ramname) = specifier
                # Regular expression to locate RAM instantiations
                ram_pattern = re.compile(
                    # group 2 is the module name
                    # group 4 is the parameter list
                    rf"(\s*{ram_module}\s#\([^;]*ramname\(\"(\S*)\"\)[^;]*?.\s*\)\s*[\S]*\s)(\(.)([^;]*)\;",
                    re.DOTALL
                )
                # Apply the transformation to all matching instances
                bisted_code = ram_pattern.sub(add_port_to_ramleaf, bisted_code)
            else:
                # specifier for instances
                assert(type(specifier) is str)
                # print(f'finding {specifier} in {str(file_path)}')
                instance_pattern = re.compile(
                    # A somewhat fragile search for instances - this won't handle parameters, but it turns out
                    # we don't need to in all the cases we care about.
                    # group 1 is any whitespace before the module name
                    # group 2 is the module name
                    # group 3 is the instance of the module
                    # group 4 is the opening parenthesis of the parameter list
                    # group 5 is the parameter list including the trailing semicolon
                    rf"(^\s*)({specifier})(.*?)(\(.)(.*?\)\;)",
                    re.DOTALL | re.MULTILINE
                )
                bisted_code = instance_pattern.sub(add_port_to_instance, bisted_code)

        bist_params = ''
        for port, count in ports_added.items():
            bist_params += f'\trbif.slave\t{port}[{count}],\n'
        modules[module_name] = [mod.group(1), mod.group(2), bist_params, mod.group(4), bisted_code]
        if len(ports_added) > 0:
            module_ports[module_name] = {}
            for port, count in ports_added.items():
                module_ports[module_name][port] = count

        for c in modules[module_name]:
            olines += c

    # extract the end matter and append it
    em = end_matter.search(lines)
    olines += em.group(2)

    # Write the modified Verilog content back to the file
    with open(file_path.parent / Path(file_path.stem + '.sv'), 'w') as file:
        file.write(olines)

    print(f"Modified file saved to {file_path.stem + '.sv'}; added {module_ports}")
    return module_ports

def extract_modules(file_path):
    modules_found = {}
    with open(file_path, 'r') as file:
        lines = file.read()
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
    for mod in module_pattern.finditer(lines):
        modules_found[mod.group(3)] = []
    return modules_found


if __name__ == "__main__":
    # list of files to process, starting with the lowest hierarchy member first.
    # Each entry has this format:
    #     Path( path_to_file ) : 'top_level_module_name'
    verilog_files = {
        Path("./candidate/libs/VexRiscv_CramSoC.v") : 'VexRiscvAxi4',
        Path("./candidate/cram_axi.v") : 'cram_axi',
    }
    module_ports_to_add = {
        ("Ram_1w_1rs", "RAM_DP_1024_32") : ".rbif_rdram1kx32",
        ("Ram_1w_1rs", "RAM_DP_512_64") : ".rbif_rdram512x64",
        ("Ram_1w_1rs", "RAM_DP_128_22") : ".rbif_rdram128x22",
        ("Ram_1w_1rs", "RAM_DP_32_16_WM") : ".rbif_rdram32x16",
        ("Ram_1w_1rs", "RAM_DP_32_16_MM") : ".rbif_rdram32x16",
    }

    # pass #1 - insert BIST ports into the RAM and leaf cell module decl
    module_ports_added = {}
    for file, top_level in verilog_files.items():
        for (mod, ports) in add_ports_to_module(file, module_ports_to_add).items():
            module_ports_added[mod] = ports

        sv_file = Path(str(file).replace('.v', '.sv'))

        # pass #2 - go through processed files and propagate leaf cells to top module
        #   1. iterate through each module
        #   2. search for instances of modules that were modified - these are the keys in the dictionary
        #      `module_ports_added`
        #   3. add the new ports to the found instance - this may require merging port counts
        #   4. return a list of the module instantiations that were modified
        #   5. if the list was not empty, repeat 1.
        ports_at_level = {}
        while True:
            print(f'entering pass 2 with {module_ports_added}')
            for (mod, ports) in add_ports_to_module(sv_file, module_ports_added).items():
                ports_at_level[mod] = ports
            module_ports_added = ports_at_level
            if len(ports_at_level) == 1 and top_level in ports_at_level:
                ports_at_level = {}
                break
            else:
                ports_at_level = {}
        print(f'end {file}, {module_ports_added}')
