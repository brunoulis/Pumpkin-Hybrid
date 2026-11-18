import unittest

from idle import parse_ticks


class TickReportTests(unittest.TestCase):
    def test_waits_for_complete_report(self):
        self.assertIsNone(parse_ticks("TPS: 20.0 MSPT: 0.02ms"))

    def test_reads_pumpkin_percentiles_and_rates(self):
        report = (
            "$ TPS: \x1b[92m20.0\x1b[0m MSPT: 0.45ms\n"
            "Percentiles: P50: 0.22ms P95: 1.50ms P99: 4.10ms. Sample: 100\n"
        )
        self.assertEqual(parse_ticks(report), {
            "tps": 20.0, "mspt": 0.45, "p50_ms": 0.22,
            "p95_ms": 1.50, "p99_ms": 4.10, "ticks": 100,
        })
