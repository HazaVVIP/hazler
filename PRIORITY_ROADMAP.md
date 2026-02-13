# Hazler Development Roadmap - Priority Summary
## Quick Reference for Next Version Development

**Current Version:** 0.1.0  
**Target Version:** 0.2.0  
**Timeline:** Q1-Q3 2026 (9 months)

---

## 🎯 Mission Statement

Transform Hazler from a "stable but ordinary" web crawler into a **top-tier security reconnaissance tool** that bug bounty hunters and penetration testers choose as their primary weapon.

---

## 🔥 P0 Features (Must-Have for 0.2.0) - Q1 2026

These are **essential** to compete with top-tier tools like Katana, Gospider, and Burp Spider.

### 1. Headless Browser Support ⭐⭐⭐⭐⭐
- **New Crate:** `hazler-browser`
- **Technology:** chromiumoxide or fantoccini
- **Why:** 90% of modern web apps are SPAs that require JavaScript execution
- **Effort:** 3 weeks
- **CLI:** `hazler https://app.com --headless`

### 2. Advanced WAF Evasion ⭐⭐⭐⭐⭐
- **Target Crate:** `hazler-http`
- **Features:** 
  - Realistic browser header rotation (Chrome, Firefox, Safari)
  - Request timing randomization
  - sec-ch-ua headers (Chrome fingerprint)
  - TLS fingerprint randomization (future)
- **Why:** Gets blocked by Cloudflare, Akamai in real-world scenarios
- **Effort:** 2 weeks
- **CLI:** `hazler https://target.com --stealth aggressive`

### 3. Tool Integration Formats ⭐⭐⭐⭐⭐
- **Target Crate:** `hazler-cli`
- **Formats:**
  - Nuclei template output
  - ffuf wordlist format
  - Burp Suite XML import
  - Pipeline mode (stdin/stdout)
- **Why:** Must integrate into existing security workflows
- **Effort:** 1 week
- **CLI:** 
  ```bash
  hazler https://target.com -o nuclei > template.yaml
  hazler https://target.com -o ffuf | ffuf -w - -u https://target.com/FUZZ
  cat urls.txt | hazler --pipeline | grep api
  ```

### 4. GraphQL Intelligence ⭐⭐⭐⭐
- **Target Crate:** `hazler-parser`
- **Features:**
  - Auto-detect GraphQL endpoints
  - Introspection query execution
  - Schema extraction and visualization
  - Sample query generation
- **Why:** GraphQL is ubiquitous in modern APIs
- **Effort:** 1 week
- **CLI:** `hazler https://api.com --graphql-introspect`

### 5. Source Map Parser ⭐⭐⭐⭐
- **Target Crate:** `hazler-js-parser`
- **Features:**
  - Auto-detect and download .map files
  - Extract original file paths
  - Reconstruct source code
  - Reveal project structure
- **Why:** Source maps expose sensitive internal structure
- **Effort:** 1 week
- **Output Example:**
  ```
  [INFO] Found source map: app.js.map
  [INFO] Project structure revealed:
    - src/admin/Dashboard.tsx
    - src/api/internal/users.ts
    - src/utils/secrets.ts
  ```

**Q1 Total Effort:** 8 weeks  
**Q1 Impact:** Makes Hazler competitive with top tools

---

## 🚀 P1 Features (High-Value Differentiators) - Q2 2026

These features will make Hazler **better than** competitors.

### 6. Response Diff Engine ⭐⭐⭐⭐
- **Target Crate:** `hazler-core` (new module: `differ.rs`)
- **Technology:** SimHash algorithm
- **Features:**
  - Compare responses over time
  - Detect content changes
  - Baseline storage
  - Change percentage calculation
- **Why:** Unique feature, great for monitoring targets
- **Effort:** 2 weeks
- **CLI:** 
  ```bash
  hazler https://target.com --save-baseline baseline.json
  hazler https://target.com --compare baseline.json
  ```

### 7. Entropy-Based Secret Detection ⭐⭐⭐⭐
- **Target Crate:** `hazler-secrets`
- **Algorithm:** Shannon entropy calculation
- **Features:**
  - Detect high-entropy strings (>4.5 bits)
  - Find unknown/custom API keys
  - Context extraction
  - Reduce false positives
- **Why:** Catches secrets missed by regex patterns
- **Effort:** 1 week
- **Output:**
  ```
  [HIGH] High-Entropy String (entropy: 4.87)
    Value: Xk7mP9...vB2nL [REDACTED]
    Context: const apiKey = "Xk7mP9...vB2nL";
  ```

### 8. Smart Fuzzing Module ⭐⭐⭐⭐
- **New Crate:** `hazler-fuzzer`
- **Features:**
  - Parameter discovery
  - Endpoint mutation (pluralization, extensions)
  - API version testing (v1, v2, v3)
  - BOLA/IDOR pattern hints
- **Why:** Proactive discovery vs. passive crawling
- **Effort:** 2 weeks
- **CLI:** `hazler https://api.com --fuzz aggressive`

### 9. Authentication Framework ⭐⭐⭐⭐
- **Target Crate:** `hazler-http`
- **Methods:**
  - Basic Auth
  - Bearer Token
  - Cookie-based
  - OAuth 2.0
  - Custom headers
- **Why:** Essential for crawling authenticated areas
- **Effort:** 2 weeks
- **CLI:** 
  ```bash
  hazler https://app.com --auth-bearer "eyJhbGc..."
  hazler https://app.com --auth-cookie "session=abc123"
  hazler https://app.com --auth-file credentials.json
  ```

### 10. Intelligent Rate Limiting ⭐⭐⭐
- **Target Crate:** `hazler-core`
- **Features:**
  - Per-domain adaptive rate limiting
  - Detect 429 responses
  - Auto-adjust concurrency
  - Circuit breaker pattern
  - Exponential backoff with jitter
- **Why:** Avoid bans while maximizing speed
- **Effort:** 1 week

**Q2 Total Effort:** 8 weeks  
**Q2 Impact:** Differentiation from competitors

---

## 🎨 P2 Features (Polish & Scale) - Q3 2026

### 11. Proxy Pool Manager
- **Target Crate:** `hazler-http`
- **Features:** Proxy rotation, health checks, SOCKS5/HTTP support
- **Effort:** 1 week

### 12. Crawl State Persistence
- **Target Crate:** `hazler-core`
- **Features:** Save/resume capability, SQLite/JSON storage
- **Effort:** 1 week

### 13. Diff Mode (CLI)
- **Target Crate:** `hazler-cli`
- **Features:** Compare two crawls, highlight changes
- **Effort:** 1 week

### 14. Watch Mode (CLI)
- **Target Crate:** `hazler-cli`
- **Features:** Continuous monitoring, scheduling, webhooks
- **Effort:** 1 week

### 15. Multi-Format Parser
- **Target Crate:** `hazler-parser`
- **Features:** XML/RSS, JSON API, sitemap.xml, robots.txt
- **Effort:** 1 week

### 16. JS Beautifier
- **Target Crate:** `hazler-js-parser`
- **Features:** Beautify minified JS for better analysis
- **Effort:** 1 week

### 17. Priority Queue
- **Target Crate:** `hazler-core`
- **Features:** Score URLs by interest (API > static assets)
- **Effort:** 1 week

### 18. Distributed Crawling
- **Target Crate:** `hazler-core`
- **Features:** Redis job queue, multiple workers, horizontal scaling
- **Effort:** 3 weeks

**Q3 Total Effort:** 10 weeks  
**Q3 Impact:** Production-ready, enterprise-scale

---

## 📊 Competitive Advantage Matrix

After implementing P0 + P1 features, Hazler will have:

| Feature | Hazler 0.2 | Katana | Gospider | Hakrawler | Burp |
|---------|-----------|--------|----------|-----------|------|
| Speed | ⭐⭐⭐⭐ | ⭐⭐⭐⭐ | ⭐⭐⭐ | ⭐⭐⭐⭐ | ⭐⭐ |
| Headless Browser | ✅ | ✅ | ❌ | ❌ | ✅ |
| Secret Detection | ⭐⭐⭐⭐ | ❌ | ❌ | ❌ | ⭐⭐⭐ |
| **Entropy Detection** | ✅ | ❌ | ❌ | ❌ | ❌ |
| **Response Diffing** | ✅ | ❌ | ❌ | ❌ | ⭐⭐ |
| **Smart Fuzzing** | ✅ | ⭐⭐ | ❌ | ❌ | ⭐⭐⭐ |
| GraphQL | ✅ | ✅ | ❌ | ❌ | ⭐⭐⭐ |
| Source Maps | ✅ | ❌ | ❌ | ❌ | ⭐⭐ |
| Tool Integration | ⭐⭐⭐⭐ | ⭐⭐⭐⭐ | ⭐⭐⭐ | ⭐⭐ | ⭐⭐⭐⭐ |
| Auth Handling | ✅ | ✅ | ❌ | ❌ | ⭐⭐⭐⭐ |

**Unique Advantages:**
1. ✨ **Entropy-based secret detection** - Nobody else has this
2. ✨ **Response diffing** - Rare feature, great for monitoring
3. ✨ **Source map parsing** - Often overlooked goldmine
4. ✨ **Integrated fuzzing** - Proactive, not just passive

---

## 🎯 Success Metrics

### Technical KPIs
- **Crawl Speed:** 200+ pages/sec (from 100 currently)
- **Discovery Rate:** +30% more endpoints than Katana
- **Secret False Positives:** <5%
- **WAF Bypass Rate:** >90%
- **Test Coverage:** >80%
- **Memory Usage:** <500MB for 10k pages

### Adoption KPIs
- **GitHub Stars:** 1000+ (from ~100)
- **Weekly Downloads:** 5000+ (`cargo install`)
- **Bug Bounty Mentions:** 50+ reports citing Hazler
- **Tool Integrations:** Used in 3+ popular workflows

### Quality KPIs
- **Crash Rate:** <0.1%
- **Issue Resolution Time:** <7 days (P1 bugs)
- **Documentation Coverage:** 100% of features

---

## 📋 Implementation Checklist

### Q1 2026 - Foundation (Weeks 1-8)

**Weeks 1-2: WAF Evasion & Integration**
- [ ] Implement browser header database (100+ User-Agents)
- [ ] Add sec-ch-ua headers for Chrome fingerprint
- [ ] Implement request timing randomization
- [ ] Create Nuclei output format
- [ ] Create ffuf output format
- [ ] Create Burp XML output format
- [ ] Implement pipeline mode (stdin/stdout)
- [ ] Write integration tests with real tools

**Weeks 3-5: Headless Browser**
- [ ] Create `hazler-browser` crate
- [ ] Integrate chromiumoxide
- [ ] Implement basic page loading
- [ ] Add request interception (XHR/Fetch)
- [ ] Add screenshot capability
- [ ] Integrate with main crawler
- [ ] Add CLI flags (--headless, --screenshot)
- [ ] Performance optimization
- [ ] Write comprehensive tests

**Weeks 6-7: Parser Enhancements**
- [ ] Implement GraphQL detection
- [ ] Build introspection query system
- [ ] Add schema extraction and parsing
- [ ] Implement source map detection
- [ ] Add .map file download logic
- [ ] Parse source maps and extract paths
- [ ] Integrate with JS parser
- [ ] Add verbose output for discoveries
- [ ] Write parser tests

**Week 8: Integration & Testing**
- [ ] End-to-end integration tests
- [ ] Performance benchmarks
- [ ] Bug fixes and polish
- [ ] Update documentation
- [ ] Create example workflows

### Q2 2026 - Differentiation (Weeks 9-16)

**Weeks 9-10: Response Diffing**
- [ ] Implement SimHash algorithm
- [ ] Create differ module
- [ ] Add baseline storage (JSON/SQLite)
- [ ] Implement comparison logic
- [ ] Add CLI flags (--save-baseline, --compare)
- [ ] Create diff visualization
- [ ] Write tests for edge cases

**Week 11: Entropy Detection**
- [ ] Implement Shannon entropy calculation
- [ ] Add high-entropy string detection
- [ ] Integrate with existing secret scanner
- [ ] Add context extraction
- [ ] Tune threshold values
- [ ] Reduce false positives
- [ ] Update tests

**Weeks 12-13: Smart Fuzzing**
- [ ] Create `hazler-fuzzer` crate
- [ ] Implement parameter discovery
- [ ] Add endpoint mutation logic
- [ ] Build API version testing
- [ ] Add BOLA/IDOR pattern detection
- [ ] Integrate with crawler
- [ ] Add CLI flags
- [ ] Write fuzzing tests

**Weeks 14-15: Authentication**
- [ ] Design auth framework
- [ ] Implement Basic Auth
- [ ] Implement Bearer Token
- [ ] Implement Cookie-based auth
- [ ] Add OAuth 2.0 support
- [ ] Create auth config file format
- [ ] Add session management
- [ ] Test with real authenticated sites

**Week 16: Rate Limiting & Retry**
- [ ] Implement exponential backoff
- [ ] Add circuit breaker pattern
- [ ] Create per-domain rate limiter
- [ ] Add 429 response detection
- [ ] Implement jitter in delays
- [ ] Auto-adjust concurrency
- [ ] Write resilience tests

### Q3 2026 - Polish (Weeks 17-26)
- [ ] All P2 features
- [ ] Performance optimization
- [ ] Memory profiling
- [ ] Comprehensive documentation
- [ ] Video tutorials
- [ ] Blog posts
- [ ] Community engagement

---

## 🚦 Decision Framework

When prioritizing work, use this framework:

### High Priority If:
- ✅ Competitors have it (table-stakes)
- ✅ Blocks major use cases
- ✅ Requested by multiple users
- ✅ Low effort, high impact
- ✅ Enables other features

### Medium Priority If:
- ⚠️ Nice-to-have, not essential
- ⚠️ Complex implementation
- ⚠️ Niche use case
- ⚠️ Can be worked around

### Low Priority If:
- ⏸️ Few users need it
- ⏸️ Very complex, uncertain value
- ⏸️ Better solved by external tools
- ⏸️ Maintenance burden

---

## 📚 Key Resources

### Development
- **Main Docs:** `AUDIT_AND_ROADMAP.md` - Strategic overview
- **Technical Details:** `TECHNICAL_RECOMMENDATIONS.md` - Code examples
- **This Doc:** Quick reference and checklist

### Research
- **Katana:** github.com/projectdiscovery/katana
- **Gospider:** github.com/jaeles-project/gospider
- **Hakrawler:** github.com/hakluke/hakrawler
- **Burp Suite:** portswigger.net/burp

### Community
- **GitHub Issues:** Feature requests and bugs
- **Discussions:** Design decisions
- **Discord:** Real-time collaboration (planned)

---

## 💡 Key Insights

### What Makes a Great Security Crawler?

1. **Speed:** Fast enough for large targets (100-200 pages/sec)
2. **Intelligence:** Smart detection (secrets, endpoints, vulnerabilities)
3. **Stealth:** Evade WAFs and detection mechanisms
4. **Integration:** Works with existing tools (Nuclei, ffuf, Burp)
5. **Completeness:** Handles modern tech (SPAs, GraphQL, APIs)
6. **Reliability:** Doesn't crash, handles errors gracefully

### Hazler's Unique Position

**Strengths to Amplify:**
- ✅ Rust speed and safety
- ✅ Already has secret detection
- ✅ Clean architecture for extensions
- ✅ Good documentation culture

**Gaps to Close:**
- ❌ No headless browser (critical!)
- ❌ Limited tool integration
- ❌ Basic WAF evasion
- ❌ Missing modern API support (GraphQL)

**Differentiators to Build:**
- ✨ Entropy-based detection
- ✨ Response diffing
- ✨ Integrated fuzzing
- ✨ Source map intelligence

---

## 🎬 Next Actions

### Immediate (This Week)
1. ✅ Review and approve this roadmap
2. ✅ Set up project board for tracking
3. ✅ Create feature branches for P0 items
4. ✅ Announce roadmap to community

### Week 1-2 (Starting Point)
1. 🔨 Start with WAF evasion (quick win)
2. 🔨 Implement tool integration formats
3. 🔨 Set up CI/CD for new tests
4. 📝 Write blog post: "Hazler 0.2 Roadmap"

### Month 1 Target
1. ✅ WAF evasion complete and tested
2. ✅ Tool integration (Nuclei, ffuf, Burp) working
3. ✅ Pipeline mode functional
4. 📊 First performance benchmarks published

---

## 🏆 Success Definition

**Hazler 0.2.0 will be successful when:**

1. A security professional can replace their current crawler with Hazler
2. Hazler appears in bug bounty reports as the primary recon tool
3. The community actively contributes features and bug fixes
4. Hazler is integrated into popular security workflows
5. Performance matches or exceeds top competitors
6. Documentation is comprehensive and examples are clear

**Ultimate Goal:** "The Go-To Recon Tool for Bug Bounty Hunters"

---

## 📞 Questions or Feedback?

- **GitHub Issues:** Technical questions, bugs, feature requests
- **Email:** [Maintainer email]
- **Discord:** [Coming soon]

**Let's make Hazler the best security crawler in the Rust ecosystem! 🦀🔥**
