#!/usr/bin/env python3
"""Recompute the complete rejected experiment summary from saved data only."""
import gzip
import json
import math
from pathlib import Path
from statistics import median


def summarize():
    base = Path(__file__).resolve().parent
    report = json.loads(gzip.decompress((base / "comparison.json.gz").read_bytes()))
    assert report["succeeded"] and len(report["rows"]) == 24
    assert report["settings"]["trials"] == 21
    rows = []
    for row in report["rows"]:
        samples = row["samples"]
        assert set(samples) == {"js", "control", "candidate"}
        assert all(len(v) == 21 and all(math.isfinite(x) and x > 0 for x in v)
                   for v in samples.values())
        assert row["counts"]["control"] == row["counts"]["candidate"]
        for lane, values in samples.items():
            assert math.isclose(median(values), row["medianMs"][lane], rel_tol=1e-12)
        ratios = [c / b for c, b in zip(samples["candidate"], samples["control"])]
        assert math.isclose(row["candidateOverControl"],
                            median(samples["candidate"]) / median(samples["control"]),
                            rel_tol=1e-12)
        rows.append({
            "case": row["case"], "profile": row["profile"], "stage": row["stage"],
            "medianMs": row["medianMs"], "counts": row["counts"],
            "pairedMedianChangePercent": (median(ratios) - 1) * 100,
            "pairedWins": sum(x < 1 for x in ratios),
            "pairedTies": sum(x == 1 for x in ratios), "pairs": len(ratios),
            "ratioOfMediansChangePercent": (row["candidateOverControl"] - 1) * 100,
            "candidateOverJS": row["candidateOverJS"],
            "p95BatchMeanMs": row["p95BatchMeanMs"],
        })
    assert len({(r["case"], r["profile"], r["stage"]) for r in rows}) == 24
    er_parse = [r for r in rows if r["case"].startswith("er/") and r["stage"] == "parse"]
    assert len(er_parse) == 6 and all(r["pairedMedianChangePercent"] > 0 for r in er_parse)
    return {
        "decision": "rejected; dense-array control restored",
        "method": "Pair by trial index. Median(candidate/control - 1) differs from ratio of independent medians. Samples are batch means, not individual-operation latency.",
        "rows": rows, "rowCount": len(rows), "trialsPerLanePerRow": 21,
        "sampleCount": len(rows) * 21 * 3,
        "erParsePairedChangePercentRange": [
            min(r["pairedMedianChangePercent"] for r in er_parse),
            max(r["pairedMedianChangePercent"] for r in er_parse)],
        "erParseWinsRange": [min(r["pairedWins"] for r in er_parse),
                            max(r["pairedWins"] for r in er_parse)],
        "firstUse": "not run after clear ordinary-parse regression",
    }


if __name__ == "__main__":
    print(json.dumps(summarize(), indent=2))
