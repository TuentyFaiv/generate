# Generate CLI
With this CLI you can generate base code to start to program fast. Also you can personalize the base code to generate.

## Options
```zsh
A command line interface for Frontend development

Usage: tfverse [OPTIONS] [NAME]

Arguments:
  [NAME]  Project name

Options:
  -t, --tool <TOOL>          Template tool to choose
  -a, --arch <ARCH>          Frontend piece or project template
  -l, --language <LANGUAGE>  TypeScript or JavaScript
  -s, --styles <STYLES>      Styles type
  -p, --path <PATH>          Template path to generate
  -y, --sure                 Accept all questions
  -g, --global               Set configuration
  -c, --config <FILE>        Config file to customize this CLI
  -h, --help                 Print help
  -V, --version              Print version
```

## Config file
To personalize the cli you must create a `.json` file. And use the following [schema](https://gvtmpjzawvdhojsxlece.supabase.co/storage/v1/object/public/cli/schema/config.json).

The commented paths are the default values.
### Exmaple `tfverse_config.json`
```json
{
  "$schema": "https://gvtmpjzawvdhojsxlece.supabase.co/storage/v1/object/public/cli/schema/config.json",
  "repository": "git@github.com:YOUR_ORG_OR_YOUR_USER",
  "root": "", // The project root, default is "./"
  "paths": { // The modules paths
    "globals": {
      "contexts": null, // "./src/logic/contexts"
      "schemas": null, // "./src/logic/schemas"
      "services": null, // "./src/logic/services"
      "types": null, // "./src/logic/typing"
      "ui": null // "./src/ui"
    },
    "react": {
      "hocs": null, // "./src/logic/hocs"
      "hooks": {
        "global": null, // "./src/logic/hooks"
        "internal": null // "hooks"
      },
      "locales": null, // "./public/locales"
      "routes": null // "./src/logic/routes"
    },
    "svelte": {
      "actions": null, // "./src/logic/actions"
      "layouts": null, // "./src/routes"
      "locales": null, // "./static/locales"
      "pages": null, // "./src/routes"
      "stores": null // "./src/logic/stores"
    },
    "vanilla": {
      "classes": null, // "./src/logic/classes"
      "functions": null // "./src/logic/functions"
    }
  },
  "tools_type": { // The list to generate libraries and projects boilerplates
    "vanilla": [
      [
        "base", // Name to show in the list
        {
          "name": "REPOSITORY_NAME",
          "project": "main",
          "library": null
        }
      ]
    ],
    "react": [
      [
        "base", // Name to show in the list
        {
          "name": "REPOSITORY_NAME",
          "project": "main",
          "library": null
        }
      ]
    ],
    "svelte": [
      [
        "base", // Name to show in the list
        {
          "name": "REPOSITORY_NAME",
          "project": "main",
          "library": null
        }
      ]
    ]
  },
  "templates": { // The paths to your custom templates, all paths are concat with "$HOME"
    "react": {
      "component": null, // Example: "path/to/your/folder/component"
      "context": null,
      "hoc": null,
      "hook": null,
      "layout": null,
      "page": null,
      "schema": null,
      "service": null
    },
    "svelte": {
      "action": null,
      "context": null,
      "layout": null,
      "page": null,
      "schema": null,
      "service": null,
      "store": null,
      "component": null
    },
    "vanilla": {
      "class": null,
      "component": null,
      "function": null,
      "layout": null,
      "page": null,
      "schema": null,
      "service": null
    }
  }
}
```

## Templates paths
If you want to personalize this CLI you must create the following files inside the folders that you set in the `templates` option.
### React Component
| Path | To |
| --- | --- |
| `/component.(ts\|js)x` | Component |
| `/import.(ts\|js)` | Component imports |
| `/proptypes.ts` | Component props |
| `/styles.(ts\|js)` | Component styles |
| `/styles.responsive.(ts\|js)` | Component responsive |
### Svelte Component
| Path | To |
| --- | --- |
| `/component.svelte` | Component |
| `/script.(ts\|js).svelte` | Component script |
| `/import.(ts\|js)` | Component imports |
| `/proptypes.ts` | Component props |
| `/styles.(ts\|js)` | Component styles |
| `/styles.responsive.(ts\|js)` | Component responsive |
### Vanilla Component
> Not ready yet
### React Layout
| Path | To |
| --- | --- |
| `/layout.(ts\|js)x` | Layout |
| `/layout.styles.(ts\|js)x` | Layout barrel styles |
| `/proptypes.ts` | Layout props |
| `/styles.(ts\|js)` | Layout styles |
| `/styles.responsive.(ts\|js)` | Layout responsive |
### Svelte Layout
| Path | To |
| --- | --- |
| `/layout.svelte` | Layout |
| `/layout.styles.(ts\|js)x` | Layout barrel styles |
| `/script.(ts\|js).svelte` | Layout script |
| `/proptypes.ts` | Layout props |
| `/styles.(ts\|js)` | Layout styles |
| `/styles.responsive.(ts\|js)` | Layout responsive |
### Vanilla Layout
> Not ready yet
### React Page
| Path | To |
| --- | --- |
| `/page.(ts\|js)x` | Page |
| `/proptypes.ts` | Page props |
| `/locale.json` | Locale for i18n |
| `/i18n.(ts\|js)` | i18n context |
| `/styles.(ts\|js)` | Page styles |
| `/styles.responsive.(ts\|js)` | Page responsive |
| `/router.(ts\|js)` | Router |
| `/route.(ts\|js)` | Route |
| `/config.(ts\|js)` | Config file project |
| `/config.aliases.(ts\|js)` | Project aliases |
| `/tsconfig.json` | TSConfig file project |
| `/tsconfig.aliases.json` | TSConfig aliases |
### Svelte Page
| Path | To |
| --- | --- |
| `/page.svelte` | Page |
| `/script.(ts\|js).svelte` | Page script |
| `/locale.json` | Locale for i18n |
| `/i18n.(ts\|js)` | i18n store |
| `/proptypes.ts` | Page props |
| `/styles.(ts\|js)` | Page styles |
| `/styles.responsive.(ts\|js)` | Page responsive |
| `/config.(ts\|js)` | Svelte config |
| `/config.aliases.(ts\|js)` | Svelte config aliases |
### Vanilla Page
> Not ready yet
### React Schema
| Path | To |
| --- | --- |
| `/schema.(ts\|js)x` | Schema |
| `/proptypes.ts` | Schema props |
### Svelte Schema
| Path | To |
| --- | --- |
| `/schema.(ts\|js)x` | Schema |
| `/proptypes.ts` | Schema props |
| `/proptypes.imports.ts` | Schema props imports |
### Vanilla Schema
> Not ready yet

## Templates keywords

| Keyword | Description |
| --- | --- |
| `NAME_CAMEL` | Camel case name |
| `NAME_PASCAL` | Pascal case name |
| `NAME_DASH` | Dash case name |
| `NAME_CONSTANT` | Constant case name |
| `NAME_SNAKE` | Snake case name |
| `NAME_LOWER` | Lowercase name |
| `NAME` | User input name |
| `NAMESPACE` | User input namespace |
| `EXT_STYLES` | Styles extension |

## Templates keywords for svelte

| Keyword | Description |
| --- | --- |
| `SCRIPT` | Script tag |

## Templates keywords for schemas

| Keyword | Description |
| --- | --- |
| `/* NEXT_TYPE */` | Next type |
| `/* PROPTYPES */` | PropTypes |

## Tempaltes keywords for pages

| Keyword | Description |
| --- | --- |
| `/* NEXT_ALIAS */` | Alias for next page |
| `/* NEXT_LOCALE */` | Locale for next page |
| `/* NEXT_ROUTE */` | Route for next page (Only ReactJS) |

## Templates keywords for pages and schemas

| Keyword | Description |
| --- | --- |
| `/* NEXT_IMPORT */` | Next import |