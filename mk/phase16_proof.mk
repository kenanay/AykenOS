# Phase-16 QEMU Fail-Closed Proof Test
# This makefile provides targets for running and validating the fail-closed proof

.PHONY: phase16-proof-test phase16-proof-build phase16-proof-run phase16-proof-audit

# Build kernel with BCIB proof test enabled
phase16-proof-build:
	@echo "========================================="
	@echo "Phase-16: Building fail-closed proof test"
	@echo "========================================="
	$(MAKE) clean-noimg
	$(MAKE) all AYKEN_PHASE16_BCIB_PROOF_TEST=1

# Run QEMU with proof test
phase16-proof-run: phase16-proof-build
	@echo "========================================="
	@echo "Phase-16: Running fail-closed proof in QEMU"
	@echo "========================================="
	@mkdir -p out/logs
	$(MAKE) qemu-run QEMU_TIMEOUT=10

# Audit proof results
phase16-proof-audit:
	@echo "========================================="
	@echo "Phase-16: Auditing fail-closed proof"
	@echo "========================================="
	@bash tools/audit_fail_closed_proof.sh

# Full proof test: build + run + audit
phase16-proof-test: phase16-proof-run phase16-proof-audit
	@echo "========================================="
	@echo "Phase-16: Fail-closed proof test complete"
	@echo "========================================="
