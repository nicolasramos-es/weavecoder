# n8n Skills — Skills de workflows n8n para Hermes Agent

7 skills instalados desde el repositorio [czlonkowski/n8n-skills](https://github.com/czlonkowski/n8n-skills) (v1.9.0, 5.1k stars, MIT).

Ubicación: `~/.hermes/skills/n8n/`

## Skills instalados

| Skill | Descripción | Activación |
|-------|-------------|------------|
| **n8n-expression-syntax** | Sintaxis correcta de expresiones n8n ({{}}), variables $json, $node, errores comunes | Cuando se escriben expresiones o se mapean datos entre nodos |
| **n8n-mcp-tools-expert** | Guía experta de herramientas MCP de n8n: search_nodes, get_node, validate_node, n8n_update_partial_workflow | PRIORIDAD MÁXIMA. Consultar siempre antes de llamar a cualquier tool MCP de n8n |
| **n8n-workflow-patterns** | 5 patrones arquitectónicos probados: webhook, API HTTP, base de datos, IA, programado | Al construir workflows, diseñar estructura o elegir patrón |
| **n8n-validation-expert** | Interpretar errores de validación, falsos positivos, auto-fix | Cuando validate_node o validate_workflow devuelven errores |
| **n8n-node-configuration** | Configuración de nodos según operación, dependencias de propiedades | Al configurar parámetros de nodos, saber qué campos son requeridos |
| **n8n-code-javascript** | Código JavaScript en Code nodes: $input, $helpers, DateTime, patrones de producción | Cuando un workflow necesita un Code node |
| **n8n-code-python** | Código Python en Code nodes: _input, _json, limitaciones (sin librerías externas) | Solo cuando el usuario pide Python explícitamente (95% de casos usar JS) |

## Requisitos

Para que funcionen al 100% se necesita:

1. **Servidor MCP n8n** (`n8n-mcp`) — [instalación](https://github.com/czlonkowski/n8n-mcp)
2. Configurar `mcp_servers` en `~/.hermes/config.yaml` apuntando al servidor MCP
3. Tener n8n corriendo (local o remoto)

Sin el servidor MCP, los skills son solo documentación — el agente sabe cómo usarlos pero no tiene las tools MCP disponibles.

## Notas

- Estos skills son para *Claude Code* originalmente, convertidos a formato Hermes
- Están diseñados para trabajar juntos: patrones → tools → configuración → expresiones → código → validación
- El más importante es `n8n-mcp-tools-expert` — consultar SIEMPRE antes de llamar tools MCP
