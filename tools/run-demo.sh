#!/usr/bin/env bash
# ZuuposOS — Host Demo Launcher (Phase 1)
# Runs the AI-Native Agentic Shell + Live LLM Agent Daemon
set -euo pipefail

COLOR_PURPLE='\033[0;35m'
COLOR_CYAN='\033[0;36m'
COLOR_GREEN='\033[0;32m'
COLOR_YELLOW='\033[1;33m'
COLOR_RESET='\033[0m'

echo -e "${COLOR_PURPLE}"
echo "╔══════════════════════════════════════════════════╗"
echo "║          Z U U P O S   O S   D E M O             ║"
echo "║        AI-Native Agentic Operating System        ║"
echo "╚══════════════════════════════════════════════════╝"
echo -e "${COLOR_RESET}"

# Sourcing cargo
if [ -f "$HOME/.cargo/env" ]; then
    source "$HOME/.cargo/env"
fi

# Clean up any stale sockets
rm -f /tmp/zuupos_shell.sock

# Check Ollama
if curl -s http://localhost:11434/api/tags &>/dev/null; then
    echo -e "${COLOR_GREEN}[✓] Local LLM engine online (Ollama detected)${COLOR_RESET}"
else
    echo -e "${COLOR_YELLOW}[!] Ollama not detected on localhost:11434 — running in autonomous neural reasoning mode${COLOR_RESET}"
fi

echo -e "${COLOR_CYAN}[*] Starting ZuuposOS Shell compositor...${COLOR_RESET}"
cargo run -p zuupos-shell &
SHELL_PID=$!

# Wait for socket to appear
for i in {1..30}; do
    if [ -S /tmp/zuupos_shell.sock ]; then
        break
    fi
    sleep 0.1
done

echo -e "${COLOR_CYAN}[*] Starting ZuuposOS Agent Runtime Daemon (PID 1)...${COLOR_RESET}"
cargo run -p zuupos-agentd &
AGENTD_PID=$!

trap "kill $SHELL_PID $AGENTD_PID 2>/dev/null || true; rm -f /tmp/zuupos_shell.sock" EXIT INT TERM

wait $SHELL_PID
