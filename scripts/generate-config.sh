#!/bin/bash

# Script pour générer automatiquement une configuration complète et dynamique
# Ce script analyse automatiquement :
#   1. La structure FileConfig du code Rust pour les champs système
#   2. Le template_config.json du template sélectionné pour les variables
#
# Usage: ./scripts/generate-config.sh [template_path] [output_file]
# Exemples:
#   ./scripts/generate-config.sh                    # Mode interactif
#   ./scripts/generate-config.sh apps/astro         # Template spécifique
#   ./scripts/generate-config.sh apps/astro config.yaml  # Avec fichier de sortie
#   ./scripts/generate-config.sh --list             # Liste tous les templates

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
FILE_CONFIG_PATH="$PROJECT_ROOT/src/config/file_config.rs"

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

# ========================================
# FONCTIONS D'EXTRACTION DYNAMIQUE
# ========================================

# Fonction pour extraire les champs de FileConfig depuis le code Rust
extract_fileconfig_fields() {
    log_info "Extraction des champs système depuis FileConfig..." >&2

    if [[ ! -f "$FILE_CONFIG_PATH" ]]; then
        log_error "Fichier FileConfig non trouvé: $FILE_CONFIG_PATH" >&2
        exit 1
    fi

    # Extraire tous les champs publics de la structure FileConfig
    awk '/^pub struct FileConfig/,/^}/' "$FILE_CONFIG_PATH" | \
    grep -E '^\s*pub\s+[a-z_]+:' | \
    grep -v "additional_vars" | \
    awk '{print $2}' | \
    sed 's/:.*$//' | \
    sort
}

# Fonction pour extraire les métadonnées d'un champ FileConfig
extract_field_metadata() {
    local field="$1"
    local is_optional="false"
    local field_type=""
    local valid_values=""

    # Vérifier si le champ est Option<> (donc optionnel)
    if grep "pub $field:" "$FILE_CONFIG_PATH" | grep -q "Option<"; then
        is_optional="true"
    fi

    # Extraire le type
    field_type=$(grep "pub $field:" "$FILE_CONFIG_PATH" | sed -E 's/.*:\s*(.+)$/\1/' | tr -d ',')

    # Extraire les valeurs valides pour github_tag
    if [[ "$field" == "github_tag" ]]; then
        valid_values=$(grep -A 5 "validate_github_tag" "$FILE_CONFIG_PATH" | \
                      grep -o '"[^"]*"' | tr -d '"' | paste -sd '|' -)
    fi

    echo "$is_optional|$field_type|$valid_values"
}

# Fonction pour extraire les variables depuis template_config.json
extract_template_variables() {
    local template_config_file="$1"

    log_info "Extraction des variables depuis template_config.json..." >&2

    if [[ ! -f "$template_config_file" ]]; then
        log_error "Fichier template_config.json non trouvé: $template_config_file" >&2
        exit 1
    fi

    # Extraire tous les noms de variables uniques
    jq -r '.[].replacements[].name | select(. != null)' "$template_config_file" 2>/dev/null | sort -u || {
        log_error "Erreur lors de l'analyse du fichier JSON. Vérifiez que jq est installé et que le JSON est valide." >&2
        exit 1
    }
}

# Fonction pour générer une valeur par défaut intelligente
generate_default_value() {
    local field="$1"
    local template_path="$2"
    local timestamp="$3"

    # Extraire le nom du template et la catégorie depuis le chemin
    local template_name=$(basename "$template_path")
    local template_category=$(dirname "$template_path" | sed "s|$TEMPLATES_DIR/||")

    case "$field" in
        "project_name")
            echo "test-${template_name}-${timestamp}"
            ;;
        "name")
            echo "@nextnode/test-${template_name}-${timestamp}"
            ;;
        "template_category")
            echo "$template_category"
            ;;
        "template_name")
            echo "$template_name"
            ;;
        "template_branch")
            echo "main"
            ;;
        "github_tag")
            echo "$template_category"
            ;;
        "create_develop_branch")
            echo "false"
            ;;
        "project_description"|"description")
            echo "Projet de test généré automatiquement pour le template ${template_name}"
            ;;
        "project_author"|"author")
            echo "NextNodeSolutions <contact@nextnode.fr>"
            ;;
        "project_version"|"version")
            echo "1.0.0"
            ;;
        "project_license"|"license")
            echo "MIT"
            ;;
        "repository_url")
            echo "https://github.com/NextNodeSolutions/test-${template_name}-${timestamp}"
            ;;
        "website_url")
            echo "https://test-${template_name}-${timestamp}.fly.dev"
            ;;
        "dev_domain")
            echo "test-${template_name}-${timestamp}-dev.fly.dev"
            ;;
        "project_keywords"|"keywords")
            # Retourner une structure spéciale pour les arrays
            echo "ARRAY|${template_name},nextnode,test"
            ;;
        *)
            # Valeur générique pour les champs inconnus
            echo "${field}_value"
            ;;
    esac
}

# Fonction pour lister tous les templates disponibles
list_available_templates() {
    log_info "Templates disponibles:"
    echo ""

    find "$TEMPLATES_DIR" -name "template_config.json" -type f | \
    while read -r config_file; do
        local template_path=$(dirname "$config_file")
        local relative_path=${template_path#$TEMPLATES_DIR/}
        local category=$(dirname "$relative_path")
        local name=$(basename "$relative_path")

        echo "  $relative_path"
        echo "    Catégorie: $category"
        echo "    Nom: $name"
        echo "    Config: $config_file"
        echo ""
    done
}

# Fonction pour générer le fichier YAML complet
generate_dynamic_yaml() {
    local template_path="$1"
    local output_file="$2"
    local timestamp=$(date +"%Y%m%d_%H%M%S")

    # Vérifier l'existence du template
    local template_config_file="$template_path/template_config.json"
    if [[ ! -f "$template_config_file" ]]; then
        log_error "Template non trouvé: $template_path"
        log_error "Fichier manquant: $template_config_file"
        exit 1
    fi

    log_info "Génération de la configuration pour: $template_path"

    # Extraire les champs système depuis FileConfig
    local system_fields=$(extract_fileconfig_fields)

    # Extraire les variables du template
    local template_vars=$(extract_template_variables "$template_config_file")

    # Début du fichier YAML avec métadonnées
    cat > "$output_file" << EOF
# ========================================
# CONFIGURATION GÉNÉRÉE AUTOMATIQUEMENT
# ========================================
# Date de génération: $(date)
# Template analysé: $template_path
# Sources:
#   - Champs système: $FILE_CONFIG_PATH
#   - Variables template: $template_config_file
#
# Usage:
#   Mode local:  cargo run -- --config $output_file
#   Mode remote: cargo run -- --remote --config $output_file

# ========================================
# CHAMPS SYSTÈME (FileConfig)
# ========================================
# Ces champs sont définis dans la structure Rust FileConfig
# et sont traités spécialement par le générateur

EOF

    # Ajouter les champs système avec métadonnées
    while read -r field; do
        if [[ -n "$field" ]]; then
            local metadata=$(extract_field_metadata "$field")
            local is_optional=$(echo "$metadata" | cut -d'|' -f1)
            local field_type=$(echo "$metadata" | cut -d'|' -f2)
            local valid_values=$(echo "$metadata" | cut -d'|' -f3)

            local value=$(generate_default_value "$field" "$template_path" "$timestamp")

            if [[ "$is_optional" == "true" ]]; then
                # Commenter les champs optionnels par défaut
                echo "# $field: \"$value\"  # [OPTIONNEL] $field_type" >> "$output_file"

                # Ajouter les valeurs valides si disponibles
                if [[ -n "$valid_values" ]]; then
                    echo "#   Valeurs autorisées: $valid_values" >> "$output_file"
                fi
            else
                echo "$field: \"$value\"  # [REQUIS] $field_type" >> "$output_file"
            fi
        fi
    done <<< "$system_fields"

    # Section variables du template
    cat >> "$output_file" << EOF

# ========================================
# VARIABLES DU TEMPLATE
# ========================================
# Ces variables sont définies dans template_config.json
# et seront placées dans additional_vars lors de l'exécution

EOF

    # Ajouter les variables du template
    while read -r var; do
        if [[ -n "$var" ]]; then
            # Éviter les doublons avec les champs système
            if ! echo "$system_fields" | grep -q "^$var$"; then
                local value=$(generate_default_value "$var" "$template_path" "$timestamp")

                # Gérer les arrays spécialement
                if [[ "$value" == "ARRAY|"* ]]; then
                    local array_values=${value#ARRAY|}
                    echo "$var:" >> "$output_file"
                    IFS=',' read -ra values <<< "$array_values"
                    for val in "${values[@]}"; do
                        echo "  - \"$val\"" >> "$output_file"
                    done
                else
                    echo "$var: \"$value\"" >> "$output_file"
                fi
            fi
        fi
    done <<< "$template_vars"

    # Section finale avec notes
    cat >> "$output_file" << EOF

# ========================================
# NOTES D'UTILISATION
# ========================================
#
# Structure de la configuration:
# - Les champs commentés (#) sont optionnels
# - Les champs REQUIS doivent être fournis
# - Les champs système vont directement dans FileConfig
# - Les autres variables vont dans additional_vars via #[serde(flatten)]
#
# Validation automatique:
# - github_tag: doit être "apps", "packages" ou "utils"
# - template_branch: "main" par défaut
# - create_develop_branch: false par défaut
#
# Modes d'exécution:
# - Local: génère le projet localement
# - Remote: crée un repo GitHub avec le code généré

EOF

    log_success "Configuration générée: $output_file"
}

# ========================================
# FONCTION D'AIDE ET INTERFACE UTILISATEUR
# ========================================

show_help() {
    echo "Script de génération de configuration dynamique pour project-generator"
    echo ""
    echo "Usage: $0 [options] [template_path] [output_file]"
    echo ""
    echo "Options:"
    echo "  --list, -l         Liste tous les templates disponibles"
    echo "  --help, -h         Affiche cette aide"
    echo ""
    echo "Arguments:"
    echo "  template_path      Chemin du template (ex: apps/astro, packages/library)"
    echo "  output_file        Fichier de sortie (défaut: config-[template]-[date].yaml)"
    echo ""
    echo "Exemples:"
    echo "  $0                          # Mode interactif"
    echo "  $0 --list                   # Liste les templates"
    echo "  $0 apps/astro               # Génère config pour Astro"
    echo "  $0 packages/library config.yaml  # Avec fichier spécifique"
    echo ""
    echo "Le script analyse automatiquement:"
    echo "  - FileConfig struct dans le code Rust"
    echo "  - template_config.json du template sélectionné"
    echo "  - Génère une configuration complète avec toutes les clés possibles"
}

# Mode interactif pour sélectionner un template
interactive_template_selection() {
    log_info "Mode interactif - Sélection du template"
    echo ""

    # Créer un tableau des templates disponibles
    local templates=()
    while IFS= read -r line; do
        templates+=("$line")
    done < <(find "$TEMPLATES_DIR" -name "template_config.json" -type f | sed "s|$TEMPLATES_DIR/||" | sed "s|/template_config.json||" | sort)

    if [[ ${#templates[@]} -eq 0 ]]; then
        log_error "Aucun template trouvé dans $TEMPLATES_DIR"
        exit 1
    fi

    echo "Templates disponibles:"
    for i in "${!templates[@]}"; do
        echo "  $((i+1)). ${templates[i]}"
    done
    echo ""

    while true; do
        read -p "Sélectionnez un template (1-${#templates[@]}): " selection

        if [[ "$selection" =~ ^[0-9]+$ ]] && [[ "$selection" -ge 1 ]] && [[ "$selection" -le ${#templates[@]} ]]; then
            echo "${templates[$((selection-1))]}"
            return 0
        else
            echo "Sélection invalide. Veuillez entrer un nombre entre 1 et ${#templates[@]}."
        fi
    done
}

# ========================================
# FONCTION PRINCIPALE
# ========================================

main() {
    # Vérification des dépendances
    if ! command -v jq &> /dev/null; then
        log_error "jq est requis mais non installé. Installez-le avec: brew install jq"
        exit 1
    fi

    # Traitement des arguments
    case "${1:-}" in
        "--list"|"-l")
            list_available_templates
            exit 0
            ;;
        "--help"|"-h"|"")
            if [[ "${1:-}" == "--help" ]] || [[ "${1:-}" == "-h" ]]; then
                show_help
                exit 0
            fi
            # Mode interactif si aucun argument
            local template_path=$(interactive_template_selection)
            ;;
        *)
            local template_path="$1"
            ;;
    esac

    # Valider le chemin du template
    local full_template_path="$TEMPLATES_DIR/$template_path"
    if [[ ! -d "$full_template_path" ]]; then
        log_error "Template non trouvé: $template_path"
        log_info "Utilisez --list pour voir les templates disponibles"
        exit 1
    fi

    # Générer le nom du fichier de sortie
    local template_name=$(basename "$template_path")
    local output_file="${2:-config-${template_name}-$(date +%Y%m%d).yaml}"

    log_info "Template sélectionné: $template_path"
    log_info "Fichier de sortie: $output_file"
    echo ""

    # Générer la configuration
    generate_dynamic_yaml "$full_template_path" "$output_file"

    # Afficher le résultat
    echo ""
    log_info "Contenu généré:"
    echo "----------------------------------------"
    cat "$output_file"
    echo "----------------------------------------"
    echo ""
    log_success "Configuration prête à utiliser!"
    log_info "Commandes suggérées:"
    log_info "  cargo run -- --config $output_file"
    log_info "  cargo run -- --remote --config $output_file"
}

# Exécution du script
main "$@"