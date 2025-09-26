# Project Template Generator

This CLI tool allows you to generate new projects from customizable templates. It provides an interactive interface for selecting templates and inputting project-specific variables, with support for both local and remote generation via GitHub.

## Features

- **Local Mode**: Interactive project generation with template selection
- **Remote Mode**: Automatic generation + GitHub repository creation
- **File Configuration**: YAML and JSON support for automation
- **Smart Validation**: Project and package names with appropriate validation
- **Automatic Installation**: Dependency installation after generation
- **Dynamic Replacement**: Customizable variables in templates
- **GitHub Integration**: Automatic repository creation with initial push

## Installation

1. Ensure you have Rust and Cargo installed on your system.
2. Clone this repository:
   ```bash
   git clone [repository_url]
   cd [repository_name]
   ```
3. Build the project:
   ```bash
   cargo build --release
   ```

## Available CLI Arguments

```
Options:
  -d, --debug                    Enable debug output
  -f, --config <CONFIG>          Path to configuration file (YAML or JSON)
  -c, --category <CATEGORY>      Template category
  -n, --template <TEMPLATE>      Template name
      --remote                   GitHub mode (generation + repository creation)
      --token <TOKEN>           GitHub token for remote mode
  -h, --help                    Show help
  -V, --version                 Show version
```

## Quick Start - Template Testing

Pour tester rapidement vos templates, utilisez les commandes Make automatisées :

### Commandes disponibles

```bash
# Afficher les templates disponibles et les commandes
make show-templates

# Générer une configuration pour le template Library
make config-library

# Générer une configuration pour le template Astro
make config-astro

# Tester le template Library (génère config + lance le test)
make test-library

# Tester le template Astro (génère config + lance le test)
make test-astro

# Nettoyer les fichiers de configuration de test
make clean-configs
```

### Script automatique de configuration

Le script `scripts/generate-config.sh` analyse automatiquement les `template_config.json` et génère des fichiers de configuration prêts à l'emploi :

```bash
# Générer une configuration pour un template spécifique
./scripts/generate-config.sh library
./scripts/generate-config.sh astro

# Avec un nom de fichier personnalisé
./scripts/generate-config.sh library ma-config-custom.yaml
```

**Avantages :**
- ✅ **Extraction automatique** des placeholders depuis `template_config.json`
- ✅ **Valeurs par défaut intelligentes** selon le type de template
- ✅ **Noms uniques** avec timestamp pour éviter les conflits
- ✅ **Configuration complète** prête à l'utilisation


## Tutorial 1: Remote Mode (GitHub)

Remote mode generates a project locally then automatically creates a GitHub repository with the code.

### Prerequisites

1. **GitHub Token**: You must have a GitHub token with appropriate permissions
2. **Configuration File**: A YAML or JSON file is required
3. **Environment Variables**: Optional, can be used instead of the `--token` flag

### File Configuration

Create a configuration file (e.g., `config.yaml`):

```yaml
# Required configuration
project_name: "my-awesome-project"         # Project name (used for folder)
name: "@myorg/my-awesome-project"         # Package name (for package.json, etc.)
description: "Description of my project"  # GitHub repository description

# Template configuration
template_category: "apps"                  # Category: "apps", "packages", or "utils"
template_name: "nextjs-app"               # Specific template name

# Additional variables (optional, depends on template)
author: "My Name"
license: "MIT"
version: "1.0.0"
keywords: "nextjs,react,webapp"
```

### Environment Variables

You can set your GitHub token in the environment:

```bash
export GITHUB_TOKEN="ghp_your_token_here"
```

### Commands

#### Option 1: With token as argument
```bash
cargo run -- --remote --config config.yaml --token ghp_your_token_here
```

#### Option 2: With environment variable
```bash
export GITHUB_TOKEN="ghp_your_token_here"
cargo run -- --remote --config config.yaml
```

### Automatic Process

When you execute the command, the tool:

1. ✅ Reads your configuration file
2. ✅ Validates required data
3. ✅ Generates the project in a temporary folder
4. ✅ Replaces all variables in template files
5. ✅ Installs dependencies (npm, pnpm, etc.)
6. ✅ Asks for the GitHub repository name (default: project name)
7. ✅ Creates the GitHub repository in the organization
8. ✅ Initializes Git, makes initial commit and pushes code
9. ✅ Cleans up temporary folder
10. ✅ Displays the new repository URL

### Complete Example

```bash
# 1. Create the config file
cat > my-project.yaml << EOF
project_name: "portfolio-client"
name: "@nextnode/portfolio-client"  
description: "Modern portfolio site with Next.js"
template_category: "apps"
template_name: "nextjs-app"
author: "NextNode Team"
license: "MIT"
version: "0.1.0"
EOF

# 2. Execute remote generation
export GITHUB_TOKEN="ghp_your_personal_token"
cargo run -- --remote --config my-project.yaml

# 3. The tool will ask:
# > Project name: portfolio-client
# > Do you want to use the project name as the repository name? (Y/n)
```

## Tutorial 2: Local Mode (All Possibilities)

Local mode offers several generation options based on your needs.

### Option 1: Complete Interactive Mode

The simplest method to get started:

```bash
cargo run
```

**Interactive process:**
1. Template selection from a list
2. Project name input (automatic validation)
3. Package name input (automatic validation)  
4. Custom variables input according to template
5. Generation in parent folder (`../`)
6. Automatic dependency installation

**Example session:**
```
? Select a template: › nextjs-app (apps)
? Enter value for project_name: › my-new-site
? Enter value for name: › @myorg/my-new-site
? Enter value for description: › My modern website
? Enter value for author: › My Name
✓ Project generated successfully!
✓ Dependencies installed successfully!
```

### Option 2: Semi-Automatic Mode with Configuration

Use a config file but stay local:

```bash
cargo run -- --config config.yaml
```

**Advantages:**
- No manual interaction
- Reproducible and scriptable
- Ideal for automation

**Configuration file example:**
```yaml
project_name: "my-local-project"
name: "@myorg/my-local-project"
description: "Automatically generated project"
template_category: "packages"
template_name: "library"
author: "Developer"
license: "Apache-2.0"
version: "0.1.0"
# Custom variables according to template
main_export: "index.ts"
typescript: "true"
```

### Option 3: Targeted Mode with Arguments

Directly specify the desired template:

```bash
cargo run -- --category apps --template nextjs-app
```

**Process:**
1. Pre-selected template
2. Interactive input only for variables
3. Immediate generation

### Option 4: Debug Mode

To diagnose or understand the process:

```bash
cargo run -- --debug --config config.yaml
```

**Information displayed:**
- Configuration file parsing
- Extracted variables and their values
- Processed files and replacements made
- Dependency installation steps
- Details of each operation

### Option 5: Advanced Combinations

#### Partial configuration + interaction
```yaml
# config-partial.yaml
project_name: "base-project"
template_category: "apps"
template_name: "react-app"
# Other variables will be asked interactively
```

```bash
cargo run -- --config config-partial.yaml
```

#### Debug with specific template
```bash
cargo run -- --debug --category packages --template library
```

### Available Templates and Categories

**Supported categories:**
- `apps`: Complete applications (Next.js, React, etc.)
- `packages`: NPM libraries and packages  
- `utils`: Development utilities and tools

**Templates Repository:**
You can find all available templates at: https://github.com/NextNodeSolutions/utils_project-templates

**Template structure:**
```
templates/
├── apps/
│   ├── nextjs-app/
│   │   ├── template_config.json
│   │   ├── package.json
│   │   ├── src/
│   │   └── ...
│   └── react-app/
├── packages/
│   └── library/
└── utils/
    └── cli-tool/
```

### Available System Variables

All these variables can be used in your templates:

- `{{project_name}}`: Project name (validated: lowercase, hyphens, underscores)
- `{{name}}`: Package name (validated: npm standard format)
- `{{description}}`: Project description
- `{{author}}`: Project author
- `{{license}}`: License type
- `{{version}}`: Initial version
- `{{keywords}}`: Keywords (transformed to array for JSON)

### Template Configuration

Each template contains a `template_config.json`:

```json
{
  "files_to_replace": [
    "package.json",
    "README.md", 
    "src/config.ts"
  ],
  "replacements": [
    {
      "name": "project_name",
      "key": "name", 
      "value": "{{project_name}}",
      "type": "string"
    },
    {
      "name": "keywords",
      "key": "keywords",
      "value": "{{keywords}}",
      "type": "array"
    }
  ]
}
```

### Output Directory

**Local Mode:** Projects are generated in `../project-name/`

**Structure after generation:**
```
../
├── my-new-project/
│   ├── package.json        # Variables replaced
│   ├── README.md          # Variables replaced
│   ├── src/
│   ├── node_modules/      # Dependencies installed
│   └── ...
```

## Environment Variables Configuration

### GitHub Token

For remote mode, you can use:

```bash
# Option 1: Environment variable
export GITHUB_TOKEN="ghp_your_token"

# Option 2: .env file (if supported)
echo "GITHUB_TOKEN=ghp_your_token" >> .env

# Option 3: CLI argument
cargo run -- --remote --token ghp_your_token --config config.yaml
```

### Global Debug

```bash
# Enable debug for all operations
export DEBUG=1
cargo run
```

## Use Case Examples

### 1. Developing a new webapp
```bash
# Complete interactive mode
cargo run
# Select: nextjs-app (apps)
# Fill in the requested variables
```

### 2. Creating an NPM library
```bash
# With predefined configuration
cat > lib-config.yaml << EOF
project_name: "utils-math"
name: "@myorg/utils-math"
description: "Reusable mathematical utilities"
template_category: "packages"
template_name: "library"
author: "My Team"
license: "MIT"
EOF

cargo run -- --config lib-config.yaml
```

### 3. Generation + GitHub Deployment
```bash
# Remote mode with repository creation
export GITHUB_TOKEN="ghp_your_token"
cargo run -- --remote --config production-config.yaml
```

### 4. Development with debugging
```bash
# See all process details
cargo run -- --debug --category apps --template nextjs-app
```

## Validation and Security

- **Project names**: Strict validation (lowercase, letters, numbers, hyphens, underscores)
- **Package names**: NPM standard format validation
- **GitHub tokens**: Never displayed in plain text, secure handling
- **Temporary files**: Automatic cleanup after remote generation
- **Permissions**: Rights verification before repository creation

## Troubleshooting

### Common Errors

**"GitHub token is required"**
```bash
export GITHUB_TOKEN="your_token"
# or
cargo run -- --token your_token --remote --config config.yaml
```

**"Config file is required for remote mode"**
```bash
# Remote mode always requires a config file
cargo run -- --remote --config your-config.yaml
```

**"Failed to parse config file"**
```bash
# Check YAML/JSON syntax
cargo run -- --debug --config your-config.yaml
```

**"Template not found"**
```bash
# Check available categories and templates
cargo run -- --debug
```

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## License

This project is licensed under the GPL-3.0-only License.