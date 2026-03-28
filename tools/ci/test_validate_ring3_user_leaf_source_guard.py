#!/usr/bin/env python3
"""Black-box tests for validate_ring3_user_leaf_source_guard.py."""

from __future__ import annotations

import json
import subprocess
import tempfile
import unittest
from pathlib import Path


GOOD_PROC = """
static uint64_t proc_alloc_user_image_frame(void)
{
    return phys_alloc_frame_high();
}

static uint64_t load_flat_image(uint64_t pml4_phys, const uint8_t *image, uint64_t size)
{
    uint64_t phys = proc_alloc_user_image_frame();
    return phys;
}

static uint64_t load_elf_image(uint64_t pml4_phys, const uint8_t *image, uint64_t size)
{
    uint64_t phys = proc_alloc_user_image_frame();
    return phys;
}
"""

BAD_PROC = """
static uint64_t proc_alloc_user_image_frame(void)
{
    return phys_alloc_frame();
}

static uint64_t load_flat_image(uint64_t pml4_phys, const uint8_t *image, uint64_t size)
{
    uint64_t phys = phys_alloc_frame();
    return phys;
}

static uint64_t load_elf_image(uint64_t pml4_phys, const uint8_t *image, uint64_t size)
{
    uint64_t phys = proc_alloc_user_image_frame();
    return phys;
}
"""

GOOD_SCHED = """
static int sched_capture_walk_snapshot(uint64_t root_phys, uint64_t va, void *out)
{
    uint64_t active_cr3;
    uint64_t kernel_cr3;
    uint64_t saved_rflags = 0;
    __asm__ volatile("mov %%cr3, %0" : "=r"(active_cr3));
    kernel_cr3 = paging_get_kernel_pml4_phys() & AYKEN_PTE_ADDR_MASK;
    __asm__ volatile("pushfq; popq %0" : "=r"(saved_rflags));
    __asm__ volatile("mov %0, %%cr3" :: "r"(kernel_cr3) : "memory");
    __asm__ volatile("mov %0, %%cr3" :: "r"(active_cr3) : "memory");
    __asm__ volatile("sti");
    return 1;
}
"""

BAD_SCHED = """
static int sched_capture_walk_snapshot(uint64_t root_phys, uint64_t va, void *out)
{
    return 0;
}
"""


class Ring3UserLeafSourceGuardTest(unittest.TestCase):
    def setUp(self) -> None:
        self.tmp = tempfile.TemporaryDirectory()
        self.root = Path(self.tmp.name)
        (self.root / "kernel/proc").mkdir(parents=True)
        (self.root / "kernel/sched").mkdir(parents=True)
        self.validator = Path(__file__).with_name("validate_ring3_user_leaf_source_guard.py")
        self.report = self.root / "report.json"
        self.violations = self.root / "violations.txt"

    def tearDown(self) -> None:
        self.tmp.cleanup()

    def _write_sources(self, proc_text: str, sched_text: str) -> None:
        (self.root / "kernel/proc/proc.c").write_text(proc_text, encoding="utf-8")
        (self.root / "kernel/sched/sched.c").write_text(sched_text, encoding="utf-8")

    def _run(self) -> tuple[int, dict]:
        proc = subprocess.run(
            [
                "python3",
                str(self.validator),
                "--source-root",
                str(self.root),
                "--out-report",
                str(self.report),
                "--violations-out",
                str(self.violations),
            ],
            check=False,
        )
        payload = json.loads(self.report.read_text(encoding="utf-8"))
        return proc.returncode, payload

    def test_pass_with_high_phys_and_kernel_safe_walk(self) -> None:
        self._write_sources(GOOD_PROC, GOOD_SCHED)
        rc, report = self._run()
        self.assertEqual(rc, 0)
        self.assertEqual(report.get("verdict"), "PASS")

    def test_fail_on_low_phys_allocator_and_missing_walk_safety(self) -> None:
        self._write_sources(BAD_PROC, BAD_SCHED)
        rc, report = self._run()
        self.assertEqual(rc, 2)
        self.assertIn("user_image_allocator_not_high_phys", report.get("violations", []))
        self.assertIn(
            "forbidden_low_phys_allocator_call:load_flat_image",
            report.get("violations", []),
        )
        self.assertIn(
            "walk_snapshot_not_kernel_cr3_safe:missing_kernel_root_lookup",
            report.get("violations", []),
        )


if __name__ == "__main__":
    unittest.main()
