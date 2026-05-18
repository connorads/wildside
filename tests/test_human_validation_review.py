from __future__ import annotations

import sys
import unittest
from pathlib import Path
from tempfile import TemporaryDirectory
from unittest.mock import patch


REPO_ROOT = Path(__file__).resolve().parent.parent
sys.path.insert(0, str(REPO_ROOT / "scripts"))

import build_human_validation_review as review  # noqa: E402


class HumanValidationReviewTests(unittest.TestCase):
    def test_rel_from_output_supports_sibling_cache_paths(self) -> None:
        output_dir = REPO_ROOT / ".cache" / "human-validation"
        image_path = (
            REPO_ROOT
            / ".cache"
            / "recordings"
            / "issue-135-level1-background"
            / "verify"
            / "native"
            / "000000_start.png"
        )

        self.assertEqual(
            review.rel_from_output(image_path, output_dir),
            "../recordings/issue-135-level1-background/verify/native/000000_start.png",
        )

    def test_validate_inputs_reports_missing_images_and_transcripts(self) -> None:
        with TemporaryDirectory() as tmp:
            root = Path(tmp)
            image = root / "missing.png"
            transcript = root / "missing.txt"
            item = review.ReviewItem(
                backlog_row="#999",
                title="Missing bundle",
                verifier="verify",
                decision="decide",
                images=(review.ReviewImage(image, "Missing image"),),
                transcripts=(transcript,),
            )

            with patch.object(review, "REVIEW_ITEMS", (item,)):
                self.assertEqual(review.validate_inputs(), [image, transcript])

    def test_render_item_links_images_and_transcripts(self) -> None:
        output_dir = REPO_ROOT / ".cache" / "human-validation"
        image = (
            REPO_ROOT
            / ".cache"
            / "recordings"
            / "issue-140-prize-block-window"
            / "verify"
            / "native"
            / "000359_scenario_end.png"
        )
        transcript = (
            REPO_ROOT
            / ".cache"
            / "recordings"
            / "issue-140-prize-block-window"
            / "transcript.txt"
        )
        item = review.ReviewItem(
            backlog_row="#140",
            title="Prize block lifecycle art",
            verifier="cargo run -p kidcli -- verify-recording --slug issue-140-prize-block-window --native-only",
            decision="Inspect prize block art.",
            images=(review.ReviewImage(image, "Scenario end"),),
            transcripts=(transcript,),
        )

        html = review.render_item(item, output_dir)

        self.assertIn("#140 Prize block lifecycle art", html)
        self.assertIn(
            "../recordings/issue-140-prize-block-window/verify/native/000359_scenario_end.png",
            html,
        )
        self.assertIn(
            "../recordings/issue-140-prize-block-window/transcript.txt",
            html,
        )


if __name__ == "__main__":
    unittest.main()
