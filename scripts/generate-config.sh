#!/bin/bash

# Script pour générer automatiquement une configuration de test à partir d'un template_config.json
# Usage: ./scripts/generate-config.sh <template_type> [output_file]
# Exemples:
#   ./scripts/generate-config.sh library
#   ./scripts/generate-config.sh astro test-astro-config.yaml

set -e

# Couleurs pour l'affichage
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Variables par défaut
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"
TEMPLATES_DIR="$(cd "${PROJECT_ROOT}/../project-templates" && pwd)"

# Fonction d'affichage coloré
log_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

log_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

log_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Fonction d'aide
show_help() {
    echo "Usage: $0 <template_type> [output_file]"
    echo ""
    echo "Template types disponibles:"
    echo "  astro    - Template Astro (apps/astro)"
    echo "  library  - Template Library (packages/library)"
    echo ""
    echo "Arguments:"
    echo "  template_type  Type de template (obligatoire)"
    echo "  output_file    Nom du fichier de sortie (optionnel, par défaut: test-<template>-config.yaml)"
    echo ""
    echo "Exemples:"
    echo "  $0 library"
    echo "  $0 astro custom-config.yaml"
    exit 0
}

# Vérification des arguments
if [[ $# -eq 0 ]] || [[ "$1" == "-h" ]] || [[ "$1" == "--help" ]]; then
    show_help
fi

TEMPLATE_TYPE="$1"
OUTPUT_FILE="${2:-test-${TEMPLATE_TYPE}-config.yaml}"

# Mapping des types de templates vers leurs chemins
case "$TEMPLATE_TYPE" in
    "astro")
        TEMPLATE_PATH="$TEMPLATES_DIR/apps/astro"
        TEMPLATE_CATEGORY="apps"
        TEMPLATE_NAME="astro"
        ;;
    "library")
        TEMPLATE_PATH="$TEMPLATES_DIR/packages/library"
        TEMPLATE_CATEGORY="packages"
        TEMPLATE_NAME="library"
        ;;
    *)
        log_error "Type de template non reconnu: $TEMPLATE_TYPE"
        log_error "Types supportés: astro, library"
        exit 1
        ;;
esac

# Vérification de l'existence du template
TEMPLATE_CONFIG_FILE="$TEMPLATE_PATH/template_config.json"
if [[ ! -f "$TEMPLATE_CONFIG_FILE" ]]; then
    log_error "Fichier template_config.json non trouvé: $TEMPLATE_CONFIG_FILE"
    exit 1
fi

log_info "Analyse du template: $TEMPLATE_TYPE"
log_info "Fichier de configuration template: $TEMPLATE_CONFIG_FILE"

# Extraction des placeholders du template_config.json
log_info "Extraction des placeholders..."

# Utilisation de jq pour extraire tous les placeholders {{variable}}
PLACEHOLDERS=$(jq -r '
    [.. | strings] |
    map(match("\\{\\{([^}]+)\\}\\}"; "g")) |
    flatten |
    map(.captures[0].string) |
    unique |
    sort[]
' "$TEMPLATE_CONFIG_FILE" 2>/dev/null || {
    log_error "Erreur lors de l'analyse du fichier JSON. Vérifiez que jq est installé et que le JSON est valide."
    exit 1
})

if [[ -z "$PLACEHOLDERS" ]]; then
    log_warning "Aucun placeholder trouvé dans le template_config.json"
    exit 1
fi

log_success "Placeholders trouvés:"
echo "$PLACEHOLDERS" | while read -r placeholder; do
    echo "  - {{$placeholder}}"
done

# Génération du timestamp pour des noms uniques
TIMESTAMP=$(date +"%Y%m%d_%H%M%S")

# Création du fichier de configuration YAML
log_info "Génération du fichier de configuration: $OUTPUT_FILE"

cat > "$OUTPUT_FILE" << EOF
# Configuration générée automatiquement pour le template $TEMPLATE_TYPE
# Généré le $(date)
# Source: $TEMPLATE_CONFIG_FILE

# Configuration de base
template_category: "$TEMPLATE_CATEGORY"
template_name: "$TEMPLATE_NAME"

EOF

# Ajout des variables avec des valeurs par défaut intelligentes
echo "$PLACEHOLDERS" | while read -r placeholder; do
    case "$placeholder" in
        "project_name")
            echo "project_name: \"test-${TEMPLATE_TYPE}-${TIMESTAMP}\"" >> "$OUTPUT_FILE"
            ;;
        "project_description")
            echo "project_description: \"Projet de test généré automatiquement pour le template ${TEMPLATE_TYPE}\"" >> "$OUTPUT_FILE"
            ;;
        "project_author")
            echo "project_author: \"NextNodeSolutions <contact@nextnode.fr>\"" >> "$OUTPUT_FILE"
            ;;
        "project_license")
            echo "project_license: \"MIT\"" >> "$OUTPUT_FILE"
            ;;
        "project_version")
            echo "project_version: \"1.0.0\"" >> "$OUTPUT_FILE"
            ;;
        "project_keywords")
            case "$TEMPLATE_TYPE" in
                "astro")
                    cat >> "$OUTPUT_FILE" << EOF
project_keywords:
  - "astro"
  - "typescript"
  - "tailwind"
  - "test"
EOF
                    ;;
                "library")
                    cat >> "$OUTPUT_FILE" << EOF
project_keywords:
  - "typescript"
  - "library"
  - "nextnode"
  - "test"
EOF
                    ;;
                *)
                    cat >> "$OUTPUT_FILE" << EOF
project_keywords:
  - "test"
  - "${TEMPLATE_TYPE}"
EOF
                    ;;
            esac
            ;;
        "name")
            echo "name: \"@nextnode/test-${TEMPLATE_TYPE}-${TIMESTAMP}\"" >> "$OUTPUT_FILE"
            ;;
        "repository_url")
            echo "repository_url: \"https://github.com/NextNodeSolutions/test-${TEMPLATE_TYPE}-${TIMESTAMP}\"" >> "$OUTPUT_FILE"
            ;;
        "website_url")
            echo "website_url: \"https://test-${TEMPLATE_TYPE}-${TIMESTAMP}.fly.dev\"" >> "$OUTPUT_FILE"
            ;;
        "dev_domain")
            echo "dev_domain: \"test-${TEMPLATE_TYPE}-${TIMESTAMP}-dev.fly.dev\"" >> "$OUTPUT_FILE"
            ;;
        *)
            # Pour les variables non reconnues, utiliser une valeur par défaut générique
            echo "${placeholder}: \"${placeholder}_value\"" >> "$OUTPUT_FILE"
            ;;
    esac
done

# Ajout de variables optionnelles communes
cat >> "$OUTPUT_FILE" << EOF

# Variables optionnelles supplémentaires
author: "NextNodeSolutions"
license: "MIT"
version: "1.0.0"

# Configuration GitHub (pour mode remote)
# github_tag: "${TEMPLATE_CATEGORY}"
# create_develop_branch: true
EOF

log_success "Fichier de configuration généré: $OUTPUT_FILE"
log_info "Vous pouvez maintenant utiliser cette configuration avec:"
log_info "  cargo run -- --config $OUTPUT_FILE"
log_info ""
log_info "Ou modifier le fichier selon vos besoins avant de l'utiliser."

# Affichage du contenu généré
echo ""
log_info "Contenu du fichier généré:"
echo "----------------------------------------"
cat "$OUTPUT_FILE"
echo "----------------------------------------"