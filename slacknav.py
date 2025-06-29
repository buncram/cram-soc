#!/usr/bin/env python3

"""
give me a template for an ncurses-based python text-based UI that can run on both windows and linux. The dataset we're showing has the structure where each entry represents a path. Each path has at least a start and end point, and 0 or more midpoints, along with a total delay associated with the path.

The UI has the following features:

A top half window which displays data that is a histogram of delay counts. The vertical axis is number of paths, the horizontal axis is the delay of the various paths. Each bin will be drawn using a '#" symbol, and the selected bin is done by rendering the # with an inverted background. The bins are plotted horizontally, and navigated using left/right arrow keys, and selected with the space bar. The number of bins is dynamically set to equal the width of the terminal screen, and thus each bin represents delays from the maximum delay out of the set divided  by the width of the screen in characters.

the lower half window is split into two further halves. The top quarter shows a list of all the paths that are within a selected bin. The list can be scrolled up and down with the up/down arrows, and when the space bar is pressed, the whole screen toggles to another mode where details about the path are displayed and a scrollable text window with all the path details. Hitting 'esc' exits out of that mode and returns to the ui.

the bottom quarter is divided into thirds and contains filters that narrow down the data shown in the upper part of the screen. The left third contains a list of "startpoints", middle third contains a list of "midpoints" and the right third contains a list of "endpoints". Any element in these lists can be toggled as selected or unselected by pressing space on a list entry, or all-selected by hitting a default "*" entry that's always available on the top of the list. As these lists are updated, the upper screen data would ideally dynamically respond and update based on the filters.

Finally, at the very bottom of the UI, there is a one-line text area where commands can be typed in that are used to do advanced commands/modifications. the exact command vocabulary is tbd but each command consists of a verb followed by zero or more arguments and up/down arrows navigates the command history, placing it as the current command in the buffer.

Panes are selected by hitting "tab". Any time tab is hit, the selected pane changes and rotates through the panes.

Try your best, this is a tough one!
"""

import npyscreen
import curses
import math
import argparse
import re
from collections import defaultdict

def parse_sta_report(report):
    with open(report, 'r') as f:
        content = f.read()

    blocks = re.split(r'\n\s*Startpoint:', content)
    results = []

    for idx, block in enumerate(blocks[1:]):
        block_text = f"  Startpoint:{block}"  # reattach trimmed Startpoint
        try:
            start_raw = re.search(r"^  Startpoint: ([^()]+)", block_text, re.M).group(1).strip()
            end_raw = re.search(r"^  Endpoint: ([^()]+)", block_text, re.M).group(1).strip()
            slack = float(re.search(r"^\s*slack.*?(-?\d+\.\d+)", block_text, re.M).group(1))
        except AttributeError:
            continue  # skip malformed block

        # Normalize startpoint and endpoint
        startpoint = re.sub(r'(reg(?:_+\d+)+)_+', 'reg*', start_raw)
        endpoint  = re.sub(r'(reg(?:_+\d+)+)_+', 'reg*', end_raw)

        point_lines = re.findall(r"^\s*(\S+)(?: \([^)]+\))?\s+(\d+\.\d+)\s+\S+", block_text, re.M)
        logic_paths = []

        for full_path, incr in point_lines:
            if float(incr) == 0.0:
                continue
            path_clean = re.sub(r"\([^)]*\)", "", full_path).strip()
            path_clean = re.sub(r"U\d+/(Z|ZN|CO)$", "", path_clean)
            if len(path_clean) == 0:
                path_clean = "  "
            logic_paths.append(path_clean)

        # Collapse consecutive identical paths
        collapsed = []
        last = None
        count = 0
        for path in logic_paths:
            if path == last:
                count += 1
            else:
                if last:
                    collapsed.append(f"{last}/logic+{count}")
                last = path
                count = 1
        if last:
            collapsed.append(f"{last}/logic+{count}")

        results.append({
            "startpoint": startpoint,
            "endpoint": endpoint,
            "slack": slack,
            "collapsed_paths": collapsed
        })

    return results

def get_min_max_slack(results):
    slacks = [entry["slack"] for entry in results]
    return min(slacks), max(slacks)

def get_unique_startpoints(results):
    return sorted(set(entry["startpoint"] for entry in results))

def get_unique_endpoints(results):
    return sorted(set(entry["endpoint"] for entry in results))

def get_unique_collapsed_path_elements(results):
    path_set = set()
    for entry in results:
        path_set.update(entry["collapsed_paths"])
    return sorted(path_set)

def bin_results_by_slack(results, min_slack, max_slack, num_bins):
    bin_width = (max_slack - min_slack) / num_bins
    bins = [[] for _ in range(num_bins)]

    for record in results:
        s = record["slack"]
        if s == max_slack:
            index = num_bins - 1
        elif s > max_slack:
            continue
        else:
            index = int((s - min_slack) // bin_width)
        bins[index].append(record)

    return bins

class HistogramWidget(npyscreen.BoxTitle):
    def __init__(self, *args, **kwargs):
        self.bins = []
        self.selected_bin = 0
        super().__init__(*args, **kwargs)

    def update_histogram(self):
        height = self.height - 4  # space for borders + X axis
        width = self.width - 2    # inside box border
        max_count = max(len(b) for b in self.bins) if self.bins else 1
        bin_width = 1  # fixed as per assumption
        bin_count = len(self.bins)

        # Reset content
        display_lines = [" " * width for _ in range(height)]

        for x in range(bin_count):
            bin_len = len(self.bins[x])
            bin_height = int((bin_len / max_count) * height)
            char = '.'
            if x == self.selected_bin:
                char = "#"

            for y in range(bin_height):
                row = height - y - 1
                line = list(display_lines[row])
                line[x] = char
                display_lines[row] = "".join(line)

        # Label top Y-axis
        display_lines.insert(0, f"max={max_count}".ljust(width))

        # Label X-axis with start slack of each bin
        slack_labels = [""] * width
        if hasattr(self, "slack_min") and hasattr(self, "slack_max"):
            slack_range = self.slack_max - self.slack_min
            slack_step = slack_range / bin_count
            for x in range(0, width, max(1, width // 8)):
                slack_value = self.slack_min + (x * slack_step)
                label = f"{slack_value:.2f}"
                for i, ch in enumerate(label):
                    if x + i < width:
                        slack_labels[x + i] = ch
        display_lines.append("".join(ch or " " for ch in slack_labels))

        self.values = display_lines
        self.display()

class PathListWidget(npyscreen.BoxTitle):
    _contained_widget = npyscreen.MultiLine

    def update_paths(self, paths):
        self.entry_data = paths
        self.values = [f"{p['startpoint']} -> {p['endpoint']} ({p['slack']})" for p in paths]
        self.display()

    def get_selected_path(self):
        if not hasattr(self, 'entry_data') or not self.entry_data:
            return None
        index = self.entry_widget.cursor_line
        if 0 <= index < len(self.entry_data):
            return self.entry_data[index]
        return None

    def actionHighlighted(self, act_on_this, keypress):
        selected_path = self.get_selected_path()
        npyscreen.notify_confirm(f"SELECTED!!!!", title="Path details") #, wide = True

    # def handle_input(self, key):
    #     # Forward input to the underlying MultiLine widget
    #     self.entry_widget.handle_input(_input = key)

class FilterWidget(npyscreen.BoxTitle):
    def __init__(self, *args, category=None, **kwargs):
        super().__init__(*args, **kwargs)
        self.category = category
        self.selected_items = set()

    def update_filter_items(self, data):
        items = {p[self.category] for p in data}
        self.values = ["*"] + sorted(items)

    def toggle_item(self, index):
        item = self.values[index]
        if item == "*":
            self.selected_items = set()
        elif item in self.selected_items:
            self.selected_items.remove(item)
        else:
            self.selected_items.add(item)


class CommandLine(npyscreen.Textfield):
    def __init__(self, *args, **kwargs):
        super().__init__(*args, **kwargs)
        self.command_history = []
        self.history_index = -1
        self.prompt = "slacknav> "
        self.value = self.prompt
        self.cursor_position = len(self.value)

    def get_user_input(self):
        return self.value[len(self.prompt):]

    def process_command(self):
        cmd = self.get_user_input().strip()
        if cmd:
            if cmd == 'quit' or cmd == 'exit':
                exit(0)
            else:
                self.command_history.append(cmd)
                self.history_index = len(self.command_history)
                npyscreen.notify_confirm(f"Echo: {cmd}", title="Command Output", wide=True)
        self.value = self.prompt
        self.cursor_position = len(self.value)

    def history_up(self):
        if self.history_index > 0:
            self.history_index -= 1
            self.value = self.prompt + self.command_history[self.history_index]
            self.cursor_position = len(self.value)

    def history_down(self):
        if self.history_index < len(self.command_history) - 1:
            self.history_index += 1
            self.value = self.prompt + self.command_history[self.history_index]
        else:
            self.value = self.prompt
        self.cursor_position = len(self.value)

    def when_value_edited(self):
        if not self.value.startswith(self.prompt):
            self.value = self.prompt + self.get_user_input()
        self.cursor_position = len(self.value)

    def h_exit_down(self, ch=None):
        self.process_command()

class PathDetailPopup(npyscreen.Popup):
    def create(self):
        self.detail_text = self.add(npyscreen.Pager)


class MainApp(npyscreen.NPSAppManaged):
    def __init__(self, report):
        self.report = report
        super().__init__()

    def onStart(self):
        curses.start_color()
        curses.init_pair(1, curses.COLOR_RED, curses.COLOR_BLACK)
        curses.init_pair(2, curses.COLOR_GREEN, curses.COLOR_BLACK)
        form = self.addForm("MAIN", MainForm, name="Path Delay UI")
        self.setNextForm("MAIN")
        form.edit()

class MainForm(npyscreen.FormBaseNew):
    def create(self):
        self.report = self.parentApp.report
        self.paths = paths = parse_sta_report(self.report)
        self.min_slack, self.max_slack = get_min_max_slack(paths)
        self.absmax_slack = self.max_slack
        startpoints = get_unique_startpoints(paths)
        endpoints = get_unique_endpoints(paths)
        midpoints = get_unique_collapsed_path_elements(paths)

        max_y, max_x = self.useable_space()
        max_y = max_y - 5

        self.selected_bin = 0
        self.data = []  # Replace with actual data
        self.filtered_data = self.data

        self.histogram = self.add(HistogramWidget, name="Delay Histogram",
                                  relx=0, rely=0, max_height=max_y // 3)
        histo_width = self.histogram.width - 2 # space for border
        bins = bin_results_by_slack(self.paths, self.min_slack, self.max_slack, histo_width)
        self.histogram.bins = bins
        self.histogram.min_slack = self.min_slack
        self.histogram.max_slack = self.max_slack

        self.path_list = self.add(PathListWidget, name="Paths in Bin",
                                  relx=0, rely=max_y // 3, max_height=max_y // 3)
        self.filter_start = self.add(FilterWidget, name="Start Points", relx=0,
                                     rely=2 * max_y // 3, max_height=max_y // 3, max_width=max_x // 3 - 1, category="start")
        self.filter_mid = self.add(FilterWidget, name="Mid Points", relx=max_x // 3,
                                   rely=2 * max_y // 3, max_height=max_y // 3, max_width=max_x // 3 - 1, category="mid")
        self.filter_end = self.add(FilterWidget, name="End Points", relx=2 * max_x // 3,
                                   rely=2 * max_y // 3, max_height=max_y // 3, max_width=max_x // 3 - 1, category="end")

        self.command_line = self.add(CommandLine, relx=0, rely=max_y, max_width=max_x - 5)

        self.pane_order = [self.histogram, self.path_list,
                           self.filter_start, self.filter_mid, self.filter_end, self.command_line]

        # setup default
        self.set_editing(self.histogram)
        self.update_display()

    def handle_input(self, key):
        if key != -1:
            self.process_key(key)
        self.update_display()

    def update_display(self):
        self.histogram.update_histogram()
        self.path_list.update_paths(self.get_paths_in_selected_bin())
        self.filter_start.update_filter_items(self.data)
        self.filter_mid.update_filter_items(self.data)
        self.filter_end.update_filter_items(self.data)

    def get_paths_in_selected_bin(self):
        # Apply filters and select bin contents
        # Placeholder: return all paths
        return self.histogram.bins[self.histogram.selected_bin]

    def set_editing(self, widget):
        for w in self.pane_order:
            w.editing = (w == widget)

    def process_key(self, key):
        current = self._widgets__[self.editw]

        # if key == ord('\t'):
        #     self.pane_index = (self.pane_index + 1) % len(self.pane_order)
        #     self.set_editing(self.pane_order[self.pane_index])
        if key in (curses.KEY_LEFT, curses.KEY_RIGHT, curses.KEY_B1, curses.KEY_B3):
            if current == self.histogram:
                self.selected_bin = max(0, min(
                    self.selected_bin + (1 if key in ((curses.KEY_RIGHT, curses.KEY_B3)) else -1),
                    len(self.histogram.bins) - 1
                ))
                self.histogram.selected_bin = self.selected_bin
        elif key in ((ord('+'), ord('-'))):
            if current == self.histogram:
                if key == ord('+'): # zoom in
                    span = self.histogram.max_slack - self.histogram.min_slack
                    self.histogram.max_slack = self.histogram.min_slack + span / 2
                else: # zoom out
                    self.histogram.max_slack = min(self.histogram.max_slack * 2, self.absmax_slack)
                bins = bin_results_by_slack(self.paths, self.histogram.min_slack, self.histogram.max_slack, self.histogram.width - 2)
                self.histogram.bins = bins

        elif key == ord(' '):
            if current in (self.filter_start, self.filter_mid, self.filter_end):
                index = current.cursor_line
                current.toggle_item(index)
        elif key in (curses.KEY_UP, curses.KEY_A2):
            if current == self.command_line:
                self.command_line.history_up()
            elif current == self.path_list:
                self.path_list.handle_input(key)
        elif key in (curses.KEY_DOWN, curses.KEY_C2):
            if current == self.command_line:
                self.command_line.history_down()
            elif current == self.path_list:
                self.path_list.handle_input(key)
        elif key == 27:  # ESC
            pass
        elif key == ord('q'):
            exit(0)


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description="Slack navigator", prog="slacknav")
    parser.add_argument(
        "--report", required=True, help="Delay file to parse", type=str
    )
    args = parser.parse_args()

    app = MainApp(args.report)
    app.run()
