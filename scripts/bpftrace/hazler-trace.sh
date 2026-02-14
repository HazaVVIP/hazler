#!/usr/bin/env bash

#
# Hazler eBPF Monitor Helper Script
# Simplifies running bpftrace scripts for Hazler debugging
#

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
BOLD='\033[1m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m' # No Color

show_banner() {
    echo -e "${BOLD}"
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo "  Hazler eBPF Monitor"
    echo "  Advanced debugging with bpftrace"
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo -e "${NC}"
}

check_requirements() {
    echo -e "${YELLOW}Checking requirements...${NC}"
    
    # Check if running as root
    if [ "$EUID" -ne 0 ]; then
        echo -e "${RED}Error: This script must be run as root (use sudo)${NC}"
        exit 1
    fi
    
    # Check if bpftrace is installed
    if ! command -v bpftrace &> /dev/null; then
        echo -e "${RED}Error: bpftrace is not installed${NC}"
        echo "Install with: sudo apt install bpftrace (Ubuntu/Debian)"
        exit 1
    fi
    
    # Check kernel version
    KERNEL_VERSION=$(uname -r | cut -d. -f1)
    if [ "$KERNEL_VERSION" -lt 4 ]; then
        echo -e "${RED}Warning: Kernel 4.9+ required for eBPF${NC}"
    fi
    
    echo -e "${GREEN}✓ All requirements met${NC}"
    echo ""
}

show_usage() {
    cat << EOF
${BOLD}Usage:${NC}
  sudo ./hazler-trace.sh <monitor> [hazler-args]

${BOLD}Available Monitors:${NC}
  network     - Monitor network connections, DNS, TLS
  perf        - Profile performance, memory, I/O
  security    - Security monitoring and alerts
  http        - HTTP request/response tracing
  all         - Run all monitors (separate terminals required)

${BOLD}Examples:${NC}
  # Monitor network activity
  sudo ./hazler-trace.sh network hazler https://example.com

  # Profile performance  
  sudo ./hazler-trace.sh perf hazler https://example.com -d 3

  # Security audit
  sudo ./hazler-trace.sh security hazler https://target.com --all

  # Save output to file
  sudo ./hazler-trace.sh network hazler https://example.com > trace.log

${BOLD}Tips:${NC}
  - Press Ctrl+C to stop monitoring
  - Use 'all' monitor in tmux or separate terminals
  - Check README.md for detailed documentation

EOF
}

run_monitor() {
    local monitor=$1
    shift
    local script="${SCRIPT_DIR}/hazler-${monitor}.bt"
    
    if [ ! -f "$script" ]; then
        echo -e "${RED}Error: Monitor script not found: $script${NC}"
        exit 1
    fi
    
    echo -e "${GREEN}Starting ${monitor} monitor...${NC}"
    echo -e "${YELLOW}Command: bpftrace $script -c \"$*\"${NC}"
    echo ""
    
    # Run bpftrace with the command
    if [ $# -gt 0 ]; then
        bpftrace "$script" -c "$*"
    else
        bpftrace "$script"
    fi
}

run_all_monitors() {
    local hazler_cmd="$*"
    
    echo -e "${YELLOW}All Monitors Mode${NC}"
    echo ""
    echo "This will start 4 monitors:"
    echo "  1. Network Monitor"
    echo "  2. Performance Profiler"
    echo "  3. Security Monitor"
    echo "  4. HTTP Tracer"
    echo ""
    echo -e "${YELLOW}You need to run these in separate terminals:${NC}"
    echo ""
    echo "Terminal 1:"
    echo "  sudo bpftrace ${SCRIPT_DIR}/hazler-network.bt"
    echo ""
    echo "Terminal 2:"
    echo "  sudo bpftrace ${SCRIPT_DIR}/hazler-perf.bt"
    echo ""
    echo "Terminal 3:"
    echo "  sudo bpftrace ${SCRIPT_DIR}/hazler-security.bt"
    echo ""
    echo "Terminal 4:"
    echo "  sudo bpftrace ${SCRIPT_DIR}/hazler-http.bt"
    echo ""
    echo "Then run Hazler:"
    echo "  $hazler_cmd"
    echo ""
}

main() {
    show_banner
    check_requirements
    
    if [ $# -lt 1 ]; then
        show_usage
        exit 1
    fi
    
    local monitor=$1
    shift
    
    case "$monitor" in
        network|perf|security|http)
            run_monitor "$monitor" "$@"
            ;;
        all)
            run_all_monitors "$@"
            ;;
        -h|--help|help)
            show_usage
            ;;
        *)
            echo -e "${RED}Error: Unknown monitor: $monitor${NC}"
            echo ""
            show_usage
            exit 1
            ;;
    esac
}

main "$@"
