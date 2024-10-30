#!/usr/bin/env python3
# encoding: utf-8

import os

# Define the copyright header
copyright_header = """\
// (c) Copyright CrossBar, Inc. 2024.
//
// This documentation describes Open Hardware and is licensed under the
// [CERN-OHL-W-2.0].
//
// You may redistribute and modify this documentation under the terms of the
// [CERN-OHL- W-2.0 (http://ohwr.org/cernohl)]. This documentation is
// distributed WITHOUT ANY EXPRESS OR IMPLIED WARRANTY, INCLUDING OF
// MERCHANTABILITY, SATISFACTORY QUALITY AND FITNESS FOR A PARTICULAR PURPOSE.
// Please see the [CERN-OHL- W-2.0] for applicable conditions.
"""


# Walk through directories and files recursively
for root, _, files in os.walk('.'):
    for filename in files:
        # Check if the file extension matches .v or .sv
        if filename.endswith('.v') or filename.endswith('.sv') or filename.endswith('.rs'):
            file_path = os.path.join(root, filename)

            # Read the file content
            with open(file_path, 'r') as file:
                content = file.read()

            if "Copyright CrossBar, Inc. 2024." not in content:
                # Write the copyright header and the original content back to the file
                print(f"Updating {str(file_path)} with license header...")
                with open(file_path, 'w') as file:
                    file.write(copyright_header + '\n' + content)
