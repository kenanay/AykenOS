#!/usr/bin/env bash
# ==========================================
# Pre-CI Discipline Test Suite
# ==========================================
# Tests pre_ci_discipline.sh behavior using mock make commands.
# All properties from docs/specs/pre-ci-discipline/design.md are covered.
#
# Usage: bash scripts/ci/test_pre_ci_discipline.sh
# Exit:  0 = all tests pass, 1 = failure
# ==========================================

set -euo pipefail

PASS_COUNT=0
FAIL_COUNT=0
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
DISCIPLINE_SCRIPT="${SCRIPT_DIR}/pre_ci_discipline.sh"

# ---- Helpers ----

pass() { echo "  ✅ PASS: $1"; PASS_COUNT=$((PASS_COUNT + 1)); }
fail() { echo "  ❌ FAIL: $1"; echo "     $2"; FAIL_COUNT=$((FAIL_COUNT + 1)); }

# Run discipline script with mock make.
# ABI_EXIT, BOUNDARY_EXIT, HYGIENE_EXIT, CONSTITUTIONAL_EXIT control gate results.
run_with_mocks() {
    local abi_exit="${ABI_EXIT:-0}"
    local boundary_exit="${BOUNDARY_EXIT:-0}"
    local hygiene_exit="${HYGIENE_EXIT:-0}"
    local constitutional_exit="${CONSTITUTIONAL_EXIT:-0}"

    # Inject mock make into PATH via a temp dir
    local tmpdir
    tmpdir="$(mktemp -d)"
    cat > "${tmpdir}/make" <<MOCK
#!/usr/bin/env bash
case "\$1" in
  ci-gate-abi)            exit ${abi_exit} ;;
  ci-gate-boundary)       exit ${boundary_exit} ;;
  ci-gate-hygiene)        exit ${hygiene_exit} ;;
  ci-gate-constitutional) exit ${constitutional_exit} ;;
  *) exit 0 ;;
esac
MOCK
    chmod +x "${tmpdir}/make"

    local output exit_code
    output="$(PATH="${tmpdir}:${PATH}" EVIDENCE_ROOT="/tmp/test-evidence" bash "${DISCIPLINE_SCRIPT}" 2>&1)" || exit_code=$?
    exit_code="${exit_code:-0}"

    rm -rf "${tmpdir}"
    printf '%s\n' "${output}" "${exit_code}"
}

# Capture output and exit code separately
run_discipline() {
    local abi_exit="${ABI_EXIT:-0}"
    local boundary_exit="${BOUNDARY_EXIT:-0}"
    local hygiene_exit="${HYGIENE_EXIT:-0}"
    local constitutional_exit="${CONSTITUTIONAL_EXIT:-0}"

    local tmpdir
    tmpdir="$(mktemp -d)"
    cat > "${tmpdir}/make" <<MOCK
#!/usr/bin/env bash
case "\$1" in
  ci-gate-abi)            exit ${abi_exit} ;;
  ci-gate-boundary)       exit ${boundary_exit} ;;
  ci-gate-hygiene)        exit ${hygiene_exit} ;;
  ci-gate-constitutional) exit ${constitutional_exit} ;;
  *) exit 0 ;;
esac
MOCK
    chmod +x "${tmpdir}/make"

    DISCIPLINE_OUTPUT=""
    DISCIPLINE_EXIT=0
    DISCIPLINE_OUTPUT="$(PATH="${tmpdir}:${PATH}" EVIDENCE_ROOT="/tmp/test-evidence" bash "${DISCIPLINE_SCRIPT}" 2>&1)" || DISCIPLINE_EXIT=$?

    rm -rf "${tmpdir}"
}

# ==========================================
# Feature: pre-ci-discipline, Property 1: Kapı sırası değişmezi
# ==========================================
echo ""
echo "=== Property 1: Kapı Sırası Değişmezi ==="

# All pass — verify all 4 gate names appear in order
ABI_EXIT=0 BOUNDARY_EXIT=0 HYGIENE_EXIT=0 CONSTITUTIONAL_EXIT=0 run_discipline
abi_pos=$(echo "${DISCIPLINE_OUTPUT}" | grep -n "ABI Gate" | head -1 | cut -d: -f1)
boundary_pos=$(echo "${DISCIPLINE_OUTPUT}" | grep -n "Boundary Gate" | head -1 | cut -d: -f1)
hygiene_pos=$(echo "${DISCIPLINE_OUTPUT}" | grep -n "Hygiene Gate" | head -1 | cut -d: -f1)
constitutional_pos=$(echo "${DISCIPLINE_OUTPUT}" | grep -n "Constitutional Gate" | head -1 | cut -d: -f1)

if [ -n "${abi_pos}" ] && [ -n "${boundary_pos}" ] && [ -n "${hygiene_pos}" ] && [ -n "${constitutional_pos}" ] \
   && [ "${abi_pos}" -lt "${boundary_pos}" ] \
   && [ "${boundary_pos}" -lt "${hygiene_pos}" ] \
   && [ "${hygiene_pos}" -lt "${constitutional_pos}" ]; then
    pass "Kapı sırası ABI→Boundary→Hygiene→Constitutional"
else
    fail "Kapı sırası yanlış" "Beklenen: ABI < Boundary < Hygiene < Constitutional, Alınan pozisyonlar: ${abi_pos} ${boundary_pos} ${hygiene_pos} ${constitutional_pos}"
fi

# ==========================================
# Feature: pre-ci-discipline, Property 2: Fail-closed davranışı
# ==========================================
echo ""
echo "=== Property 2: Fail-Closed Davranışı ==="

# ABI fails → Boundary must NOT appear
ABI_EXIT=1 BOUNDARY_EXIT=0 HYGIENE_EXIT=0 CONSTITUTIONAL_EXIT=0 run_discipline
if echo "${DISCIPLINE_OUTPUT}" | grep -q "Boundary Gate"; then
    fail "ABI başarısız → Boundary çalışmamalı" "Boundary Gate çıktıda görüldü"
else
    pass "ABI başarısız → Boundary çalışmadı"
fi

# Boundary fails → Hygiene must NOT appear
ABI_EXIT=0 BOUNDARY_EXIT=1 HYGIENE_EXIT=0 CONSTITUTIONAL_EXIT=0 run_discipline
if echo "${DISCIPLINE_OUTPUT}" | grep -q "Hygiene Gate"; then
    fail "Boundary başarısız → Hygiene çalışmamalı" "Hygiene Gate çıktıda görüldü"
else
    pass "Boundary başarısız → Hygiene çalışmadı"
fi

# Hygiene fails → Constitutional must NOT appear
ABI_EXIT=0 BOUNDARY_EXIT=0 HYGIENE_EXIT=1 CONSTITUTIONAL_EXIT=0 run_discipline
if echo "${DISCIPLINE_OUTPUT}" | grep -q "Constitutional Gate"; then
    fail "Hygiene başarısız → Constitutional çalışmamalı" "Constitutional Gate çıktıda görüldü"
else
    pass "Hygiene başarısız → Constitutional çalışmadı"
fi

# Constitutional fails → exit 2, all 4 ran
ABI_EXIT=0 BOUNDARY_EXIT=0 HYGIENE_EXIT=0 CONSTITUTIONAL_EXIT=1 run_discipline
if echo "${DISCIPLINE_OUTPUT}" | grep -q "Constitutional Gate"; then
    pass "Constitutional başarısız → Constitutional çalıştı (son kapı)"
else
    fail "Constitutional başarısız → Constitutional görülmedi" ""
fi

# ==========================================
# Feature: pre-ci-discipline, Property 3: Başarısızlık çıkış kodu = 2
# ==========================================
echo ""
echo "=== Property 3: Başarısızlık Çıkış Kodu ==="

for gate_pos in "ABI" "Boundary" "Hygiene" "Constitutional"; do
    case "${gate_pos}" in
        ABI)            ABI_EXIT=1 BOUNDARY_EXIT=0 HYGIENE_EXIT=0 CONSTITUTIONAL_EXIT=0 run_discipline ;;
        Boundary)       ABI_EXIT=0 BOUNDARY_EXIT=1 HYGIENE_EXIT=0 CONSTITUTIONAL_EXIT=0 run_discipline ;;
        Hygiene)        ABI_EXIT=0 BOUNDARY_EXIT=0 HYGIENE_EXIT=1 CONSTITUTIONAL_EXIT=0 run_discipline ;;
        Constitutional) ABI_EXIT=0 BOUNDARY_EXIT=0 HYGIENE_EXIT=0 CONSTITUTIONAL_EXIT=1 run_discipline ;;
    esac
    if [ "${DISCIPLINE_EXIT}" -eq 2 ]; then
        pass "${gate_pos} başarısız → çıkış kodu 2"
    else
        fail "${gate_pos} başarısız → çıkış kodu 2 beklendi" "Alınan: ${DISCIPLINE_EXIT}"
    fi
done

# ==========================================
# Feature: pre-ci-discipline, Property 4: Başarısızlık çıktısı bütünlüğü
# ==========================================
echo ""
echo "=== Property 4: Başarısızlık Çıktısı Bütünlüğü ==="

# ABI failure: output must contain gate name + fail-closed + evidence path
ABI_EXIT=1 BOUNDARY_EXIT=0 HYGIENE_EXIT=0 CONSTITUTIONAL_EXIT=0 run_discipline

if echo "${DISCIPLINE_OUTPUT}" | grep -q "ABI Gate"; then
    pass "Başarısızlık çıktısı kapı adını içeriyor (ABI Gate)"
else
    fail "Başarısızlık çıktısı kapı adını içermiyor" "ABI Gate bulunamadı"
fi

if echo "${DISCIPLINE_OUTPUT}" | grep -q "fail-closed"; then
    pass "Başarısızlık çıktısı 'fail-closed' mesajını içeriyor"
else
    fail "Başarısızlık çıktısı 'fail-closed' içermiyor" ""
fi

if echo "${DISCIPLINE_OUTPUT}" | grep -q "evidence\|reports"; then
    pass "Başarısızlık çıktısı kanıt yolunu içeriyor"
else
    fail "Başarısızlık çıktısı kanıt yolunu içermiyor" ""
fi

# ==========================================
# Feature: pre-ci-discipline, Property 5: Başarı çıktısı bütünlüğü
# ==========================================
echo ""
echo "=== Property 5: Başarı Çıktısı Bütünlüğü ==="

ABI_EXIT=0 BOUNDARY_EXIT=0 HYGIENE_EXIT=0 CONSTITUTIONAL_EXIT=0 run_discipline

if [ "${DISCIPLINE_EXIT}" -eq 0 ]; then
    pass "Tüm kapılar geçti → çıkış kodu 0"
else
    fail "Tüm kapılar geçti → çıkış kodu 0 beklendi" "Alınan: ${DISCIPLINE_EXIT}"
fi

if echo "${DISCIPLINE_OUTPUT}" | grep -q "ALL GATES PASS"; then
    pass "Başarı çıktısı 'ALL GATES PASS' içeriyor"
else
    fail "Başarı çıktısı 'ALL GATES PASS' içermiyor" ""
fi

if echo "${DISCIPLINE_OUTPUT}" | grep -q "Real CI remains mandatory"; then
    pass "Başarı çıktısı 'Real CI remains mandatory' uyarısını içeriyor"
else
    fail "Başarı çıktısı CI uyarısını içermiyor" ""
fi

# Her kapı için PASS onayı
for gate in "ABI Gate" "Boundary Gate" "Hygiene Gate" "Constitutional Gate"; do
    if echo "${DISCIPLINE_OUTPUT}" | grep -q "PASS.*${gate}\|${gate}.*PASS\|✅ PASS"; then
        pass "PASS onayı mevcut: ${gate}"
    else
        fail "PASS onayı eksik: ${gate}" ""
    fi
done

# ==========================================
# Feature: pre-ci-discipline, Property 6: Deterministik yeniden üretilebilirlik
# ==========================================
echo ""
echo "=== Property 6: Deterministik Yeniden Üretilebilirlik ==="

ABI_EXIT=0 BOUNDARY_EXIT=0 HYGIENE_EXIT=0 CONSTITUTIONAL_EXIT=0 run_discipline
exit1="${DISCIPLINE_EXIT}"

ABI_EXIT=0 BOUNDARY_EXIT=0 HYGIENE_EXIT=0 CONSTITUTIONAL_EXIT=0 run_discipline
exit2="${DISCIPLINE_EXIT}"

if [ "${exit1}" -eq "${exit2}" ]; then
    pass "Aynı giriş → aynı çıkış kodu (${exit1} = ${exit2})"
else
    fail "Deterministik değil" "İlk çalıştırma: ${exit1}, İkinci: ${exit2}"
fi

ABI_EXIT=1 BOUNDARY_EXIT=0 HYGIENE_EXIT=0 CONSTITUTIONAL_EXIT=0 run_discipline
exit1="${DISCIPLINE_EXIT}"

ABI_EXIT=1 BOUNDARY_EXIT=0 HYGIENE_EXIT=0 CONSTITUTIONAL_EXIT=0 run_discipline
exit2="${DISCIPLINE_EXIT}"

if [ "${exit1}" -eq "${exit2}" ]; then
    pass "Başarısızlık senaryosu deterministik (${exit1} = ${exit2})"
else
    fail "Başarısızlık senaryosu deterministik değil" "İlk: ${exit1}, İkinci: ${exit2}"
fi

# ==========================================
# Feature: pre-ci-discipline, Property 7: Workspace mutasyon yasağı
# ==========================================
echo ""
echo "=== Property 7: Workspace Mutasyon Yasağı ==="

# Capture git status before and after
before="$(git status --porcelain 2>/dev/null | wc -l | tr -d ' ')"
ABI_EXIT=0 BOUNDARY_EXIT=0 HYGIENE_EXIT=0 CONSTITUTIONAL_EXIT=0 run_discipline
after="$(git status --porcelain 2>/dev/null | wc -l | tr -d ' ')"

if [ "${before}" -eq "${after}" ]; then
    pass "Workspace değişmedi (tracked dosya sayısı: ${before} → ${after})"
else
    fail "Workspace değişti" "Önce: ${before} değişiklik, Sonra: ${after} değişiklik"
fi

# ==========================================
# Feature: pre-ci-discipline, Property 1.2: Hook konfigürasyon doğrulama
# ==========================================
echo ""
echo "=== Property 1.2: Hook Konfigürasyon Doğrulama ==="

HOOK_FILE="${SCRIPT_DIR}/../../docs/hooks/pre-ci-discipline.kiro.hook"

if ! command -v jq >/dev/null 2>&1; then
    echo "  ⚠️  SKIP: jq bulunamadı, hook JSON doğrulaması atlandı"
else
    if [ ! -f "${HOOK_FILE}" ]; then
        fail "Hook dosyası bulunamadı" "${HOOK_FILE}"
    else
        enabled="$(jq -r '.enabled' "${HOOK_FILE}")"
        [ "${enabled}" = "true" ] \
            && pass "Hook enabled: true" \
            || fail "Hook enabled değil" "Beklenen: true, Alınan: ${enabled}"

        when_type="$(jq -r '.when.type' "${HOOK_FILE}")"
        [ "${when_type}" = "agentStop" ] \
            && pass "Hook when.type: agentStop" \
            || fail "Hook when.type yanlış" "Beklenen: agentStop, Alınan: ${when_type}"

        then_type="$(jq -r '.then.type' "${HOOK_FILE}")"
        [ "${then_type}" = "runCommand" ] \
            && pass "Hook then.type: runCommand" \
            || fail "Hook then.type yanlış" "Beklenen: runCommand, Alınan: ${then_type}"

        short_name="$(jq -r '.shortName' "${HOOK_FILE}")"
        [ "${short_name}" = "pre-ci-discipline" ] \
            && pass "Hook shortName: pre-ci-discipline" \
            || fail "Hook shortName yanlış" "Beklenen: pre-ci-discipline, Alınan: ${short_name}"

        workspace="$(jq -r '.workspaceFolderName' "${HOOK_FILE}")"
        [ "${workspace}" = "AykenOS" ] \
            && pass "Hook workspaceFolderName: AykenOS" \
            || fail "Hook workspaceFolderName yanlış" "Beklenen: AykenOS, Alınan: ${workspace}"

        SIM_FILE="${SCRIPT_DIR}/../../docs/hooks/ci-gate-simulation.kiro.hook"
        if [ -f "${SIM_FILE}" ]; then
            sim_name="$(jq -r '.shortName' "${SIM_FILE}")"
            [ "${sim_name}" != "${short_name}" ] \
                && pass "ci-gate-simulation shortName farklı (${sim_name} ≠ ${short_name})" \
                || fail "ci-gate-simulation ve pre-ci-discipline aynı shortName" "${sim_name}"
        fi
    fi
fi

# ==========================================
# Feature: pre-ci-discipline, Property 2.1: RUN_ID format doğrulama
# ==========================================
echo ""
echo "=== Property 2.1: RUN_ID Format Doğrulama ==="

ABI_EXIT=0 BOUNDARY_EXIT=0 HYGIENE_EXIT=0 CONSTITUTIONAL_EXIT=0 run_discipline
run_id="$(echo "${DISCIPLINE_OUTPUT}" | grep "RUN_ID:" | head -1 | sed 's/.*RUN_ID: *//')"

if echo "${run_id}" | grep -qE '^[0-9]{8}T[0-9]{6}Z-[0-9a-f]{7,}(-[0-9]+)?$'; then
    pass "RUN_ID formatı doğru: ${run_id}"
else
    fail "RUN_ID formatı yanlış" "Beklenen: YYYYMMDDTHHMMSSZ-<sha>, Alınan: '${run_id}'"
fi

ABI_EXIT=0 BOUNDARY_EXIT=0 HYGIENE_EXIT=0 CONSTITUTIONAL_EXIT=0 run_discipline
run_id2="$(echo "${DISCIPLINE_OUTPUT}" | grep "RUN_ID:" | head -1 | sed 's/.*RUN_ID: *//')"
sha1="$(echo "${run_id}"  | sed 's/^[^-]*-//' | cut -d- -f1)"
sha2="$(echo "${run_id2}" | sed 's/^[^-]*-//' | cut -d- -f1)"

if [ "${sha1}" = "${sha2}" ] && [ -n "${sha1}" ]; then
    pass "RUN_ID git SHA tutarlı: ${sha1}"
else
    fail "RUN_ID git SHA tutarsız" "İlk: ${sha1}, İkinci: ${sha2}"
fi

# ==========================================
# Sonuç
# ==========================================
echo ""
echo "=========================================="
echo "Test Sonuçları: ${PASS_COUNT} geçti, ${FAIL_COUNT} başarısız"
echo "=========================================="

if [ "${FAIL_COUNT}" -gt 0 ]; then
    exit 1
fi
exit 0
