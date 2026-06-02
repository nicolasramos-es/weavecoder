## Tema
Error en la etiqueta de caché Docker causado por el uso de barras diagonales (/) en el nombre de la rama Git (ej. fix/branch).

## Decisión clave
Implementar el uso de la variable `BRANCH_SLUG` para sanitizar los nombres de las ramas y generar etiquetas de caché Docker válidas.

## Datos relevantes
- Problema: Las etiquetas de caché Docker no permiten el carácter `/`.
- Causa: Nombres de rama con estructura tipo `fix/branch`.
- Solución técnica: Utilizar `BRANCH_SLUG` para transformar el nombre de la rama antes de generar la etiqueta.

## Categoría
Desarrollo
