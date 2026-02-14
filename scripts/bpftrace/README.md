# Hazler eBPF/bpftrace Scripts

This directory contains eBPF/bpftrace scripts for advanced debugging, performance profiling, and security monitoring of Hazler.

## 🎯 Overview

eBPF (Extended Berkeley Packet Filter) allows safe, efficient kernel-level tracing and monitoring without modifying kernel source or loading kernel modules. These scripts provide deep insights into Hazler's runtime behavior.

## 📋 Prerequisites

### Install bpftrace

**Ubuntu/Debian:**
```bash
sudo apt update
sudo apt install -y bpftrace linux-headers-$(uname -r)
```

**Fedora/RHEL:**
```bash
sudo dnf install -y bpftrace kernel-devel
```

**Arch Linux:**
```bash
sudo pacman -S bpftrace linux-headers
```

### Requirements
- Linux kernel 4.9+ (5.x+ recommended)
- Root/sudo access
- bpftrace 0.9.0+
- Debug symbols (optional, for better tracing)

### Verify Installation
```bash
bpftrace --version
sudo bpftrace -e 'BEGIN { printf("bpftrace works!\n"); exit(); }'
```

## 🛠️ Available Scripts

### 1. `hazler-network.bt` - Network Monitor 🌐

Monitors all network activity including connections, DNS queries, and data transfer.

**Features:**
- TCP connection tracking
- DNS resolution monitoring  
- TLS/SSL handshake timing
- Data transfer statistics
- Connection duration histograms

**Usage:**
```bash
# Monitor existing Hazler process
sudo bpftrace hazler-network.bt

# Launch Hazler with monitoring
sudo bpftrace hazler-network.bt -c "hazler https://example.com"

# Save output to file
sudo bpftrace hazler-network.bt -c "hazler https://example.com" > network-trace.log
```

**Output Example:**
```
🔍 Hazler Network Monitor Started
TIME       PROCESS              EVENT                PID        DETAILS
523        hazler-cli           TCP_CONNECT          12345      IP: 93.184.216.34:443
524        hazler-cli           CONNECT_DONE         12345      Duration: 45 ms
525        hazler-cli           TLS_CONNECT          12345      Starting TLS handshake
670        hazler-cli           TLS_DONE             12345      TLS handshake: 145 ms
```

### 2. `hazler-perf.bt` - Performance Profiler ⚡

Profiles CPU usage, memory allocations, I/O operations, and concurrency.

**Features:**
- Memory allocation tracking (malloc/free)
- File I/O monitoring
- Thread creation tracking
- Lock contention detection
- Page fault monitoring
- CPU scheduling analysis

**Usage:**
```bash
sudo bpftrace hazler-perf.bt -c "hazler https://example.com -d 3"
```

**Use Cases:**
- Find memory leaks
- Identify performance bottlenecks
- Analyze I/O patterns
- Debug concurrency issues

### 3. `hazler-security.bt` - Security Monitor 🛡️

Monitors security-relevant events and detects suspicious patterns.

**Features:**
- Suspicious port detection
- Sensitive file access monitoring
- Process execution tracking
- Privilege escalation detection
- SSL certificate validation
- Data exfiltration detection
- High connection rate alerts

**Usage:**
```bash
sudo bpftrace hazler-security.bt -c "hazler https://example.com"
```

**Security Checks:**
- ⚠️ Connections to suspicious ports (31337, 12345, etc.)
- ⚠️ Access to /etc/passwd, /etc/shadow
- ⚠️ Process execution (execve)
- ⚠️ Privilege escalation (setuid/setgid)
- ⚠️ Large data transfers (> 10MB)
- ⚠️ SSL verification failures

### 4. `hazler-http.bt` - HTTP Tracer 🌐

Traces HTTP-level operations and timing.

**Features:**
- Request/response tracking
- HTTP timing analysis
- Response size monitoring
- Timeout detection
- Request rate analysis

**Usage:**
```bash
sudo bpftrace hazler-http.bt -c "hazler https://api.example.com"
```

## 📊 Example Workflows

### Debug Performance Issues

```bash
# Run performance profiler
sudo bpftrace hazler-perf.bt -c "hazler https://slowsite.com -d 5" > perf-analysis.txt

# Analyze results
grep "LARGE_ALLOC" perf-analysis.txt  # Find large allocations
grep "SLOW_LOCK" perf-analysis.txt    # Find lock contention
```

### Monitor Network Activity

```bash
# Monitor all network connections
sudo bpftrace hazler-network.bt -c "hazler https://example.com --all"

# Focus on TLS handshakes
sudo bpftrace hazler-network.bt -c "hazler https://example.com" | grep TLS
```

### Security Audit

```bash
# Run security monitor
sudo bpftrace hazler-security.bt -c "hazler https://target.com -d 3" > security-audit.log

# Check for alerts
grep "⚠️" security-audit.log
grep "CRITICAL" security-audit.log
```

### Combined Monitoring

```bash
# Run all monitors in parallel (different terminals)
Terminal 1: sudo bpftrace hazler-network.bt
Terminal 2: sudo bpftrace hazler-perf.bt
Terminal 3: sudo bpftrace hazler-security.bt
Terminal 4: sudo bpftrace hazler-http.bt

# Then run Hazler
hazler https://example.com --all
```

## 🔧 Advanced Usage

### Custom Filters

Edit scripts to add custom filters:

```bash
# Only monitor specific domains
/comm == "hazler" && str(args->filename) =~ /example\.com/

# Only track large allocations
/arg0 > 10485760/  # > 10MB

# Filter by PID
/pid == 12345/
```

### Output Formatting

```bash
# JSON output
sudo bpftrace hazler-network.bt -f json

# Save to file with timestamps
sudo bpftrace hazler-network.bt | ts '[%Y-%m-%d %H:%M:%S]' > trace.log

# Real-time analysis with awk
sudo bpftrace hazler-network.bt | awk '/TLS_DONE/ { print "TLS:", $NF }'
```

### Performance Impact

eBPF has minimal overhead, but you can adjust sampling rates:

```bash
# Reduce sampling for production
hardware:cpu-cycles:10000000  # Sample every 10M cycles instead of 1M

# Disable expensive probes
# Comment out SSL tracing for better performance
```

## 🐛 Troubleshooting

### "Permission denied"
```bash
# Need root access
sudo bpftrace script.bt
```

### "Could not resolve symbol"
```bash
# Install debug symbols
sudo apt install libc6-dbg libssl-dev
```

### "BPF program too large"
```bash
# Simplify the script or increase limit
sudo sysctl -w net.core.bpf_jit_limit=1000000000
```

### "Kernel headers not found"
```bash
# Install headers for your kernel
sudo apt install linux-headers-$(uname -r)
```

## 📚 Understanding the Output

### Network Monitor

```
Connection Duration (ms):
[1, 2)           5 |@@@@                                            |
[2, 4)          12 |@@@@@@@@@@@@                                    |
[4, 8)          25 |@@@@@@@@@@@@@@@@@@@@@@@@@@                      |
[8, 16)         48 |@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@|
[16, 32)        32 |@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@                 |
```
Most connections complete in 8-16ms.

### Performance Profiler

```
Allocation sizes:
[0, 1)          120 |@@@@@                                           |
[1, 2)          240 |@@@@@@@@@@@@                                    |
[2, 4)          480 |@@@@@@@@@@@@@@@@@@@@@@@@                        |
[4, 8)          960 |@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@|
```
Most allocations are 4-8 bytes (typical for small objects).

### Security Monitor

```
Security Score: ✅ CLEAN
```
No security alerts detected.

## 🎓 Learning Resources

- [bpftrace Reference Guide](https://github.com/iovisor/bpftrace/blob/master/docs/reference_guide.md)
- [Linux eBPF Documentation](https://www.kernel.org/doc/html/latest/bpf/)
- [BPF Performance Tools Book](http://www.brendangregg.com/bpf-performance-tools-book.html)

## 🤝 Contributing

Want to add more eBPF scripts? Ideas:
- Browser-specific tracing for hazler-browser
- GraphQL query analysis
- Database connection monitoring
- Custom application-level tracing

## ⚖️ License

These scripts are part of Hazler and follow the same MIT license.

## ⚠️ Important Notes

1. **Root Access Required**: eBPF requires root/sudo privileges
2. **Production Use**: Test scripts in development before production use
3. **Performance**: eBPF has minimal overhead but still impacts performance
4. **Linux Only**: These scripts only work on Linux systems
5. **Privacy**: Scripts may capture sensitive data - handle responsibly

## 🚀 Quick Start

```bash
# 1. Install bpftrace
sudo apt install bpftrace

# 2. Run a simple network monitor
cd scripts/bpftrace
sudo bpftrace hazler-network.bt -c "hazler https://example.com"

# 3. Analyze the output
# Look for patterns, bottlenecks, or issues
```

Happy tracing! 🎯
