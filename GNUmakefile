# GNUmakefile — the Unix install contract for rlg's command-line tools.
#
#   make                          build the release binaries
#   make test                     run the test suite (from Makefile)
#   make install                  install to $(DESTDIR)$(PREFIX)
#   make uninstall                remove what install put there
#
# PREFIX defaults to /usr/local and DESTDIR to nothing, so
# `make DESTDIR=/tmp/stage install` stages a tree for a package.
# Manpages and shell completions are generated from the binaries' own
# CLI definitions at install time; nothing generated is committed.
#
# GNU make reads this file before Makefile; the include below keeps
# every development target (verify, lint, demo, ...) available.

.DEFAULT_GOAL := release

include Makefile

PREFIX     ?= /usr/local
BINDIR     ?= $(PREFIX)/bin
MANDIR     ?= $(PREFIX)/share/man/man1
BASHDIR    ?= $(PREFIX)/share/bash-completion/completions
ZSHDIR     ?= $(PREFIX)/share/zsh/site-functions
FISHDIR    ?= $(PREFIX)/share/fish/vendor_completions.d

BINARIES   := rlg rlg-report rlg-mcp
# The binaries whose CLI definitions generate manpages and completions.
CLI_TOOLS  := rlg rlg-report
RELEASE    := target/release

.PHONY: release install uninstall
release: ## Build the release binaries (rlg, rlg-report, rlg-mcp).
	cargo build --release --locked -p rlg-cli -p rlg-report -p rlg-mcp

install: release ## Install binaries, manpages and completions under PREFIX.
	mkdir -p "$(DESTDIR)$(BINDIR)" "$(DESTDIR)$(MANDIR)" \
	  "$(DESTDIR)$(BASHDIR)" "$(DESTDIR)$(ZSHDIR)" "$(DESTDIR)$(FISHDIR)"
	for bin in $(BINARIES); do \
	  install -m 0755 "$(RELEASE)/$$bin" "$(DESTDIR)$(BINDIR)/$$bin"; \
	done
	for bin in $(CLI_TOOLS); do \
	  "$(RELEASE)/$$bin" --manpage > "$(DESTDIR)$(MANDIR)/$$bin.1"; \
	  "$(RELEASE)/$$bin" --completions bash > "$(DESTDIR)$(BASHDIR)/$$bin"; \
	  "$(RELEASE)/$$bin" --completions zsh > "$(DESTDIR)$(ZSHDIR)/_$$bin"; \
	  "$(RELEASE)/$$bin" --completions fish > "$(DESTDIR)$(FISHDIR)/$$bin.fish"; \
	done

uninstall: ## Remove what install put under PREFIX.
	for bin in $(BINARIES); do rm -f "$(DESTDIR)$(BINDIR)/$$bin"; done
	for bin in $(CLI_TOOLS); do \
	  rm -f "$(DESTDIR)$(MANDIR)/$$bin.1" "$(DESTDIR)$(BASHDIR)/$$bin" \
	    "$(DESTDIR)$(ZSHDIR)/_$$bin" "$(DESTDIR)$(FISHDIR)/$$bin.fish"; \
	done
