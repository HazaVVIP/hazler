# Audit Argparse/Fields Hazler v0.2.0

## Ringkasan Eksekutif

Hazler versi 0.1.0 saat ini memiliki **60+ argparse/fields** yang membuat CLI menjadi kompleks. Audit ini mengidentifikasi peluang untuk menggabungkan, menyederhanakan, dan mengoptimalkan arguments tanpa mengurangi fungsionalitas.

**Target Pengurangan:** Dari 60+ arguments → ~35-40 arguments dengan konsolidasi cerdas

**Prinsip Utama:** 
- ✅ Fungsionalitas tetap prioritas #1
- ✅ Konsolidasi yang meningkatkan UX dan mengurangi kompleksitas
- ✅ Backward compatibility di pertimbangkan
- ✅ Defaults yang masuk akal untuk use case umum

---

## Status Saat Ini: Inventaris Arguments

### 1. Core Crawling (6 arguments) ✅ **SUDAH OPTIMAL**
- `url` (required) - Target URL
- `-d, --max-depth` (default: 3)
- `-c, --concurrency` (default: 10)
- `-p, --max-pages` (default: 0/unlimited)
- `-t, --timeout` (default: 10s)
- `-u, --user-agent` (default: "Hazler/0.1.0")

**Status:** Tidak perlu perubahan - parameters ini adalah core functionality yang sering disesuaikan.

---

### 2. Output & Reporting (14 arguments) 🔴 **PERLU KONSOLIDASI**

#### Arguments Saat Ini:
- `-o, --output-format` (default: tree)
- `--include-body` (flag)
- `--fields` (comma-separated)
- `--stats` (flag)
- `--report` (flag)
- `--html-report FILE`
- `--pdf-report FILE`
- `--export-sqlite FILE`
- `--export-openapi FILE`
- `--export-postman FILE`
- `--webhook-slack URL`
- `--webhook-discord URL`
- `--webhook-url URL`
- `-v, --verbose` (flag)

#### Rekomendasi Konsolidasi:

- [x] **Gabungkan Report Generation (4 args → 1 arg)**
  - Dari: `--report`, `--html-report FILE`, `--pdf-report FILE`, `--export-sqlite FILE`
  - Ke: `--export TYPE:FILE` dimana TYPE = {summary, html, pdf, sqlite}
  - Contoh: `--export html:report.html --export pdf:report.pdf --export sqlite:data.db`
  - **Benefit:** Lebih konsisten, dapat multiple exports sekaligus
  - **Impact:** Mengurangi 3 arguments
  - **Status:** ✅ SELESAI - Implemented in v0.2.0
  - **Note:** Juga mencakup openapi dan postman untuk konsistensi (total: 6 args → 1 arg, pengurangan 5 arguments)

- [ ] **Gabungkan Webhook Options (3 args → 2 args)**
  - Dari: `--webhook-slack URL`, `--webhook-discord URL`, `--webhook-url URL`
  - Ke: `--webhook URL --webhook-type {slack|discord|generic}`
  - Default type: auto-detect dari URL pattern
  - Contoh: `--webhook https://hooks.slack.com/...` (auto-detect slack)
  - **Benefit:** Lebih extensible untuk webhook types baru
  - **Impact:** Mengurangi 2 arguments, auto-detection menghilangkan kebutuhan --webhook-type di banyak kasus

- [ ] **Pertimbangkan gabung --stats dan --report**
  - Opsi: `--report` otomatis include stats
  - Atau: `--report {brief|full}` dimana brief = stats only, full = full report
  - **Benefit:** Mengurangi kebingungan tentang perbedaan stats vs report
  - **Impact:** Mengurangi 1 argument

**Pengurangan Total:** 5-6 arguments → 8-9 arguments

---

### 3. Security & Detection (6 arguments) 🟡 **OPTIMASI MINOR**

#### Arguments Saat Ini:
- `--no-stealth` (flag, default: stealth ON)
- `--no-secrets` (flag, default: secrets ON)
- `--aggressive` (flag)
- `--all` (flag)
- `--graphql-introspect` (flag)
- `--no-source-maps` (flag, default: maps ON)

#### Rekomendasi Konsolidasi:

- [ ] **Pertimbangkan --disable untuk boolean flags negatif**
  - Dari: `--no-stealth`, `--no-secrets`, `--no-source-maps`
  - Ke: `--disable stealth,secrets,source-maps` (comma-separated, repeatable)
  - **Benefit:** Pattern konsisten untuk disabling features
  - **Trade-off:** Sedikit lebih verbose untuk single disable
  - **Alternative:** Tetap terpisah karena ini adalah frequently-used flags
  
- [ ] **Pertimbangkan gabung --aggressive dan --all**
  - Opsi 1: `--mode {default|aggressive|comprehensive}` 
    - default = normal crawl
    - aggressive = current --aggressive
    - comprehensive = current --all
  - Opsi 2: Tetap terpisah karena use case berbeda
  - **Rekomendasi:** TETAP TERPISAH - use case jelas berbeda
  
- [ ] **--graphql-introspect bisa masuk ke --all atau --aggressive**
  - Jika user pakai `--all`, GraphQL introspect otomatis enabled
  - Tetap bisa di-enable independently dengan `--graphql-introspect`
  - **Benefit:** Mengurangi flag yang perlu diingat untuk comprehensive scan

**Pengurangan Total:** Minimal (0-1 arguments) - area ini sudah cukup optimal

---

### 4. Domain Scoping (2 arguments) 🟡 **BISA DISEDERHANAKAN**

#### Arguments Saat Ini:
- `--strict-domain` (flag)
- `--subs` (flag)

#### Rekomendasi Konsolidasi:

- [ ] **Gabung ke single scope argument**
  - Dari: `--strict-domain`, `--subs` (mutually exclusive)
  - Ke: `--scope {strict|same-domain|subdomains}`
  - Default: `same-domain` (current behavior without flags)
  - **Benefit:** Lebih jelas, menghindari mutually exclusive flags
  - **Impact:** Mengurangi 1 argument

**Pengurangan Total:** 2 arguments → 1 argument

---

### 5. Browser & JavaScript (3 arguments) ✅ **SUDAH OPTIMAL**

#### Arguments Saat Ini:
- `--browser` (flag)
- `--screenshot-path PATH` (optional)
- `--disable-images` (flag)

**Status:** Tidak perlu perubahan - relatif sedikit dan logical grouping sudah baik.

---

### 6. Fuzzing (4 arguments) 🔴 **PERLU KONSOLIDASI**

#### Arguments Saat Ini:
- `--fuzz` (flag - smart fuzzing)
- `--fuzz-params` (flag)
- `--fuzz-endpoints` (flag)
- `--fuzz-level {minimal|default|aggressive}`

#### Rekomendasi Konsolidasi:

- [ ] **Konsolidasi fuzzing options**
  - **Opsi 1:** Gabung ke `--fuzz MODE` dimana MODE = {off|smart|params|endpoints|full}
    - smart = current --fuzz
    - params = parameter discovery
    - endpoints = endpoint paths
    - full = params + endpoints + smart
    - Tambah: `--fuzz-level` tetap terpisah untuk aggressiveness
  
  - **Opsi 2:** Single flag `--fuzz` dengan modifiers di --fuzz-level
    - `--fuzz-level {off|minimal|default|aggressive|full}`
    - minimal = basic variations only
    - default = smart fuzzing
    - aggressive = smart + params
    - full = all fuzzing modes
    
  - **Rekomendasi:** Opsi 2 lebih simple
  - Contoh: `--fuzz --fuzz-level aggressive` vs current `--fuzz --fuzz-params --fuzz-endpoints --fuzz-level aggressive`

**Pengurangan Total:** 4 arguments → 2 arguments

---

### 7. Response Analysis & Comparison (6 arguments) 🟡 **OPTIMASI MINOR**

#### Arguments Saat Ini:
- `--baseline FILE`
- `--compare FILE`
- `--diff-threshold` (default: 0.85)
- `--cluster-responses` (flag)
- `--cluster-algorithm` (default: kmeans)
- `--num-clusters` (default: 5)

#### Rekomendasi Konsolidasi:

- [ ] **Clustering bisa disederhanakan**
  - Dari: `--cluster-responses` + `--cluster-algorithm` + `--num-clusters`
  - Ke: `--cluster {off|auto|kmeans:N|dbscan:epsilon,minpts}`
  - Default: off
  - auto = automatic algorithm selection based on data
  - Contoh: `--cluster kmeans:10` atau `--cluster auto`
  - **Benefit:** Self-documenting, mengurangi arguments
  - **Impact:** 3 arguments → 1 argument

- [ ] **Baseline/compare sudah optimal** - tidak perlu perubahan

**Pengurangan Total:** 6 arguments → 4 arguments

---

### 8. Reliability & Recovery (7 arguments) ✅ **SUDAH CUKUP OPTIMAL**

#### Arguments Saat Ini:
- `--resume FILE`
- `--auto-save SECS` (default: 60)
- `--max-retries` (default: 3)
- `--circuit-breaker` (flag)
- `--rate-limit` (default: 10)
- `--progress SECS` (default: 5)
- `--proxy URL`

#### Rekomendasi:

- [ ] **Pertimbangkan config file untuk advanced settings**
  - Parameters seperti `--auto-save`, `--max-retries`, `--rate-limit`, `--progress` bisa di config file
  - CLI flags override config file values
  - **Benefit:** CLI lebih clean untuk use case sederhana
  - **Trade-off:** Butuh documentasi config file format
  - **Rekomendasi:** TIDAK prioritas tinggi - values ini kadang perlu tweaking on-the-fly

**Pengurangan Total:** 0 arguments (tetap optimal seperti sekarang)

---

### 9. Authentication (14 arguments) 🔴 **PERLU KONSOLIDASI BESAR**

#### Arguments Saat Ini:
- `--auth-basic CREDS`
- `--auth-bearer TOKEN`
- `--auth-cookie` (repeatable)
- `--auth-header HEADER`
- `--auth-apikey KEY`
- `--auth-apikey-location` (default: header)
- `--auth-apikey-name` (default: X-API-Key)
- `--auth-oauth TOKEN`
- `--auth-file FILE`
- `--auth-form-url URL`
- `--auth-form-user-field` (default: username)
- `--auth-form-pass-field` (default: password)
- `--auth-form-username`
- `--auth-form-password`

#### Rekomendasi Konsolidasi:

- [ ] **Prioritaskan --auth-file sebagai primary method**
  - Auth file JSON lebih maintainable untuk complex auth
  - CLI options untuk quick/simple auth only
  
- [ ] **Simplifikasi CLI auth methods**
  - **Keep minimal CLI options:**
    - `--auth-file FILE` (primary, comprehensive)
    - `--auth METHOD:VALUE` (quick setup)
      - basic:user:pass
      - bearer:token
      - apikey:key
      - cookie:name=value
  - **Remove dari CLI (pindah ke --auth-file only):**
    - Form auth (complex, butuh multiple fields)
    - API key location/name customization
    - OAuth2 (complex refresh token flow)
  
- [ ] **Alternative: Unified --auth dengan method prefix**
  - `--auth basic:username:password`
  - `--auth bearer:token`
  - `--auth apikey:key@header:X-API-Key`
  - `--auth cookie:session=abc123`
  - `--auth file:auth.json`
  
**Pengurangan Total:** 14 arguments → 2-3 arguments untuk common cases

**Catatan:** Form auth dan OAuth advanced features masih available via `--auth-file`

---

## Ringkasan Rekomendasi Konsolidasi

### Prioritas Tinggi (High Impact, High Value)

- [ ] **P1: Konsolidasi Webhook (3→1)** 
  - `--webhook URL [--webhook-type TYPE]` dengan auto-detection
  - Saves: 2 arguments
  
- [ ] **P2: Konsolidasi Export/Report (4→1)**
  - `--export TYPE:FILE` (repeatable)
  - Saves: 3 arguments
  
- [ ] **P3: Simplifikasi Authentication (14→2-3)**
  - Focus on `--auth-file` untuk complex auth
  - `--auth METHOD:VALUE` untuk quick auth
  - Saves: 11-12 arguments

- [ ] **P4: Konsolidasi Fuzzing (4→2)**
  - `--fuzz` + `--fuzz-level {off|minimal|default|aggressive|full}`
  - Saves: 2 arguments

### Prioritas Menengah (Good Improvements)

- [ ] **P5: Domain Scope (2→1)**
  - `--scope {strict|same-domain|subdomains}`
  - Saves: 1 argument

- [ ] **P6: Clustering (3→1)**
  - `--cluster {off|auto|kmeans:N|dbscan:params}`
  - Saves: 2 arguments

- [ ] **P7: Stats + Report konsolidasi**
  - `--report {brief|full}` atau `--report` auto-include stats
  - Saves: 1 argument

### Sudah Optimal (No Changes Needed)

- ✅ Core crawling parameters (6 args)
- ✅ Browser & JavaScript (3 args)
- ✅ Reliability & Recovery (7 args)
- ✅ Baseline/compare functionality (2 args)

---

## Estimasi Pengurangan Total

| Kategori | Sebelum | Sesudah | Penghematan |
|----------|---------|---------|-------------|
| Core Crawling | 6 | 6 | 0 |
| Output & Reporting | 14 | 8-9 | 5-6 |
| Security & Detection | 6 | 5-6 | 0-1 |
| Domain Scoping | 2 | 1 | 1 |
| Browser & JS | 3 | 3 | 0 |
| Fuzzing | 4 | 2 | 2 |
| Response Analysis | 6 | 4 | 2 |
| Reliability | 7 | 7 | 0 |
| Authentication | 14 | 2-3 | 11-12 |
| **TOTAL** | **62** | **38-42** | **21-24** |

**Pengurangan: ~35-39% arguments dengan tetap mempertahankan 100% fungsionalitas**

---

## Implementation Plan

### Phase 1: Quick Wins (Low Risk, High Impact)
- [ ] Implementasi konsolidasi Webhook
- [ ] Implementasi konsolidasi Export/Report  
- [ ] Implementasi domain scope unification
- [ ] Implementasi clustering simplification

### Phase 2: Medium Complexity
- [ ] Implementasi fuzzing simplification
- [ ] Dokumentasi migration guide untuk users

### Phase 3: Complex Refactoring
- [ ] Implementasi authentication simplification
- [ ] Backward compatibility layer untuk deprecated flags
- [ ] Update semua examples dan documentation

### Phase 4: Polish & Release
- [ ] Update help text dan documentation
- [ ] Add warnings untuk deprecated options (dengan migration guide)
- [ ] Testing menyeluruh untuk semua code paths
- [ ] Release sebagai v0.2.0 dengan changelog lengkap

---

## Backward Compatibility Strategy

- [ ] **Deprecated arguments masih di-support dengan warnings**
  - Contoh: `--webhook-slack` → warning + auto-convert ke `--webhook URL --webhook-type slack`
  - Grace period: 2-3 releases sebelum removal
  
- [ ] **Environment variable fallbacks**
  - `HAZLER_AUTH_FILE` untuk `--auth-file`
  - `HAZLER_WEBHOOK_URL` untuk `--webhook`
  
- [ ] **Config file support untuk complex scenarios**
  - TOML/YAML config file: `~/.config/hazler/config.toml`
  - CLI args override config file
  - Mengurangi kebutuhan untuk banyak flags di CLI

---

## Risk Assessment

### Risiko Rendah ✅
- Webhook consolidation
- Export/report consolidation
- Domain scope unification
- Clustering simplification

### Risiko Menengah ⚠️
- Fuzzing simplification (butuh testing behavior changes)
- Authentication simplification (complex migration path)

### Mitigasi Risiko
- Comprehensive testing suite
- Migration guide dengan contoh before/after
- Deprecated warnings dengan helpful messages
- Keep old flags working dengan auto-conversion minimal 2 releases

---

## Success Metrics

- [ ] **Pengurangan arguments: Target 35-40% (21-24 arguments)**
- [ ] **No functionality loss: Semua use case tetap supported**
- [ ] **Improved UX: User survey/feedback positif**
- [ ] **Documentation clarity: Help text lebih mudah dipahami**
- [ ] **Backward compatibility: Deprecated flags tetap berfungsi dengan warnings**

---

## Catatan Tambahan

### Pertimbangan Config File
Untuk use case advanced dengan banyak konfigurasi, pertimbangkan:
```toml
# ~/.config/hazler/config.toml atau hazler.toml di project root

[crawl]
max_depth = 5
concurrency = 20
rate_limit = 10

[auth]
type = "bearer"
token = "${HAZLER_TOKEN}"  # environment variable support

[webhooks]
slack = "https://hooks.slack.com/..."
discord = "https://discord.com/..."

[fuzzing]
enabled = true
level = "aggressive"
params = true
endpoints = true
```

**Benefits:**
- CLI tetap clean untuk quick operations
- Complex configurations tidak perlu banyak flags
- Reusable configurations across projects
- Better for CI/CD pipelines

---

## Timeline Estimasi

- **Phase 1:** 1-2 minggu (quick wins)
- **Phase 2:** 1-2 minggu (medium complexity)
- **Phase 3:** 2-3 minggu (complex refactoring + testing)
- **Phase 4:** 1 minggu (documentation + polish)

**Total:** 5-8 minggu untuk complete implementation dengan testing menyeluruh

---

**Prepared by:** Copilot Agent  
**Date:** 2026-02-16  
**Version:** 1.0  
**Repository:** HazaVVIP/hazler  
**Target Release:** v0.2.0
