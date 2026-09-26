"""Classify ROM Drips range calls from an oracle M68K PC trace."""

import argparse
import json
from pathlib import Path


# Drips.asm loc_333F6/loc_3345C/loc_33516 call kid.asm Object_CheckInRange.
CALLS = {0x333FA, 0x33460, 0x3351A}
ENTRY = 0x32054
CLOSE = 0x3205C
NORMAL = 0x320B8
TARGETS = sorted(CALLS | {ENTRY, CLOSE, NORMAL})


def classify(rows):
    pending = None
    calls = []
    for row in rows:
        pc = row["pc"]
        if pc in CALLS:
            if pending is not None:
                raise ValueError(f"incomplete Drips call before frame {row['frame']}")
            pending = (row, False)
        elif pending is not None:
            call, entered = pending
            if row["frame"] != call["frame"] or row["address_registers"][3] != call["address_registers"][3]:
                raise ValueError(f"Drips call changed frame or A3 at frame {call['frame']}")
            if not entered and pc == ENTRY:
                pending = (call, True)
            elif entered and pc in (CLOSE, NORMAL):
                calls.append({"frame": call["frame"], "call_pc": call["pc"],
                              "object_a3": call["address_registers"][3],
                              "branch": "close" if pc == CLOSE else "normal"})
                pending = None
            else:
                raise ValueError(f"unexpected PC {pc:#x} after Drips call at frame {call['frame']}")
    if pending is not None:
        raise ValueError(f"incomplete Drips call at frame {pending[0]['frame']}")
    return calls


def summarise(calls):
    branches = [call["branch"] for call in calls]
    transitions = [f"{before}->{after}" for before, after in zip(branches, branches[1:])
                   if before != after]
    close = [call for call in calls if call["branch"] == "close"]
    return {
        "drips_calls": len(calls),
        "normal_calls": len(calls) - len(close),
        "close_calls": len(close),
        "transitions": transitions,
        "first_close": close[0] if close else None,
        "interleaved_threshold_witness":
            "normal->close" in transitions and "close->normal" in transitions,
    }


def read_trace(path):
    for line in path.read_text().splitlines():
        if line.startswith("m68k_pc_trace="):
            yield json.loads(line.split("=", 1)[1])


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    targets = ",".join(f"0x{pc:X}" for pc in TARGETS)
    parser.add_argument("trace", type=Path,
                        help=f"oracle-probe output captured with --m68k-pc-trace {targets}")
    args = parser.parse_args()
    calls = classify(read_trace(args.trace))
    summary = summarise(calls)
    print(json.dumps(summary))
    return 0 if summary["interleaved_threshold_witness"] else 1


if __name__ == "__main__":
    raise SystemExit(main())
