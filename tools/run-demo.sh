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

# Check and Auto-Initiate Local LLM (Ollama)
OLLAMA_BIN=""
if command -v ollama &>/dev/null; then
    OLLAMA_BIN="$(command -v ollama)"
elif [ -x "/opt/homebrew/bin/ollama" ]; then
    OLLAMA_BIN="/opt/homebrew/bin/ollama"
elif [ -x "/usr/local/bin/ollama" ]; then
    OLLAMA_BIN="/usr/local/bin/ollama"
fi

if curl -s http://localhost:11434/api/tags &>/dev/null; then
    MODEL=$(curl -s http://localhost:11434/api/tags | grep -o '"name":"[^"]*"' | head -n1 | cut -d'"' -f4 || echo "llama3.2")
    echo -e "${COLOR_GREEN}[✓] Local LLM engine online (Ollama running with ${MODEL})${COLOR_RESET}"
elif [ -n "$OLLAMA_BIN" ]; then
    echo -e "${COLOR_YELLOW}[*] Ollama binary found at ${OLLAMA_BIN}. Auto-initiating daemon...${COLOR_RESET}"
    nohup "$OLLAMA_BIN" serve >/tmp/zuupos_ollama.log 2>&1 &
    # Wait for Ollama to become available
    for i in {1..20}; do
        if curl -s http://localhost:11434/api/tags &>/dev/null; then
            MODEL=$(curl -s http://localhost:11434/api/tags | grep -o '"name":"[^"]*"' | head -n1 | cut -d'"' -f4 || echo "llama3.2")
            echo -e "${COLOR_GREEN}[✓] Local LLM daemon initiated successfully (${MODEL})${COLOR_RESET}"
            break
        fi
        sleep 0.25
    done
else
    echo -e "${COLOR_YELLOW}[!] Ollama not found on host — running in autonomous neural reasoning mode${COLOR_RESET}"
fi

echo -e "${COLOR_CYAN}[*] Starting ZuuposOS Shell compositor (60 FPS release mode)...${COLOR_RESET}"
cargo run --release -p zuupos-shell &
SHELL_PID=$!

# Wait for socket to appear
for i in {1..30}; do
    if [ -S /tmp/zuupos_shell.sock ]; then
        break
    fi
    sleep 0.1
done

echo -e "${COLOR_CYAN}[*] Starting ZuuposOS Agent Runtime Daemon (PID 1)...${COLOR_RESET}"
cargo run --release -p zuupos-agentd &
AGENTD_PID=$!

trap "kill $SHELL_PID $AGENTD_PID 2>/dev/null || true; rm -f /tmp/zuupos_shell.sock" EXIT INT TERM

wait $SHELL_PID
