#! /usr/bin/env python3
import argparse
import sys
from Crypto.IO import PEM
from cryptography import x509
from cryptography.hazmat.primitives.serialization import Encoding
from cryptography.hazmat.primitives.serialization import PublicFormat

import binascii

def main():
    global DEVKEY_PATH

    parser = argparse.ArgumentParser(description="Sign binary images for Precursor")
    parser.add_argument(
        "--dev-pubkey", required=False, help="developer public key (X.509 ED25519)", type=str, nargs='?', metavar=('developer key'), const='./devkey/dev-x509.crt'
    )
    parser.add_argument(
        "--output", required=False, help="output name, defaults to keystore.bin", type=str, nargs='?', metavar=('output file'), const='keystore.bin'
    )
    args = parser.parse_args()
    if not len(sys.argv) > 1:
        print("No arguments specified, doing nothing. Use --help for more information.")
        exit(1)

    if args.output is None:
        output_file = 'keystore.bin'
    else:
        output_file = args.output

    if args.dev_pubkey is None:
        dev_pubkey = './devkey/dev-x509.crt'
    else:
        dev_pubkey = args.dev_pubkey

    with open(dev_pubkey, "rb") as dev_pubkey_f:
        cert_file = dev_pubkey_f.read()
        #print(cert_file)
        cert = x509.load_pem_x509_certificate(cert_file)
        pubkey = cert.public_key().public_bytes(Encoding.Raw, PublicFormat.Raw)
        print("using public key: ", binascii.hexlify(pubkey))

        with open(output_file, "wb") as ofile:
            written = 0
            written += ofile.write(bytes([0] * 0x18 * 4)) # pad to the public key
            written += ofile.write(pubkey)
            written += ofile.write(bytes([0] * ( (0xff * 4) - written)))
            written += ofile.write(bytes(int(0x0001).to_bytes(4, 'little'))) # version 00.01 plus all other fuses blank
            print("wrote {} bytes to {}".format(written, output_file))

if __name__ == "__main__":
    main()
    exit(0)
