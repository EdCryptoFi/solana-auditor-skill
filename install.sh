#!/bin/bash

# Solana Auditor Skill — Installer
# Installs into ~/.claude/skills/solana-auditor

set -e

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
WHITE='\033[1;37m'
MAGENTA='\033[0;35m'
NC='\033[0m'

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
SKILLS_DIR="$HOME/.claude/skills"
SKILL_PATH="$SKILLS_DIR/solana-auditor"
CLAUDE_MD_PATH="$HOME/.claude/CLAUDE.md"

print_banner() {
    echo ""
    echo -e "${RED}╔══════════════════════════════════════════════════════════════╗${NC}"
    echo -e "${RED}║${NC}                                                              ${RED}║${NC}"
    echo -e "${RED}║${NC}   ${WHITE}Solana Security Auditor Skill${NC}                             ${RED}║${NC}"
    echo -e "${RED}║${NC}   ${CYAN}Full-lifecycle audit: threat model → PoC → report${NC}        ${RED}║${NC}"
    echo -e "${RED}║${NC}                                                              ${RED}║${NC}"
    echo -e "${RED}║${NC}   ${YELLOW}by Superteam Brasil${NC}                                       ${RED}║${NC}"
    echo -e "${RED}╚══════════════════════════════════════════════════════════════╝${NC}"
    echo ""
}

print_help() {
    echo "Solana Auditor Skill — Installer"
    echo ""
    echo "Usage: ./install.sh [OPTIONS]"
    echo ""
    echo "Installs the solana-auditor skill into ~/.claude/skills/solana-auditor"
    echo "and updates ~/.claude/CLAUDE.md with the skill reference."
    echo ""
    echo "Options:"
    echo "  -y, --yes      Skip confirmation"
    echo "  -h, --help     Show this help"
    echo ""
}

SKIP_CONFIRM=false
while [[ $# -gt 0 ]]; do
    case $1 in
        -y|--yes) SKIP_CONFIRM=true; shift ;;
        -h|--help) print_help; exit 0 ;;
        *) echo "Unknown option: $1"; exit 1 ;;
    esac
done

print_banner

echo -e "${WHITE}This will install:${NC}"
echo -e "  ${BLUE}•${NC} solana-auditor skill  → ${CYAN}$SKILL_PATH${NC}"
echo -e "  ${BLUE}•${NC} CLAUDE.md update      → ${CYAN}$CLAUDE_MD_PATH${NC}"
echo ""

if [ "$SKIP_CONFIRM" = false ]; then
    read -p "Proceed? [Y/n] " -n 1 -r
    echo
    if [[ $REPLY =~ ^[Nn]$ ]]; then
        echo -e "${YELLOW}Cancelled${NC}"
        exit 0
    fi
fi

echo ""

# Create directories
mkdir -p "$SKILLS_DIR"
mkdir -p "$HOME/.claude"

# Install skill
echo -e "${CYAN}[1/2]${NC} Installing solana-auditor skill..."

if [ -d "$SKILL_PATH" ]; then
    echo -e "  ${YELLOW}→${NC} Removing existing installation"
    rm -rf "$SKILL_PATH"
fi

mkdir -p "$SKILL_PATH"
cp -r "$SCRIPT_DIR/skill" "$SKILL_PATH/"
cp -r "$SCRIPT_DIR/agents" "$SKILL_PATH/"
cp -r "$SCRIPT_DIR/commands" "$SKILL_PATH/"
cp -r "$SCRIPT_DIR/rules" "$SKILL_PATH/"
echo -e "  ${GREEN}✓${NC} Installed to $SKILL_PATH"

# Update CLAUDE.md
echo -e "${CYAN}[2/2]${NC} Updating CLAUDE.md..."

if [ -f "$CLAUDE_MD_PATH" ]; then
    # Check if already referenced
    if grep -q "solana-auditor" "$CLAUDE_MD_PATH" 2>/dev/null; then
        echo -e "  ${YELLOW}→${NC} solana-auditor already in CLAUDE.md, skipping"
    else
        # Append skill reference
        cat >> "$CLAUDE_MD_PATH" << 'EOF'

## Solana Security Auditor

Full-lifecycle Solana audit skill. Conducts the complete audit engagement.

- Skill: [~/.claude/skills/solana-auditor/skill/SKILL.md](~/.claude/skills/solana-auditor/skill/SKILL.md)
- Commands: `/audit-init`, `/audit-scan`, `/audit-report`
- Agents: `lead-auditor` (opus), `vuln-researcher` (sonnet), `report-writer` (sonnet)

Use when: auditing Solana/Anchor programs, reviewing instructions for vulnerabilities,
writing exploit PoCs, or generating professional security reports.
EOF
        echo -e "  ${GREEN}✓${NC} Added to $CLAUDE_MD_PATH"
    fi
else
    # Create CLAUDE.md from scratch
    cp "$SCRIPT_DIR/CLAUDE.md" "$CLAUDE_MD_PATH"
    echo -e "  ${GREEN}✓${NC} Created $CLAUDE_MD_PATH"
fi

# Done
echo ""
echo -e "${GREEN}╔══════════════════════════════════════════════════════════════╗${NC}"
echo -e "${GREEN}║${NC}  ${WHITE}Installation Complete!${NC}                                      ${GREEN}║${NC}"
echo -e "${GREEN}╚══════════════════════════════════════════════════════════════╝${NC}"
echo ""
echo -e "${WHITE}Installed:${NC}"
echo -e "  ${GREEN}✓${NC} Skill    ${CYAN}$SKILL_PATH${NC}"
echo -e "  ${GREEN}✓${NC} CLAUDE.md ${CYAN}$CLAUDE_MD_PATH${NC}"
echo ""
echo -e "${CYAN}Try asking Claude:${NC}"
echo -e "  ${BLUE}•${NC} \"/audit-init\" in your Solana project directory"
echo -e "  ${BLUE}•${NC} \"Audit this program for vulnerabilities\""
echo -e "  ${BLUE}•${NC} \"Review the withdraw instruction for missing signer checks\""
echo -e "  ${BLUE}•${NC} \"Generate an audit report for these findings\""
echo ""
echo -e "${MAGENTA}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "${YELLOW}           Powered by Superteam Brasil${NC}"
echo -e "${MAGENTA}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo ""
