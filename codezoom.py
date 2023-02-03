#!/usr/bin/env python3

import argparse
import curses
from curses.textpad import Textbox, rectangle
import socket
import select
import time

def main(stdscr):
    parser = argparse.ArgumentParser(description="Build a Cramium FPGA dev image")
    parser.add_argument(
        "--file", required=False, help="The file to zoom around", type=str, default="../xous-cramium/load.lst"
    )
    parser.add_argument(
        "--port", required=False, help="The port to listen on", type=int, default=6502
    )
    args = parser.parse_args()
    udp_socket = socket.socket(family=socket.AF_INET, type=socket.SOCK_DGRAM)
    udp_socket.bind(("127.0.0.1", args.port))

    with open(args.file, 'r') as file_handle:
        text = file_handle.readlines()

    rows, cols = stdscr.getmaxyx()

    text_offset = 0
    stdscr.addstr(0, 0, "listening")
    stdscr.refresh()
    time.sleep(1)

    while True:
        readable, _writeable, _exceptional = select.select([udp_socket], [], [], 0.1)
        if len(readable) > 0:
            data = readable[0].recv(64)
            if data[0] == 2: # value type
                strlen = data[1]
                string = data[2:2+strlen].decode('utf-8')
                # now find the line number that contains this string
                for index, line in enumerate(text):
                    if line.lower().startswith(string.lower()):
                        text_offset = index
                        break
                # center the line number on the screen
                if text_offset > rows // 2:
                    start_line = text_offset - rows // 2
                else:
                    start_line = 0

                stdscr.clear()
                for i in range(rows):
                    if start_line + i == text_offset:
                        if start_line + i < len(text):
                            try:
                                stdscr.addstr(i, 0, text[start_line + i].rstrip(), curses.A_REVERSE)
                            except:
                                pass
                    else:
                        if start_line + i < len(text):
                            try:
                                stdscr.addstr(i, 0, text[start_line + i].rstrip())
                            except:
                                pass

        stdscr.refresh()
        try:
            key = stdscr.getkey()
            if key == 'q':
                break
        except:
            pass

if __name__ == "__main__":
    stdscr = curses.initscr()
    curses.noecho()
    curses.cbreak()
    stdscr.keypad(True)
    stdscr.nodelay(True)

    main(stdscr)

    stdscr.nodelay(False)
    curses.nocbreak()
    stdscr.keypad(False)
    curses.echo()
    curses.endwin()
    exit(0)
