import re
from pathlib import Path
from datetime import datetime

def add_ports_to_module(file_path, module_ports_map, top_ram_ports=None):
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
                    if port in top_ram_ports:
                        start = top_ram_ports[port]
                    else:
                        start = 0
                ports_added[port] = start + count
                modified_instance += f"\t\t{port}\t\t({port.lstrip('.')}[{start}:{start + count - 1}]),\n"
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

        modules[module_name] = [mod.group(1), mod.group(2), ports_added, mod.group(4), bisted_code]

    # extract the end matter and append it
    em = end_matter.search(lines)
    end_matter = em.group(2)

    return (modules, end_matter)

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
    next_level_ports = {}
    for file, top_level in verilog_files.items():
        (modules, end_matter) = add_ports_to_module(file, module_ports_to_add)

        sv_file = Path(str(file).replace('.v', '.sv'))
        module_ports = {}
        top_ram_ports = {}
        with open(sv_file, 'w') as f:
            olines = f'// Post-processing pass by bist_insert.py on {str(datetime.now())}\n\n'
            for module_name, code_elements in modules.items():
                bist_params = ''
                # convert code_elements[2] from a dict into a str
                if module_name != top_level:
                    ports_added = code_elements[2]
                    for port, count in ports_added.items():
                        p = port.lstrip('.')
                        bist_params += f'\trbif.slave\t{p}[{count}],\n'
                else:
                    # defer finalization until later
                    top_ram_ports = code_elements[2]
                    ports_added = {}

                code_elements[2] = bist_params

                if len(ports_added) > 0:
                    module_ports[module_name] = {}
                    for port, count in ports_added.items():
                        module_ports[module_name][port] = count

                for c in code_elements:
                    olines += c
            olines += end_matter
            f.write(olines)

        # at this point on the top level, all of the leaf-level RAMs instantiated at that
        # level have .rbif_xxx[n] ports inserted where n starts at 0 and goes to count-1.
        # however, the very top level module definition has not yet been inserted. In the
        # next step we have to aggregate ports of the same type and merge them to create
        # the correct count.
        if len(next_level_ports) != 0:
            for module, ports in next_level_ports.items():
                module_ports[module] = ports

        def insert_port(port, count):
            if port in module_ports[module_name]:
                start = module_ports[module_name][port]
            else:
                start = 0
            module_ports[module_name][port] = start + count
            p = port.lstrip('.')
            return f'\trbif.slave\t{p}[{start}:{count + start - 1}],\n'

        # pass #2 - go through processed files and propagate leaf cells to top module
        #   1. iterate through each module
        #   2. search for instances of modules that were modified - these are the keys in the dictionary
        #      `module_ports`
        #   3. add the new ports to the found instance - this may require merging port counts
        #   4. return a list of the module instantiations that were modified
        #   5. if the list was not empty, repeat 1.
        ports_at_level = {}
        print(f'entering pass 2 with {module_ports}')
        (modules, end_matter) = add_ports_to_module(sv_file, module_ports, top_ram_ports)
        with open(sv_file, 'w') as f:
            olines = ''
            bist_params = ''
            for module_name, code_elements in modules.items():
                if module_name not in module_ports: # create an entry if one doesn't exist
                    module_ports[module_name] = {}
                bist_params = ''

                if len(code_elements[2]) > 0:
                    ports_added = code_elements[2]
                    for port, count in ports_added.items():
                        bist_params += insert_port(port, count)
                        if port in top_ram_ports:
                            del top_ram_ports[port]

                    for port, count in top_ram_ports.items():
                        bist_params += insert_port(port, count)

                    code_elements[2] = bist_params
                else:
                    code_elements[2] = ''

                for c in code_elements:
                    olines += c
            olines += end_matter
            f.write(olines)
        print(f'end {file}, {module_ports}')
        if top_level in module_ports:
            next_level_ports[top_level] = module_ports[top_level]
