---
name: odoo-website-redesign
description: "Redisenar sitios web Odoo con enfoque hibrido de Computer Use + codigo. Flujo: entender negocio, boceto, aprobacion, Computer Use (explorar/verificar) y codigo (cambios)."
---

# Odoo Website Redesign

## Cuando usar este skill

- El usuario pide redisenar una web hecha en Odoo (v14-18)
- Cambiar diseno visual, estructura de navegacion o anadir secciones
- El usuario menciona Computer Use o codigo para el trabajo
- Cliente con tienda Odoo que quiere renovar imagen

## Flujo de trabajo (6 fases)

### FLUJO 1: Plan → Aprobación → Ejecución (para nuevo desarrollo)

** El usuario EXIGE ver el plan/boceto detallado PRIMERO y dar su visto bueno antes de tocar código/CSS.
- No ejecutar nada sin mostrarle el plan primero.
- Incluir en cada fase: tareas concretas, orden propuesto, esfuerzo estimado.
- Tras cada fase: verificar y pedir approval antes de continuar.

### FLUJO 2: Usuario crea bloques → yo lleno de contenido (contenido puro)

El usuario crea y placementa los bloques estructurales en el editor visual de Odoo (`?enable_editor=1`). Tú solo le das:
- Textos para cada bloque (SEO-optimized)
- URLs de imágenes (ir.attachment IDs ya subidos)
- Enlaces y URLs targets

El contenido se pasa como texto plano — el usuario copia y pega en los campos del editor visual. Sin XML-RPC, sin código.

**Secuencia:**
1. Usuario abre `https://globalo.es/?enable_editor=1` y crea/arrastra los bloques que quiere
2. Usuario te dice qué blocos tiene y en qué orden
3. Tú le devuelves una tabla:

| Bloque | Campo | Contenido |
|--------|-------|-----------|
| Hero | heading | "Uniformes profesionales..." |
| Hero | subheading | "Soluciones de vestuario..." |
| Hero | CTA | "Solicitar presupuesto" → `/contactus` |
| ... | ... | ... |

4. Usuario llena los campos del editor visual con esos datos.


### FASE 0: Recopilar y entender (CRITICO)

Antes de tocar nada:

1. Pide TODO el material del cliente: URL actual, PDFs de enfoque/diseno, catalogos, logos, webs de referencia
2. Clarifica la estructura de negocio (PITFALL COMUN):
   - Es la web una tienda principal con marcas/subcatalogos dentro?
   - O cada PDF/marca es un sitio independiente?
   - Pregunta explicitamente -- no asumas que el primer PDF es la marca principal
3. Identifica menus existentes a mantener:
   - Que elementos del menu actual se MANTIENEN?
   - Que es NUEVO?
   - Que se MUEVE o ELIMINA?
4. Prepara plan/boceto DETALLADO antes de ejecutar y pide aprobacion

### FASE 1: Exploracion (Computer Use)
- Acceder backend Odoo con credenciales
- Explorar configuracion: Temas, Colores, Tipografias
- Revisar categorias y estructura de menus actual
- Capturar screenshots del estado actual

### FASE 2: Tema y configuracion (Codigo + Computer Use)
- Elegir/instalar tema base
- Personalizar colores via CSS/SCSS
- Configurar tipografias
- Subir logo y assets
- Computer Use: verificar visualmente

### FASE 3: Home Page (Codigo + Computer Use)
- Disenar Header con menus acordados
- Crear Hero/Banner principal
- Anadir secciones (grid servicios, catalogos)
- Disenar Footer
- Computer Use: ajustar bloques en builder
- Codigo: CSS/SCSS para efectos y responsive

### FASE 4: Paginas interiores (Codigo)
- Pagina de Catalogos: grid con tarjetas por catalogo/marca
- Cada tarjeta: imagen + nombre + descripcion + enlace
- Enlaces: PDF descargable, URL externa o pagina interna
- Otras paginas: Personalizacion, DTF/DTI/DTV, Contacto

### FASE 5: Productos (Codigo)
- Adaptar vista de productos al nuevo diseno
- Anadir metadata tecnica (tablas tallas, iconos certificacion)
- Configurar variantes (colores, tallas)

### FASE 6: Ajustes finales (Computer Use)
- Revisar responsive
- Verificar navegacion completa y carrito
- Capturar screenshots finales para aprobacion

## Computer Use — PREREQUISITO: Safari JavaScript Apple Events

Para controlar Safari con Computer Use (cua-driver), el usuario debe habilitar manualmente:
**Safari → Preferencias → Avanzado → "Allow JavaScript from Apple Events"** ✅

Sin esto, el árbol AX de Safari solo devuelve menús del sistema (474+ elementos de MenuBar重复) y no se puede acceder al contenido web.

Si cua-driver se desconecta con error "MCP server unreachable", es多半 porque Safari necesita este ajuste.

## Computer Use vs Codigo

| Tarea | Herramienta | Nota |
|---|---|---|
| Explorar backend, ver config | Computer Use | Requiere JS Apple Events en Safari |
| Capturar estado actual | Computer Use | Requiere JS Apple Events en Safari |
| **Drag drop Website Builder** | **SOLO Computer Use** | Solo funciona en el navegador real |
| Verificar responsive | Computer Use | Solo con JS Apple Events habilitado |
| **CSS/SCSS personalizado** | **Codigo via XML-RPC** | Snippets nativos son editados via code |
| Templates QWeb | Codigo | |
| Snippets nativos (Feature Grid, Cards, etc.) | Computer Use | Entrar en modo editor: `/web#allow_editor=1` |
| Variantes de producto | Codigo | |
| Configurar colores/tipografias | Ambos | |

## BLOQUES NATIVOS DE ODOO — NO usar HTML custom

**El usuario EXIGE usar bloques/snippets nativos de Odoo website builder (s_feature_grid, s_card, s_picture, s_text_block, etc.) — NO escribir HTML custom para estructuras de contenido.**

Flujo correcto para Computer Use con bloques nativos:
1. Navegar a `https://globalo.es/web#allow_editor=1` (editor visual activo)
2. En Odoo, ir a "Contenido → Página" y editar la página correspondiente
3. En el editor: buscar en el panel de snippets ("Personalizar") los bloques:
   - **s_feature_grid** — rejilla de 2-4 características con iconos
   - **s_card_group** — tarjetas con imagen, título, descripción
   - **s_picture** — imagen sola (para hero)
   - **s_text_block** — texto enriquecido
   - **s_three_columns** — 3 columnas con iconos
   - **s_showcase** — sección con imagen + texto lado a lado
   - **s_cover** — cover/full-width con imagen de fondo
4. Arrastrar los bloques al lugar correcto, editar contenido inline
5. Para cambios massivos o CSS: usar código XML-RPC

**Regla de oro:** Si Odoo tiene un snippet nativo para lo que necesitas, ÚSALO. El HTML custom es el último recurso.

## Paleta de colores — NEGRO Y BLANCO SOLO (sin excepciones)

**El usuario EXIGE diseño monocromo: solo blanco y negro. NO usar dorado, naranja, azul ni otros colores de acento.**

Colores permitidos:
```
Negro:             #000000 (texto, bordes, elementos)
Gris oscuro:       #333333 / #222222 (subtítulos, descripciones)
Gris medio:        #666666 / #888888 (texto secundario, iconos)
Gris claro:        #f5f5f5 / #eeeeee (fondos de sección alternos)
Blanco:            #ffffff (fondo principal, tarjetas)
Gris borde:        #dddddd (bordes de tarjetas, separadores)
```

### NO usar (colores vetados)
- ❌ Dorado / naranja / ámbar (#d4a843, #f59e0b, etc.)
- ❌ Azul marino (#0a1628, #1a3a5c, etc.)
- ❌ Azul claro (#a8c4d4, #6b8aa4)
- ❌ Verde, rojo, o cualquier otro color de acento

### Estructura hero banner (B/N puro)
```html
<section style="background: #000000; padding: 80px 0;">
  <div style="text-align: center;">
    <div style="border-bottom: 2px solid #ffffff; padding-bottom: 8px; margin-bottom: 24px;">
      <span style="color: #ffffff; font-size: 12px; letter-spacing: 4px; text-transform: uppercase;">TAGLINE</span>
    </div>
    <h1 style="color: #ffffff; font-size: 72px; font-weight: 800; letter-spacing: -2px; margin: 0 0 16px;">
      GLÓBALO
    </h1>
    <p style="color: #888888; font-size: 22px; margin: 0 0 40px;">Subtítulo</p>
    <div style="display: flex; gap: 16px; justify-content: center;">
      <a href="/cta" style="background: #ffffff; color: #000000; padding: 14px 36px; display: inline-block; text-decoration: none; font-weight: 600;">CTA Principal</a>
      <a href="/contact" style="border: 2px solid #ffffff; color: #ffffff; padding: 14px 36px; display: inline-block; text-decoration: none;">CTA Secundario</a>
    </div>
  </div>
</section>
```

### Estructura 3-cajas secciones (B/N puro, bordes limpios)
```html
<div style="display: flex; gap: 24px; flex-wrap: wrap; max-width: 1200px; margin: 0 auto; padding: 0 20px;">
  <div style="flex: 1; min-width: 220px; background: #ffffff; border: 1px solid #000000;
               padding: 40px 32px; text-align: center;">
    <div style="font-size: 48px; margin-bottom: 20px; filter: grayscale(100%);">🏫</div>
    <h3 style="margin: 0 0 12px; font-size: 22px;">Colegios</h3>
    <p style="color: #666666; margin: 0 0 20px;">Descripción del servicio</p>
    <a href="/cole-gios" style="color: #000000; text-decoration: none; border-bottom: 1px solid #000; padding-bottom: 2px;">Ver más →</a>
  </div>
</div>
```

### Logos en escala de grises
```html
<img src="/web/image/ir.attachment/123458/datas"
     style="height: 50px; object-fit: contain; margin-bottom: 12px;
            filter: grayscale(100%);" loading="lazy"/>
```

### Footer completo datos empresa (B/N puro)
```html
<section style="background: #000000; padding: 50px 0; text-align: center;">
  <p style="color: #ffffff; margin: 0 0 8px;"><strong>DISEGLOB S.L.U.</strong> · CIF: ESB35909886</p>
  <p style="color: #888888; margin: 0 0 8px;">C/FAJARDO 1 BAJO B · 35500 Arrecife, Lanzarote</p>
  <p style="color: #888888; margin: 0 0 24px;">928 817 302 · info@globalo.es</p>
  <div style="display: flex; gap: 24px; justify-content: center; flex-wrap: wrap;">
    <a href="/page/politica-de-privacidad-y-cookies" style="color: #ffffff; text-decoration: none; border-bottom: 1px solid #ffffff; padding-bottom: 2px;">Política de Privacidad</a>
    <a href="/page/devoluciones" style="color: #ffffff; text-decoration: none; border-bottom: 1px solid #ffffff; padding-bottom: 2px;">Política de Envío</a>
    <a href="/page/cookie-policy" style="color: #ffffff; text-decoration: none; border-bottom: 1px solid #ffffff; padding-bottom: 2px;">Cookies</a>
  </div>
</section>
```

## Obtener datos de empresa desde Odoo (res.company)

```python
company = models.execute_kw(db, uid, password, 'res.company', 'search_read',
    [[]],
    {'fields': ['name', 'phone', 'email', 'website', 'vat', 'street', 'city', 'zip', 'country_id', 'color'],
     'limit': 10}
)
# Retorna: [{'name': 'DISEGLOB S.L.U.', 'phone': '928817302(ARRECIFE)',
#            'email': 'info@globalo.es', 'vat': 'ESB35909886',
#            'street': 'C/FAJARDO 1 BAJO B', 'city': 'Arrecife', 'zip': '35500'}]
```

**Nota:** Campo `favicon` NO existe en `res.company`. Usar solo los campos listados arriba.

## Técnicas de exploración Odoo (XML-RPC)

**PROBLEMA CONOCIDO:** `/jsonrpc` solo funciona para autenticación. Para operaciones de escritura/lectura de modelos en Odoo 17, usa SIEMPRE `/xmlrpc/2/` con `xmlrpc.client`.

### Endpoints correctos (Odoo 17+)
```
/xmlrpc/2/common  → authenticate
/xmlrpc/2/object → execute_kw (lectura/escritura)
/jsonrpc         → solo login (no usar para nada más)
```

### Autenticación con xmlrpc.client (RECOMENDADO)
```python
import xmlrpc.client
URL = "https://example.com/xmlrpc/2/common"
AUTH = ("usuario@email.com", "password")
proxy = xmlrpc.client.ServerProxy(URL)
uid = proxy.authenticate("db_name", AUTH[0], AUTH[1], {})
# => uid = 62
```

### Operaciones de modelo con execute_kw
```python
models = xmlrpc.client.ServerProxy("https://example.com/xmlrpc/2/object")
records = models.execute_kw(
    "db_name", uid, AUTH[1],    # db, uid, password
    "website.menu", "search_read",
    [[("website_id", "=", 1)]],
    {"offset": 0, "limit": 100, "fields": ["id", "name", "url", "parent_id"]}
)
# parent_id retorna como [id_int, "display_name"]
```

### Lo que NO funciona en Odoo 17
- `urllib` + `/jsonrpc` para modelos con campos many2one → `psycopg2.ProgrammingError: can't adapt type 'dict'`
- `curl` + `json` pipe → misma razón, serialización de campos complejos
- Campos many2one en search_read con jsonrpc → el parser no puede con las tuplas de relación
- Campo `datas_fname` en `ir.attachment` create → no es válido en Odoo 17

### Lo que SÍ funciona
- `xmlrpc.client.ServerProxy` vs `/xmlrpc/2/object`
- `search_read` especificando explicitly los fields deseados
- `create`, `write`, `unlink` via `execute_kw`
- `base64.b64encode()` para convertir archivos binarios a string para el campo `datas`

## Mount SMB (NAS) en macOS

### Preparación: formato de credenciales
- El usuario proporciona usuario/contraseña para el NAS
- **URL-encode** caracteres especiales en la contraseña: `@` → `%40`
  - Ejemplo: contraseña `@Multilock00@` → `Multilock00%40`
- Formato share: `//usuario:password_encoded@NAS_HOST/Share/Path`

### Montaje SMB en macOS
```bash
# Opción 1: mount_smbfs (requiere root)
sudo mount_smbfs //usuario:password_encoded@HOST/SHARE /mount/point

# Opción 2: Connect to Server en Finder (sin root)
open "smb://usuario:password_encoded@HOST/SHARE"
# Esto abre Finder y monta el volumen en /Volumes/

# Opción 3: Crear punto de montaje manual
mkdir -p ~/mnt/nueva_web
mount_smbfs //usuario:password_encoded@HOST/SHARE ~/mnt/nueva_web
```

### Errores comunes
- `mount_smbfp`: **NO existe** — comando incorrecto, usar `mount_smbfs`
- `File exists`: ya montado, verificar con `mount | grep smb`
- `Permission denied`: probablemente credenciales mal codificadas (el `@` en password)
- Shares con muchos archivos pueden dar timeout en listing (ej. raíz de `Compartido`)

### Desmontar
```bash
umount /mount/point
# o
diskutil unmount force /mount/point
```

## Subir archivos a Odoo (ir.attachment)

### Logos e imágenes
```python
import xmlrpc.client
import base64, os

models = xmlrpc.client.ServerProxy("https://globalo.es/xmlrpc/2/object")

with open("/ruta/logo.jpg", "rb") as f:
    data = base64.b64encode(f.read()).decode("utf-8")

attachment_id = models.execute_kw(
    db, uid, password,
    "ir.attachment", "create",
    [{
        "name": "Logo JHK",
        "datas": data,
        "res_model": "ir.ui.view",
        "type": "binary",
        "public": True,
    }]
)
# Retorna el ID del attachment creado
```

### PDFs a Odoo
- **Límite práctico:** < 50MB por archivo vía XML-RPC (nginx devuelve 413 Request Entity Too Large para archivos grandes)
- Para PDFs grandes (>50MB): comprimirlos con Ghostscript perfil `ebook` ANTES de subir
- **Si siguen siendo >50MB tras compresión:** servir el PDF directamente desde NAS montado en la web (sin subirlo a Odoo)

### Comprimir PDFs grandes con Ghostscript (macOS)
```bash
# Ghostscript en macOS (Homebrew)
GS=/opt/homebrew/Cellar/ghostscript/10.07.1/bin/gs
$GS -sDEVICE=pdfwrite \
    -dCompatibilityLevel=1.4 \
    -dPDFSETTINGS=/ebook \
    -dNOPAUSE -dQUIET -dBATCH \
    -sOutputFile=salida_comprimida.pdf \
    original_grande.pdf

# Resultado típico:
#   roly.pdf (93MB) → roly_ebook.pdf (47MB, -50%)
#   the_infinity.pdf (252MB) → the_infinity_ebook.pdf (26MB, -90%)
#   stamina.pdf (110MB) → resultado variable según densidad de imágenes internas
```

```python
import os, fitz

# Alternativa con PyMuPDF (menos agresivo)
doc = fitz.open("/ruta/original.pdf")
doc.save("/ruta/salida.pdf", garbage=4, deflate=True, clean=True)
doc.close()
# PyMuPDF suele comprimir menos que GS pero preserva mejor la calidad en PDFs con muchas imágenes
```

### URLs de adjuntos en Odoo 17
```
Logo:     https://globalo.es/web/image/ir.attachment/<id>/datas
PDF:      https://globalo.es/web/content/<id>
Favicon:  https://globalo.es/web/image/ir.attachment/<id>/datas/300x300
```

### Verificar adjuntos públicos
```python
# Activar acceso público en un attachment
models.execute_kw(db, uid, password, "ir.attachment", "write",
    [[attachment_id], {"public": True}])

# Marcar página web como publicada
models.execute_kw(db, uid, password, "website.page", "write",
    [[page_id], {"is_published": True}])
```

### CRÍTICO: `datas` en ir.attachment.create — string plano, NO Binary()

El campo `datas` de `ir.attachment` espera un **string base64** (no un objeto `xmlrpc.client.Binary`).

```python
# ✅ CORRECTO: base64 encode → decode a string plano
with open("/ruta/archivo.pdf", "rb") as f:
    datas = base64.b64encode(f.read()).decode("utf-8")

models.execute_kw(db, uid, password, "ir.attachment", "create", [{
    "name": "Mi PDF",
    "datas": datas,           # ← string plano, NO Binary()
    "mimetype": "application/pdf",
    "type": "binary",
    "public": True,
}])

# ❌ INCORRECTO — da TypeError: argument should be a bytes-like object
with open("/ruta/archivo.pdf", "rb") as f:
    datas = xmlrpc.client.Binary(f.read())  # ← NUNCA hagas esto

models.execute_kw(db, uid, password, "ir.attachment", "create", [{
    "name": "Mi PDF",
    "datas": datas,  # ← falla
}])
```

### Campos que NO existen en ir.attachment (Odoo 17)
- `website_published` → **no existe** en este modelo. Filtrar por este campo provoca `ValueError: Invalid field ir.attachment.website_published`. Usar `public` o `website_id` en su lugar.
- `datas_fname` → no es válido en Odoo 17.

### Límite de tamaño en uploads RPC
- **Límite práctico:** ~50MB por request (nginx lanza `413 Request Entity Too Large` si se supera)
- **Solución para PDFs 50-120MB:** comprimir con Ghostscript perfil `ebook` antes de subir (típicamente reduce 50-90%)
- **Solución para PDFs >120MB:** 即使 comprimidos siguen >50MB → servir desde NAS montado directamente en la web (sin subir a Odoo), o subir manualmente por la interfaz web de Odoo
- **Ghostscript resultados típicos (macOS con Homebrew):**
  - 93MB → 47MB (−50%)
  - 252MB → 26MB (−90%)
  - 110MB → vary según densidad de imágenes internas del PDF; puede no reducirse significativamente
- **Ghostscript path en macOS:** `/opt/homebrew/Cellar/ghostscript/<version>/bin/gs` (Homebrew-installed). No usar `which gs` ni `$PATH` — la versión del sistema puede no estar disponible. Siempre verificar con `brew list --formula ghostscript` para obtener el path exacto.
- **PyMuPDF + PDFs sin metadata:** `doc.add_metadata(reader.metadata)` falla si el PDF no tiene metadata definida (`reader.metadata is None`). Catch con `try/except` o simplemente omite el paso de metadata.
- **Pregunta precio/tiempo estimado antes de ejecutar.** El usuario gestiona clientes.
- **`write` en website.page devuelve error "Unescaped '<' not allowed in attributes values":** el HTML que escribes en `arch` se valida como XML. Cualquier `<` dentro de un atributo `style=` o `href=` dentro de un SVG inline o similar genera error. Solución: usar variables CSS o clases en vez de estilos inline complejos con `<` caracteres.
- **Si el usuario pide datos de la empresa (footer, contacto):** buscar en `res.company` (modelo base de Odoo), NO en configuración de website. Campos disponibles: `name`, `phone`, `email`, `website`, `vat`, `street`, `city`, `zip`, `country_id`, `color`. Campo `favicon` NO existe en `res.company` — error `ValueError: Invalid field 'favicon'`. El campo `vat` incluye prefijo país, ej. `ESB35909886`.
- **Pregunta precio/tiempo estimado antes de ejecutar.** El usuario gestiona clientes.
### Endpoint
```
POST https://api.minimax.io/v1/image_generation
Authorization: Bearer <token>
Content-Type: application/json
```

### Ejemplo de uso
```python
import requests, base64

response = requests.post(
    "https://api.minimax.io/v1/image_generation",
    headers={"Authorization": f"Bearer {token}"},
    json={
        "model": "image-01",
        "prompt": "Black and white logo for JHK professional uniforms brand",
    }
)
image_url = response.json()["data"][0]["b64_json"]
# Decodificar: base64.b64decode(image_url) → bytes de imagen
```

### Notas
- El token de MiniMax funciona en `api.minimax.io` pero no en `api.minimax.chat`
- Límite: ~50 imágenes por sesión
- Genera logos/ilustraciones B/W para catálogos de marca

## Pitfalls

- NO asumas estructura de negocio. Pregunta si el material es marca principal o sub-catalogos.
- NO empieces sin aprobacion del boceto. Usuario quiere ver plan primero.
- NO elimines menus existentes sin preguntar.
- NO subas archivos a prod sin verificar visualmente con Computer Use.
- Computer Use es lento para trabajo iterativo. Codigo para cambios grandes, CU solo explorar/verificar.
- **Catalogos pueden ser descargables o enlaces externos. Pregunta a qué enlaza cada tarjeta.**
- **En Odoo 17, muchos modelos dan error `psycopg2.ProgrammingError: can't adapt type 'dict'` si tienen campos many2one. Usa xmlrpc/2/object en vez de urllib/jsonrpc.**
- **Solo `/xmlrpc/2/common` funciona para autenticación en Odoo 17. `/jsonrpc` es solo para login legacy, NO para operaciones de modelo.**
- **image_generate puede fallar si `FAL_KEY` no está configurada. Fallback: usa MiniMax API vía execute_code.**
- **Ghostscript en PDFs con muchas imágenes internas:** puede dar `Failed to initialise downsample filter` y quedarse colgado o incluso inflar el archivo (un 110MB → 123MB). En esos casos usar PyMuPDF (`fitz`) con `garbage=4, deflate=True` como alternativa parcial — aunque la compresión suele ser menos agresiva.
- **PyMuPDF + PDFs sin metadata:** `doc.add_metadata(reader.metadata)` falla si el PDF no tiene metadata definida (`reader.metadata is None`). Catch con `try/except` o simplemente omite el paso de metadata.
- **Pregunta precio/tiempo estimado antes de ejecutar.** El usuario gestiona clientes.
- **`write` en website.page devuelve error "Unescaped '<' not allowed in attributes values":** el HTML que escribes en `arch` se valida como XML. Cualquier `<` dentro de un atributo `style=` o `href=` dentro de un SVG inline o similar genera error. Solución: usar variables CSS o clases en vez de estilos inline complejos con `<` caracteres.

- **Footer: usar plantilla Headline (NO website.layout directamente)**
  - La plantilla `website.template_footer_headline` (id=8309) con `key=website.template_footer_headline` controla el footer de TODAS las páginas web de Odoo.
  - Hereda de `website.layout` y usa XPath `//div[@id='footer']` para reemplazarlo.
  - **Para editarla:** buscar por `key='website.template_footer_headline'` en `ir.ui.view`.
  - **Estructura:** dos `s_text_block` — el primero para headline/título, el segundo para columnas (menú, contacto, social).
  - **CRÍTICO:** debe tener `active=True` para renderizarse. Si está `active=False`, el footer NO aparece.
  - **Para actualizarfooter datos empresa:** hacer `write` en el ID 8309 (o el que corresponda según el site) con el `arch_db` correcto.

```
# Actualizar Headline template via XML-RPC
models.execute_kw(db, uid, password, 'ir.ui.view', 'write', [[8309], {
    'arch_db': '<data inherit_id="website.layout" name="Headline" active="True">...',
    'active': True
}])
```

- **Footer monocromo (B/N):** usar Font Awesome icons: `fa-phone`, `fa-envelope`, `fa-map-marker`
- **Footer menú:** Inicio, Contáctenos, Catálogos (3 enlaces)
- **Footer tagline empresa:** "Somos un equipo apasionado cuyo objetivo es vestir a empresas y profesionales."

- **`ir.ui.view` write no acepta `arch_db` via XML-RPC.** `IrUiView.write()` rechaza `{'arch_db': val}` y `{'arch': val}` con error "got an unexpected keyword argument". Esto pasa con los templates footer/header. **Workaround:** usar `website.page` con su campo `arch_db` si es una page; o edita el template directamente desde el backend de Odoo (no vía RPC). Para `website.page`, SÍ funciona: `models.execute_kw(db, uid, password, 'website.page', 'write', [[page_id], {'arch_db': xml_string}])`.

- **`write` en ir.ui.view NO limpia el caché de plantillas**
  - Odoo marca la vista como dirty pero el worker puede servir plantillas cacheadas durante ~5 min.
  - **Solución 1:** recargar con `?debug=assets` en la URL (fuerza recompilación QWeb)
  - **Solución 2:** en backend Odoo: Ajustes → Tecnico → Interfaces → Vistas → Buscar la plantilla → Invalidar
  - **NO usar** `clear_caches()` vía XML-RPC → da `IndexError: tuple index out of range`

- **Safariax tree solo devuelve menús (474+ elementos MenuBar repetidos):** el contenido web NO aparece en el AX tree de Safari porque necesita "Allow JavaScript from Apple Events". Solución: habilitarlo en Safari → Preferencias → Avanzado, o usar zoom/screenshot para ver contenido.

- **cua-driver se desconecta ("MCP server unreachable"):**多半 por falta de permisos TCC o Safari JS Apple Events no habilitado. Ejecutar `mcp_cua_driver_check_permissions` con `prompt=true` para que muestre los diálogos de permisos. Para Safari, además habilitar manualmente "Allow JavaScript from Apple Events".

- **`computer_use` con `mode=vision` falla si el modelo activo no tiene visión:** deepseek-v4-flash (modelo de texto) no soporta `mode=vision`. Para Computer Use con visión hay que usar un modelo con capacidad visual (kimi2.6) como modelo principal o en paralelo.

- **`debug=1` vs `debug=assets`:** `debug=1` recarga el debugger de Odoo pero puede no forzar QWeb. `debug=assets` fuerza la recompilación de activos incluyendo plantillas QWeb. Usar `?debug=assets` cuando se quiera ver cambios de plantilla reflejados inmediatamente.

- **Computer Use: flujo de trabajo Safari**
  1. `mcp_cua_driver_check_permissions` → verificar permisos TCC
  2. Click en banner de cookies "Acepto" (elemento 19 típicamente)
  3. Scroll hasta encontrar el botón "Editor" (normalmente después de banners de cookies)
  4. Click en "Editor" para activar modo edición visual
  5. Para refrescar plantillas QWeb: escribir en el campo URL `?debug=assets` y recargar

- **`clear_caches()` no funciona vía XML-RPC en Odoo 17:** el método `clear_caches()` de `ir.ui.view` espera parámetros positional args pero XML-RPC solo pasa `args` y `kwargs` separadamente, causando `IndexError: tuple index out of range`. No intentar. Usar `?debug=assets` en su lugar.

- **`computer_use` con `mode=vision` falla si el modelo activo no tiene visión:** deepseek-v4-flash (modelo de texto) no soporta `mode=vision`. Para Computer Use con visión hay que usar un modelo con capacidad visual (kimi2.6) como modelo principal o en paralelo.

- **Pregunta precio/tiempo estimado antes de ejecutar.** El usuario gestiona clientes.

## Modelos a usar para este skill

| Tarea | Modelo | Proveedor |
|---|---|---|
| Visión/web (ver páginas, screenshots) | **kimi2.6** | MiniMax |
| Código HTML/CSS/QWeb | **deepseek-v4-flash** | Ollama Cloud |
| Generación de imágenes | **MiniMax API** (`image-01`) | api.minimax.io |

**No mezclar:** no usar deepseek para visión ni kimi para código. Cada modelo para su tarea.

## Senales de entrada
- Computer Use + web de cliente
- catalogos + web + Odoo
- boceto / plan + rediseno web
