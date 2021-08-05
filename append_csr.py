#!/usr/bin/python3

import argparse
import hashlib
import subprocess

def main():
    parser = argparse.ArgumentParser(description="Pad and append CSV file to FPGA bitstream")
    parser.add_argument(
        "-b", "--bitstream", required=True, help="file containing FPGA bitstream", type=str
    )
    parser.add_argument(
        "-c", "--csv-file", required=True, help="file containing CSV input", type=str
    )
    parser.add_argument(
        "-o", "--output-file", required=True, help="destination file for binary data", type=str
    )
    args = parser.parse_args()

    bitstream_pad_to = 0x278000
    pad_to = 0x7FC0
    with open(args.bitstream, "rb") as bitstream:
        with open(args.csv_file, "rb") as ifile:
            with open(args.output_file, "wb") as ofile:
                # create the CSV appendix
                data = ifile.read() # read in the whole block of CSV data

                git_rev = subprocess.Popen(["git", "describe", "--long", "--always"],
                    stdout=subprocess.PIPE,
                    stderr=subprocess.PIPE)
                (rev, err) = git_rev.communicate()
                data += b"git_rev,"
                data += rev

                odata = bytearray()
                odata += len(data).to_bytes(4, 'little')
                odata += data

                padding = bytes([0xff]) * (pad_to - len(data) - 4)
                odata += padding

                hasher = hashlib.sha512()
                hasher.update(odata)
                digest = hasher.digest()
                odata += digest
                # odata now contains the csv appendix

                # assemble the final output file
                bits = bitstream.read() # read in all the bitstream
                position = 0
                # seek past the preamble junk that's ignored
                while position < len(bits):
                   sync = int.from_bytes(bits[position:position + 4], 'big')
                   if sync == 0xaa995566:
                      break
                   position = position + 1
                program_data = bits[position:]

                ofile.write(bytes([0xff] * 8)) # insert padding so that AES blocks line up on erase block boundaries

                ofile.write(program_data)
                # pad it, so the CSR data is in the right place
                bs_padding = bytes([0xff]) * (bitstream_pad_to - len(program_data))
                ofile.write(bs_padding)

                # add the CSR data
                ofile.write(odata)


if __name__ == "__main__":
    main()
