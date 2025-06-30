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

class HistogramWidget(npyscreen.MultiLineAction):
    def __init__(self, *args, **kwargs):
        self.bins = []
        self.selected_bin = 0
        super().__init__(*args, **kwargs)

    def update_histogram(self):
        height = self.height - 4  # space for borders + X axis labels
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

        # Label X-axis with start slack of each bin
        slack_labels = [""] * width
        slack_range = self.max_slack - self.min_slack
        slack_step = slack_range / bin_count
        for x in range(0, width, max(1, width // 8)):
            slack_value = self.min_slack + (x * slack_step)
            label = f"{slack_value:.3f}"
            for i, ch in enumerate(label):
                if x + i < width:
                    slack_labels[x + i] = ch
        display_lines.append("".join(ch or " " for ch in slack_labels))

        # Label top Y-axis
        selected_count = len(self.bins[self.selected_bin])
        selected_slack = slack_step * self.selected_bin + self.min_slack
        display_lines.insert(0, f"max={max_count} selected={selected_count}({selected_slack:.4f}ns)".ljust(width))

        self.values = display_lines
        self.display()

    def handle_input(self, key):
        # npyscreen.notify_confirm(f"Key code: {key}", title="Debug")
        if key in (curses.KEY_LEFT, curses.KEY_RIGHT, 452, 454): # curses.KEY_B1, curses.KEY_B3
            self.selected_bin = max(0, min(
                self.selected_bin + (1 if key in ((curses.KEY_RIGHT, 454)) else -1), # curses.KEY_B3
                len(self.bins) - 1
            ))
            self.selected_bin = self.selected_bin
            self.update_histogram()
            self.parent.update_display()
            self.display()
        elif key in ((ord('+'), ord('-'))):
            if key == ord('+'): # zoom in
                span = self.max_slack - self.min_slack
                self.max_slack = self.min_slack + span / 2
            else: # zoom out
                self.max_slack = min(self.max_slack * 2, self.absmax_slack)
            bins = bin_results_by_slack(self.paths, self.min_slack, self.max_slack, self.width - 2)
            self.bins = bins
            self.update_histogram()
            self.parent.update_display()
            self.display()
        else:
            return super().handle_input(key)


class DetailPopup(npyscreen.ActionFormV2):
    preloaded_content = ""

    def create(self):
        self.text_widget = self.add(
            npyscreen.Pager,
            name="Details",
        )

    def beforeEditing(self):
        if self.preloaded_content:
            content = []
            content += [self.preloaded_content['startpoint']]
            for line in self.preloaded_content['collapsed_paths']:
                content += [f"  {line}"]
            content += [self.preloaded_content['endpoint']]
            content += [f"Slack: {self.preloaded_content['slack']}"]
            self.text_widget.values = content

    def set_text(self, content):
        self.preloaded_content = content

    def on_ok(self):
        self.parentApp.setNextForm("MAIN")

    def on_cancel(self):
        self.parentApp.setNextForm("MAIN")

    def handle_input(self, key):
        if key not in (curses.KEY_UP, curses.KEY_DOWN, 450, 456):
            self.on_cancel()
            self.editing = False
            self.exit_editing()
            return
        return super().handle_input(key)

class SelectableMultiLine(npyscreen.MultiLine):
    def handle_input(self, key):
        if key in ("KEY_UP", "KEY_DOWN"):
            return super().handle_input(key)
        elif key in (" ", "^M"):
            self.parent.toggle_item(self.cursor_line)
        else:
            return super().handle_input(key)

class PathListWidget(npyscreen.MultiLineAction):
    def __init__(self, *args, **keywords):
        super().__init__(*args, **keywords)
        self.add_handlers({
            "^M": self.actionHighlighted,   # Enter
            " ": self.actionHighlighted,    # Space
        })

    def actionHighlighted(self, act_on_this=None, key_press=None):
        details = self.entry_data[self.cursor_line]
        # npyscreen.notify_confirm(f"selected {details}", title="Path details") #, wide = True
        form_class, _, _ = self.parent.parentApp._Forms["DETAIL_POPUP"]
        form_class.preloaded_content = details
        self.parent.parentApp.switchForm("DETAIL_POPUP")

    def update_paths(self, paths):
        self.entry_data = paths
        self.values = [f"{p['startpoint']} -> {p['endpoint']} ({p['slack']})" for p in paths]
        self.display()

    def handle_input(self, key):
        # Normalize Windows-specific arrow keys
        if key in (450,):  # Windows Up curses.KEY_A2
            key = curses.KEY_UP
        elif key in (456,):  # Windows Down curses.KEY_C2
            key = curses.KEY_DOWN
        return super().handle_input(key)

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
        self.addFormClass("DETAIL_POPUP", DetailPopup)
        form = self.addForm("MAIN", MainForm, name="Path Delay UI")
        self.setNextForm("MAIN")
        # form.edit()

class MainForm(npyscreen.FormBaseNew):
    def create(self):
        self.report = self.parentApp.report
        self.paths = paths = parse_sta_report(self.report)
        self.min_slack, self.max_slack = get_min_max_slack(paths)
        max_y, max_x = self.useable_space()
        max_y = max_y - 1

        self.selected_bin = 0
        self.data = []  # Replace with actual data
        self.filtered_data = self.data

        self.histogram = self.add(HistogramWidget, name="Delay Histogram",
                                  relx=0, rely=0, max_height=max_y // 2)
        histo_width = self.histogram.width - 2 # space for border
        bins = bin_results_by_slack(self.paths, self.min_slack, self.max_slack, histo_width)
        self.histogram.bins = bins
        self.histogram.min_slack = self.min_slack
        self.histogram.max_slack = self.max_slack
        self.histogram.absmax_slack = self.max_slack
        self.histogram.paths = paths

        self.path_list = self.add(PathListWidget, name="Paths in Bin",
                                  relx=0, rely=max_y // 2, max_height=max_y // 2)

        self.pane_order = [self.histogram, self.path_list]

        # setup default
        self.set_editing(self.histogram)
        self.update_display()

    def update_display(self):
        self.histogram.update_histogram()
        self.path_list.update_paths(self.get_paths_in_selected_bin())

    def get_paths_in_selected_bin(self):
        return self.histogram.bins[self.histogram.selected_bin]

    def set_editing(self, widget):
        for w in self.pane_order:
            w.editing = (w == widget)

    def handle_input(self, key):
        if key != -1:
            self.process_key(key)
        super().handle_input(key)
        self.update_display()

    def process_key(self, key):
        if key == ord('q'):
            exit(0)

if __name__ == '__main__':
    parser = argparse.ArgumentParser(description="Slack navigator", prog="slacknav")
    parser.add_argument(
        "--report", required=True, help="Delay file to parse", type=str
    )
    args = parser.parse_args()

    app = MainApp(args.report)
    app.run()
