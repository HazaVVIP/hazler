# System Prompt: Saran Pengembangan Hazler

## Tujuan
Dokumen ini berisi system prompt dan saran pengembangan untuk meningkatkan kemampuan Hazler agar setidaknya dapat mengimbangi kemampuan katana dalam menemukan endpoint, khususnya melalui mesin regex yang lebih agresif dan normalisasi URL yang lebih kompleks.

---

## 1. Peningkatan Mesin Regex untuk Ekstraksi Endpoint JavaScript

### Masalah Saat Ini
Hazler saat ini hanya mengekstrak link dari elemen HTML standar (`<a href>`, `<link href>`, `<area href>`). Ini membatasi kemampuannya untuk menemukan endpoint yang didefinisikan dalam kode JavaScript.

### Saran Pengembangan

#### 1.1 Regex Pattern untuk JavaScript Endpoints

Tambahkan regex patterns yang lebih agresif untuk mendeteksi endpoint dalam file JavaScript:

```rust
// Pattern untuk URL dalam string JavaScript
const JS_URL_PATTERNS: &[&str] = &[
    // URL dalam quotes
    r#"["']https?://[^"'\s]+["']"#,
    r#"["'](/[a-zA-Z0-9/_\-\.]+)["']"#,
    
    // Fetch API calls
    r#"fetch\s*\(\s*["']([^"']+)["']"#,
    r#"fetch\s*\(\s*`([^`]+)`"#,
    
    // XMLHttpRequest
    r#"\.open\s*\(\s*["'][^"']*["']\s*,\s*["']([^"']+)["']"#,
    
    // Axios calls
    r#"axios\.(get|post|put|delete|patch)\s*\(\s*["']([^"']+)["']"#,
    
    // jQuery AJAX
    r#"\$\.ajax\s*\(\s*\{[^}]*url\s*:\s*["']([^"']+)["']"#,
    r#"\$\.(get|post)\s*\(\s*["']([^"']+)["']"#,
    
    // API endpoint definitions
    r#"(api|endpoint|url|path|route)\s*[:=]\s*["']([^"']+)["']"#,
    
    // Template literals
    r#"`/api/[^`]+`"#,
    r#"`https?://[^`]+`"#,
    
    // Relative paths in router configs
    r#"path\s*:\s*["']([^"']+)["']"#,
    r#"route\s*:\s*["']([^"']+)["']"#,
    
    // GraphQL endpoints
    r#"(graphql|gql)\s*["']([^"']+)["']"#,
    
    // WebSocket endpoints
    r#"(ws|wss)://[^"'\s]+"#,
    
    // JSON-RPC endpoints
    r#"rpc\s*:\s*["']([^"']+)["']"#,
];
```

#### 1.2 Implementasi JavaScript Parser Module

Buat modul baru `hazler-js-parser` untuk parsing JavaScript:

```rust
pub struct JavaScriptParser {
    patterns: Vec<regex::Regex>,
}

impl JavaScriptParser {
    pub fn new() -> Result<Self> {
        let patterns = JS_URL_PATTERNS
            .iter()
            .map(|p| regex::Regex::new(p))
            .collect::<Result<Vec<_>, _>>()?;
        
        Ok(Self { patterns })
    }
    
    pub fn extract_endpoints(&self, js_content: &str, base_url: &Url) -> Vec<Url> {
        let mut endpoints = HashSet::new();
        
        for pattern in &self.patterns {
            for cap in pattern.captures_iter(js_content) {
                // Extract URL from capture groups
                for i in 1..cap.len() {
                    if let Some(url_str) = cap.get(i) {
                        let url_str = url_str.as_str();
                        
                        // Try to resolve as absolute or relative URL
                        if let Ok(url) = self.normalize_and_resolve(url_str, base_url) {
                            endpoints.insert(url);
                        }
                    }
                }
            }
        }
        
        endpoints.into_iter().collect()
    }
    
    fn normalize_and_resolve(&self, url_str: &str, base_url: &Url) -> Result<Url> {
        // Remove quotes and backticks
        let cleaned = url_str.trim_matches(|c| c == '"' || c == '\'' || c == '`');
        
        // Handle template literals with variables
        let cleaned = self.replace_template_vars(cleaned);
        
        // Try absolute URL first
        if let Ok(url) = Url::parse(cleaned) {
            return Ok(url);
        }
        
        // Try relative URL
        base_url.join(cleaned)
    }
    
    fn replace_template_vars(&self, url: &str) -> String {
        // Replace ${var} with placeholder values for discovery
        url.replace("${", "{")
           .replace("}", "")
           // Common patterns
           .replace("{id}", "1")
           .replace("{userId}", "1")
           .replace("{uuid}", "00000000-0000-0000-0000-000000000000")
    }
}
```

---

## 2. Dukungan untuk File .frame

### Masalah Saat Ini
Hazler tidak mengenali atau memproses file `.frame` yang mungkin berisi definisi endpoint.

### Saran Pengembangan

#### 2.1 Frame File Parser

Tambahkan dukungan untuk parsing file `.frame`:

```rust
pub struct FrameFileParser {
    js_parser: JavaScriptParser,
}

impl FrameFileParser {
    pub fn new() -> Result<Self> {
        Ok(Self {
            js_parser: JavaScriptParser::new()?,
        })
    }
    
    pub fn extract_endpoints(&self, frame_content: &str, base_url: &Url) -> Vec<Url> {
        let mut endpoints = Vec::new();
        
        // .frame files might contain JSON-like structures
        // Try to parse as JSON first
        if let Ok(json) = serde_json::from_str::<serde_json::Value>(frame_content) {
            endpoints.extend(self.extract_from_json(&json, base_url));
        }
        
        // Also apply JavaScript patterns
        endpoints.extend(self.js_parser.extract_endpoints(frame_content, base_url));
        
        endpoints
    }
    
    fn extract_from_json(&self, json: &serde_json::Value, base_url: &Url) -> Vec<Url> {
        let mut endpoints = Vec::new();
        
        match json {
            serde_json::Value::Object(map) => {
                for (key, value) in map {
                    // Look for keys that suggest URLs
                    if key.contains("url") || key.contains("endpoint") || 
                       key.contains("path") || key.contains("route") {
                        if let Some(url_str) = value.as_str() {
                            if let Ok(url) = base_url.join(url_str) {
                                endpoints.push(url);
                            }
                        }
                    }
                    // Recurse into nested objects
                    endpoints.extend(self.extract_from_json(value, base_url));
                }
            },
            serde_json::Value::Array(arr) => {
                for item in arr {
                    endpoints.extend(self.extract_from_json(item, base_url));
                }
            },
            _ => {}
        }
        
        endpoints
    }
}
```

#### 2.2 Content-Type Detection

Perbarui crawler untuk mendeteksi dan memproses file JavaScript dan .frame:

```rust
impl Crawler {
    async fn process_page(&self, url: Url, depth: usize) -> Result<Page> {
        let response = self.http_client.get(&url).await?;
        let content_type = response.content_type.clone();
        
        let mut links = Vec::new();
        
        match content_type.as_deref() {
            // Existing HTML processing
            Some(ct) if ct.contains("text/html") => {
                links = self.html_parser.extract_links(&response.body, &url)?;
            },
            
            // NEW: JavaScript file processing
            Some(ct) if ct.contains("javascript") || ct.contains("application/json") => {
                links = self.js_parser.extract_endpoints(&response.body, &url);
            },
            
            // NEW: Frame file processing (could be custom MIME type or extension-based)
            _ if url.path().ends_with(".frame") => {
                links = self.frame_parser.extract_endpoints(&response.body, &url);
            },
            
            _ => {}
        }
        
        // ... rest of processing
    }
}
```

---

## 3. Normalisasi URL yang Lebih Kompleks

### Masalah Saat Ini
Hazler melakukan normalisasi URL dasar (menghapus fragment). Untuk menemukan lebih banyak endpoint, diperlukan normalisasi yang lebih kompleks.

### Saran Pengembangan

#### 3.1 Advanced URL Normalizer

```rust
pub struct AdvancedUrlNormalizer;

impl AdvancedUrlNormalizer {
    /// Normalize URL dengan berbagai strategi
    pub fn normalize(&self, url: &Url) -> Vec<Url> {
        let mut variants = Vec::new();
        
        // 1. Base URL (current behavior)
        let mut normalized = url.clone();
        normalized.set_fragment(None);
        variants.push(normalized.clone());
        
        // 2. Remove trailing slash
        if let Some(path) = normalized.path().strip_suffix('/') {
            let mut no_slash = normalized.clone();
            no_slash.set_path(path);
            variants.push(no_slash);
        } else {
            // Add trailing slash
            let mut with_slash = normalized.clone();
            with_slash.set_path(&format!("{}/", normalized.path()));
            variants.push(with_slash);
        }
        
        // 3. Remove query parameters (discover base endpoint)
        if normalized.query().is_some() {
            let mut no_query = normalized.clone();
            no_query.set_query(None);
            variants.push(no_query);
        }
        
        // 4. Common file extensions
        let path = normalized.path();
        if !path.contains('.') {
            // Try common API extensions
            for ext in &["json", "xml", "html", "txt"] {
                let mut with_ext = normalized.clone();
                with_ext.set_path(&format!("{}.{}", path.trim_end_matches('/'), ext));
                variants.push(with_ext);
            }
        }
        
        // 5. Remove file extension (discover directory)
        if let Some(idx) = path.rfind('.') {
            if idx > path.rfind('/').unwrap_or(0) {
                let mut no_ext = normalized.clone();
                no_ext.set_path(&path[..idx]);
                variants.push(no_ext);
            }
        }
        
        // 6. Case variations (for case-insensitive servers)
        // Note: Only for discovery mode, not for deduplication
        
        // Deduplicate
        variants.sort_by(|a, b| a.as_str().cmp(b.as_str()));
        variants.dedup();
        
        variants
    }
    
    /// Generate common API path variations
    pub fn generate_api_variations(&self, url: &Url) -> Vec<Url> {
        let mut variants = Vec::new();
        let path = url.path();
        
        // If path looks like it might be an API endpoint
        if path.contains("/api/") || path.contains("/v1/") || path.contains("/v2/") {
            // Try different versions
            for version in &["v1", "v2", "v3"] {
                let versioned_path = path
                    .replace("/v1/", &format!("/{}/", version))
                    .replace("/v2/", &format!("/{}/", version))
                    .replace("/v3/", &format!("/{}/", version));
                
                if versioned_path != path {
                    let mut versioned = url.clone();
                    versioned.set_path(&versioned_path);
                    variants.push(versioned);
                }
            }
            
            // Try different formats
            for format in &["json", "xml", "yaml"] {
                let mut with_format = url.clone();
                if let Some(query) = url.query() {
                    with_format.set_query(Some(&format!("{}&format={}", query, format)));
                } else {
                    with_format.set_query(Some(&format!("format={}", format)));
                }
                variants.push(with_format);
            }
        }
        
        variants
    }
    
    /// Canonicalize URL for deduplication
    pub fn canonicalize(&self, url: &Url) -> String {
        let mut canonical = url.clone();
        
        // Remove fragment
        canonical.set_fragment(None);
        
        // Sort query parameters
        if let Some(query) = canonical.query() {
            let mut params: Vec<_> = query.split('&').collect();
            params.sort();
            canonical.set_query(Some(&params.join("&")));
        }
        
        // Lowercase scheme and host
        let mut result = canonical.scheme().to_lowercase();
        result.push_str("://");
        result.push_str(&canonical.host_str().unwrap_or("").to_lowercase());
        
        if let Some(port) = canonical.port() {
            // Only include port if non-standard
            let standard_port = (canonical.scheme() == "http" && port == 80) ||
                               (canonical.scheme() == "https" && port == 443);
            if !standard_port {
                result.push(':');
                result.push_str(&port.to_string());
            }
        }
        
        result.push_str(canonical.path());
        
        if let Some(query) = canonical.query() {
            result.push('?');
            result.push_str(query);
        }
        
        result
    }
}
```

#### 3.2 Integration dengan Crawler

```rust
impl Crawler {
    fn should_crawl(&self, url: &Url) -> bool {
        // Use canonicalized URL for deduplication
        let canonical = self.url_normalizer.canonicalize(url);
        
        if self.visited.contains(&canonical) {
            return false;
        }
        
        self.scope_validator.is_in_scope(url)
    }
    
    fn enqueue_url(&mut self, url: Url, depth: usize) {
        // For aggressive discovery, optionally enqueue variants
        if self.config.aggressive_discovery {
            for variant in self.url_normalizer.normalize(&url) {
                if self.should_crawl(&variant) {
                    self.queue.push(variant, depth);
                }
            }
        } else {
            if self.should_crawl(&url) {
                self.queue.push(url, depth);
            }
        }
    }
}
```

---

## 4. Perubahan Default: --exclude-body

### Masalah Saat Ini
Tanpa flag `--exclude-body`, output dari hazler dapat dibanjiri oleh konten HTML body yang besar, terutama saat crawling situs dengan banyak halaman. Ini membuat output sulit dibaca dan memakan banyak ruang disk/memory.

### Saran Pengembangan

#### 4.1 Ubah Default Behavior

Ubah CLI argument untuk membuat `--exclude-body` menjadi default:

```rust
#[derive(Parser, Debug)]
#[command(name = "hazler")]
struct Args {
    // ... other fields ...
    
    /// Include response body in output (default: excluded for performance)
    /// By default, body content is excluded to prevent terminal flooding
    /// Use this flag to include full HTML body content
    #[arg(long)]
    include_body: bool,  // Changed from exclude_body to include_body
    
    // ... other fields ...
}
```

Kemudian perbarui implementasi:

```rust
#[tokio::main]
async fn main() {
    let args = Args::parse();
    
    // ... setup code ...
    
    // Now exclude_body is the default, unless --include-body is specified
    let exclude_body = !args.include_body;  // Inverted logic
    let formatter = OutputFormatter::new(exclude_body, args.fields);
    
    // ... rest of main ...
}
```

#### 4.2 Update Documentation

Update README.md untuk mencerminkan perubahan default:

```markdown
## Options

  --include-body                   Include response body in output (excluded by default)
  --fields <FIELDS>                Select specific fields to output (comma-separated)
```

```markdown
## Examples

# Default: body excluded for clean output
hazler https://example.com

# Include body content if needed
hazler https://example.com --include-body

# Exclude body and select specific fields
hazler https://example.com --fields url,status_code,links
```

#### 4.3 Pesan Warning untuk Large Body

Tambahkan warning jika body dimasukkan dan ukurannya besar:

```rust
impl OutputFormatter {
    fn filter_page(&self, page: &Page) -> serde_json::Value {
        let mut data = json!({
            "url": page.url.as_str(),
            "status_code": page.status_code,
            "depth": page.depth,
        });

        if !self.exclude_body {
            // Warn if body is very large
            if page.body.len() > 100_000 {  // 100KB threshold
                eprintln!("Warning: Large body content for {} ({} bytes)", 
                         page.url, page.body.len());
            }
            data["body"] = json!(page.body);
        } else {
            // Optionally include body size instead
            data["body_size"] = json!(page.body.len());
        }

        // ... rest of the method
    }
}
```

---

## 5. Konfigurasi Mode Agresif

### Saran Tambahan

Tambahkan mode "aggressive" untuk discovery yang lebih dalam:

```rust
#[derive(Parser, Debug)]
struct Args {
    // ... existing fields ...
    
    /// Enable aggressive endpoint discovery mode
    /// - Applies regex patterns to JavaScript files
    /// - Generates URL variations
    /// - Discovers API endpoints more thoroughly
    /// Warning: This may generate more requests
    #[arg(long)]
    aggressive: bool,
}
```

```rust
impl Config {
    pub fn aggressive(mut self, enabled: bool) -> Self {
        self.aggressive_discovery = enabled;
        self
    }
}
```

---

## 6. Priority Implementation Order

Untuk implementasi bertahap, urutan prioritas yang disarankan:

1. **PENTING:** Ubah default `--exclude-body` → `--include-body` (paling mudah, dampak besar)
2. Tambahkan JavaScript regex patterns untuk endpoint discovery
3. Implementasi advanced URL normalization
4. Tambahkan dukungan untuk file .frame
5. Implementasi mode aggressive discovery
6. Optimasi dan testing

---

## 7. Testing Recommendations

### Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_js_endpoint_extraction() {
        let parser = JavaScriptParser::new().unwrap();
        let js = r#"
            fetch('/api/users');
            axios.get('/api/posts');
            const endpoint = '/api/comments';
        "#;
        let base = Url::parse("https://example.com").unwrap();
        let endpoints = parser.extract_endpoints(js, &base);
        
        assert!(endpoints.iter().any(|u| u.path() == "/api/users"));
        assert!(endpoints.iter().any(|u| u.path() == "/api/posts"));
        assert!(endpoints.iter().any(|u| u.path() == "/api/comments"));
    }
    
    #[test]
    fn test_url_normalization() {
        let normalizer = AdvancedUrlNormalizer;
        let url = Url::parse("https://example.com/path/?query=1#frag").unwrap();
        let variants = normalizer.normalize(&url);
        
        // Should generate multiple variants
        assert!(variants.len() > 1);
        assert!(variants.iter().any(|u| u.query().is_none()));
    }
}
```

### Integration Tests

```rust
#[tokio::test]
async fn test_javascript_crawling() {
    // Setup mock server with JS file containing endpoints
    let server = MockServer::start().await;
    
    Mock::given(method("GET"))
        .and(path("/app.js"))
        .respond_with(ResponseTemplate::new(200)
            .set_body_string("fetch('/api/users')"))
        .mount(&server)
        .await;
    
    let config = Config::new().aggressive(true);
    let crawler = Crawler::new(config).unwrap();
    
    let result = crawler.crawl(Url::parse(&server.uri()).unwrap()).await.unwrap();
    
    // Should discover the API endpoint from JS
    assert!(result.pages.iter().any(|p| p.url.path() == "/api/users"));
}
```

---

## 8. Performance Considerations

### Caching untuk Regex Compilation

```rust
use once_cell::sync::Lazy;
use regex::RegexSet;

static JS_PATTERNS: Lazy<RegexSet> = Lazy::new(|| {
    RegexSet::new(JS_URL_PATTERNS).expect("Failed to compile regex patterns")
});
```

### Parallel Processing

```rust
impl JavaScriptParser {
    pub fn extract_endpoints_parallel(&self, contents: &[(&str, &Url)]) -> Vec<Url> {
        use rayon::prelude::*;
        
        contents
            .par_iter()
            .flat_map(|(content, base)| self.extract_endpoints(content, base))
            .collect()
    }
}
```

---

## 9. Ringkasan Perubahan Utama

### Perubahan Wajib
1. ✅ **Default `--exclude-body`**: Ubah flag menjadi `--include-body` untuk menghindari terminal flooding
2. ✅ **JavaScript Regex Engine**: Implementasi regex patterns untuk menemukan endpoint di JS files
3. ✅ **URL Normalization**: Implementasi normalisasi URL yang lebih canggih

### Perubahan Opsional
4. 📝 Support untuk file `.frame`
5. 📝 Mode aggressive discovery
6. 📝 API version detection
7. 📝 WebSocket endpoint discovery

---

## 10. Benchmark Target

Setelah implementasi, hazler harus mampu:

- ✅ Menemukan endpoint tersembunyi dalam file JavaScript (min. 80% detection rate)
- ✅ Memproses file .frame dan mengekstrak endpoint
- ✅ Generate URL variants untuk discovery (min. 3-5 variants per URL)
- ✅ Output default tanpa body content (kecuali dengan `--include-body`)
- ✅ Maintain performance: < 50ms overhead per page untuk regex processing

---

## Kesimpulan

Dengan implementasi saran-saran di atas, Hazler akan memiliki kemampuan endpoint discovery yang jauh lebih kuat, mendekati atau bahkan melampaui tools seperti katana. Fokus utama adalah:

1. **Regex engine yang agresif** untuk JavaScript analysis
2. **Normalisasi URL kompleks** untuk menemukan lebih banyak variants
3. **Default exclude body** untuk user experience yang lebih baik
4. **Support untuk berbagai format file** termasuk .frame

Implementasi dapat dilakukan secara bertahap, dimulai dari perubahan paling mudah (exclude-body default) hingga fitur yang lebih kompleks (JavaScript parser).
