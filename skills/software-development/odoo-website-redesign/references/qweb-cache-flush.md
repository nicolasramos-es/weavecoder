# Odoo QWeb Cache Flush — Técnicas y gotchas

## Problema
Cuando haces `write` en `ir.ui.view` (o `website.page`), Odoo marca la vista como dirty pero los workers sirven la plantilla cacheada durante ~5 min.

## Solución 1: URL con `?debug=assets` ✅ (la buena)
En el navegador, recarga la página con `?debug=assets` appended:
```
https://globalo.es/?debug=assets
https://globalo.es/page/url?debug=assets
```

Esto fuerza la recompilación QWeb en el servidor. Funciona en Odoo 14-18.

## Solución 2: Invalidar desde backend Odoo
Ajustes → Técnico → Interfaz → Vistas → Buscar el template → Botón "Invalidar"

Esto limpia el caché QWeb globalmente.

## Lo que NO funciona

### `clear_caches()` vía XML-RPC
```python
models.execute_kw(db, uid, password, 'ir.ui.view', 'clear_caches', [])
# → IndexError: tuple index out of range
```
El método `clear_caches()` de Odoo espera positional args, pero XML-RPC solo pasa args/kwargs separadamente. No usar.

### `debug=1` solo
Recarga el debugger pero NO fuerza recompilación QWeb. Algunos cambios no se ven.

### Reiniciar workers Odoo
Funciona pero es overkill y corta tráfico.

## Secuencia correcta tras actualizar un template QWeb

1. `write` en `ir.ui.view` con el nuevo `arch_db`
2. Recargar en navegador con `?debug=assets`
3. Verificar con Computer Use (cua-driver screenshot)

## Caso típico: Headline template (footer)
```
ID: 8309
key: website.template_footer_headline
inherit: website.layout
xpath: //div[@id='footer']
```

Para actualizar el footer de todas las páginas:
```python
models.execute_kw(db, uid, password, 'ir.ui.view', 'write', [[8309], {
    'arch_db': '<data inherit_id="website.layout" name="Headline" active="True">...',
    'active': True
}])
# Luego: ?debug=assets en el navegador
```