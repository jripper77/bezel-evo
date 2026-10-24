"""Check attribution and time windows before trusting the comparison report."""
import unittest
from analyze import console_rows, console_fps, distribution, summarize


class AnalysisTests(unittest.TestCase):
    def test_source_time_defines_windows_even_for_a_burst_in_the_pipe(self):
        frames = [{"CPUStartQPCTimeInMs": str(i * 1000 / 60), "MsBetweenPresents": str(1000 / 60),
                   "SwapChainAddress": "0x1"} for i in range(121)]
        result = console_fps(frames)
        self.assertEqual(result["count"], 5)
        self.assertAlmostEqual(result["median"], 60)

    def test_swap_chains_are_separate_streams(self):
        frames = []
        for chain, fps in [("0x1", 60), ("0x2", 30)]:
            frames.extend({"CPUStartQPCTimeInMs": str(i * 1000 / fps),
                           "MsBetweenPresents": str(1000 / fps),
                           "SwapChainAddress": chain} for i in range(fps * 2 + 1))
        result = console_fps(frames)
        self.assertAlmostEqual(result["min"], 30)
        self.assertAlmostEqual(result["max"], 60)

    def test_missing_or_nonfinite_data_does_not_turn_into_zero_fps(self):
        self.assertIsNone(distribution([])["median"])
        self.assertEqual(distribution([None, float("nan"), float("inf"), -1])["count"], 0)
        self.assertEqual(console_fps([])["count"], 0)

    def test_console_noise_and_other_processes_do_not_enter_target_results(self):
        samples = [{"csv": line} for line in ["starting capture",
            "Application,ProcessID,SwapChainAddress,TimeInSeconds,MsBetweenPresents",
            "game.exe,123,0x1,0.1,16.6", "browser.exe,456,0x2,0.2,2"]]
        self.assertEqual(len(console_rows(samples)), 2)
        result = summarize({"mode": "console", "pid": 123, "samples": samples, "resources": []})
        self.assertEqual(result["frames"], 1)
        self.assertEqual(result["other_pid_frames"], 1)
        self.assertIsNone(result["presented_fps"]["median"])

    def test_game_counters_allow_multiple_cores_and_report_foreground_coverage(self):
        rows = [
            {"elapsed": 0, "target_foreground": True,
             "target": {"cpu_seconds": 2, "rss_bytes": 2**20, "private_bytes": 2 * 2**20},
             "system_cpu": {"idle": 10, "total": 100}},
            {"elapsed": 2, "target_foreground": False,
             "target": {"cpu_seconds": 6, "rss_bytes": 2 * 2**20, "private_bytes": 4 * 2**20},
             "system_cpu": {"idle": 30, "total": 200}},
        ]
        result = summarize({"mode": "baseline", "resources": rows})
        self.assertEqual(result["resources"]["target"]["cpu_percent_one_core"], 200)
        self.assertEqual(result["resources"]["system_cpu_percent"], 80)
        self.assertEqual(result["target_foreground_percent"], 50)


if __name__ == "__main__":
    unittest.main()
