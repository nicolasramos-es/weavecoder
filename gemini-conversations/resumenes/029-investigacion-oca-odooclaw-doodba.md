## Tema
Investigación de buenas prácticas OCA para integrar OdooClaw en doodba-copier-template.

## Decisión clave
Se determinó que la integración requiere modificar docker-compose, configurar variables en .docker/odoo.env y utilizar MCP tools, siguiendo estándares OCA y arquitectura Doodba.

## Datos relevantes
- **Estándares:** PEP8, estructura modular, convenciones de naming.
- **Arquitectura Doodba:** entornos devel/test/prod, aislamiento de red, whitelist gateway.
- **OdooClaw:** motor Go <10MB RAM, módulo mail_bot_odooclaw, webhooks asíncronos, herencia de permisos.
- **Herramientas:** MQT (Maintainer Quality Tools) para CI/CD, RLM Acceleration (Map-Reduce).
- **Seguridad:** whitelist de red, human-in-the-loop.

## Categoría
Desarrollo
