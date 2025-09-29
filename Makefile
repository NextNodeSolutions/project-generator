.PHONY: format lint fix dev test clean config-astro config-library test-astro test-library clean-configs

# Auto-fix formatting and linting issues
fix: format lint

# Format code automatically
format:
	cargo fmt

# Lint and auto-fix clippy issues
lint:
	cargo clippy --fix --allow-dirty --allow-staged

# Development workflow - format, lint, then test
dev: fix test

# Run tests
test:
	cargo test

# Build release
build:
	cargo build --release

# Clean build artifacts
clean:
	cargo clean

# === Configuration Generation Commands ===

# Generate configuration for Astro template
config-astro:
	@echo "🚀 Génération de la configuration pour le template Astro..."
	./scripts/generate-config.sh astro

# Generate configuration for Library template
config-library:
	@echo "📦 Génération de la configuration pour le template Library..."
	./scripts/generate-config.sh library

# Generate config and test Astro template
test-astro: config-astro
	@echo "🧪 Test du template Astro avec la configuration générée..."
	cargo run -- --config test-astro-config.yaml

# Generate config and test Library template
test-library: config-library
	@echo "🧪 Test du template Library avec la configuration générée..."
	cargo run -- --config test-library-config.yaml

# Clean generated config files
clean-configs:
	@echo "🧹 Nettoyage des fichiers de configuration de test..."
	rm -f test-*-config.yaml
	@echo "✅ Fichiers de configuration nettoyés"

# Show available templates and configurations
show-templates:
	@echo "📋 Templates disponibles:"
	@echo "  - astro   (apps/astro)    : Application Astro avec TypeScript et Tailwind"
	@echo "  - library (packages/library) : Librairie TypeScript avec Vitest et Changesets"
	@echo ""
	@echo "🛠️  Commandes disponibles:"
	@echo "  make config-astro    : Générer config pour Astro"
	@echo "  make config-library  : Générer config pour Library"
	@echo "  make test-astro      : Générer config et tester Astro"
	@echo "  make test-library    : Générer config et tester Library"
	@echo "  make clean-configs   : Nettoyer les configs de test"