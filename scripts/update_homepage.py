import xmlrpc.client

url = 'https://globalo.es/xmlrpc/2/common'
db = 'globalo'
username = 'clawdia@nicolasramos.es'
password = '@ClawdiA@'
uid = xmlrpc.client.ServerProxy(url).authenticate(db, username, password, {})
models = xmlrpc.client.ServerProxy('https://globalo.es/xmlrpc/2/object')

new_arch = """<t t-name="website.inicio">
  <t t-call="website.layout">
    <div id="wrap" class="oe_structure oe_empty">

      <!-- HERO: Imagen de fondo + headline -->
      <section class="s_picture pt48 pb24 o_colored_level o_cc o_cc5" data-snippet="s_picture" data-name="Foto" style="background-image: url('/web/image/123497'); background-position: center; background-size: cover; position: relative;" data-oe-shape-data="{"shape":"web_editor/Origins/02_001","flip":[],"showOnMobile":false}">
        <div class="o_we_shape o_web_editor_Origins_02_001"/>
        <div class="container">
          <div class="row s_nb_column_fixed o_grid_mode" data-row-count="3">
            <div class="o_grid_item g-col-lg-12 g-height-3 col-lg-12 text-center" style="z-index: 2;">
              <h1 class="display-2-fs text-white o_default_snippet_text" style="font-weight: 700; text-shadow: 0 2px 8px rgba(0,0,0,0.5);">
                Uniformes y ropa de trabajo
              </h1>
              <p class="lead text-white o_default_snippet_text mt16" style="font-size: 1.25rem; text-shadow: 0 1px 4px rgba(0,0,0,0.5);">
                Para empresas, colegios e institutos - En Lanzarote
              </p>
              <a href="/contactus" class="btn btn-dark mt24" style="background-color: #000000; color: #ffffff; border-radius: 4px; padding: 12px 32px; font-weight: 600;">
                Solicitar presupuesto
              </a>
            </div>
          </div>
        </div>
      </section>

      <!-- TARJETAS: Colegios / Institutos / Personalizaciones -->
      <section class="s_card o_colored_level pt48 pb48" data-snippet="s_card" data-name="Tarjeta" style="background-color: #f5f5f5;">
        <div class="container">
          <div class="row">
            <div class="col-lg-4 mb-4 mb-lg-0">
              <div class="card rounded-0 shadow-sm h-100" style="border: 1px solid #e0e0e0;">
                <div class="card-body text-center" style="background-color: #ffffff;">
                  <h3 class="h5 text-dark font-weight-bold">Colegios</h3>
                  <p class="card-text text-muted small">Uniformes escolares para centros educativos. Camisas, pantalones, batas y prendas completas con el diseno de tu colegio.</p>
                </div>
              </div>
            </div>
            <div class="col-lg-4 mb-4 mb-lg-0">
              <div class="card rounded-0 shadow-sm h-100" style="border: 1px solid #e0e0e0;">
                <div class="card-body text-center" style="background-color: #ffffff;">
                  <h3 class="h5 text-dark font-weight-bold">Institutos</h3>
                  <p class="card-text text-muted small">Ropa de trabajo y uniformes tecnicos para estudiantes y docentes. Calidad profesional con precios educativos.</p>
                </div>
              </div>
            </div>
            <div class="col-lg-4 mb-4 mb-lg-0">
              <div class="card rounded-0 shadow-sm h-100" style="border: 1px solid #e0e0e0;">
                <div class="card-body text-center" style="background-color: #ffffff;">
                  <h3 class="h5 text-dark font-weight-bold">Personalizaciones</h3>
                  <p class="card-text text-muted small">Especialistas en textil personalizado. Bordamos y printamos tu logo en cualquier prenda. Presupuesto sin compromiso.</p>
                </div>
              </div>
            </div>
          </div>
        </div>
      </section>

      <!-- QUIENES SOMOS -->
      <section class="s_text_block o_colored_level pt48 pb0" data-snippet="s_text_block" data-name="Texto">
        <div class="container">
          <div class="row">
            <div class="col-lg-8 offset-lg-2 text-center">
              <h2 class="text-dark o_default_snippet_text" style="font-weight: 700;">Somos DISEGLOB S.L.U.</h2>
              <p class="lead text-muted o_default_snippet_text mt16">Somos un equipo apasionado cuyo objetivo es vestir a empresas y profesionales de Lanzarote y Canarias.</p>
            </div>
          </div>
        </div>
      </section>

      <!-- CTA: Ver Catalogos -->
      <section class="s_cta_box o_colored_level pt48 pb48 mt24" data-snippet="s_cta_box" data-name="Llamada a la accion" style="background-color: #000000;" data-oe-shape-data="{"shape":"web_editor/Origins/02_001","flip":[],"showOnMobile":false}">
        <div class="o_we_shape o_web_editor_Origins_02_001"/>
        <div class="container">
          <div class="row">
            <div class="col-lg-8 offset-lg-2 text-center">
              <h3 class="text-white o_default_snippet_text" style="font-weight: 700;">Explora nuestros catalogos</h3>
              <p class="text-white-50 o_default_snippet_text mt8 mb24">Descarga catalogos de JHK, Roly, Berlei, Gilsen y mas.</p>
              <a href="/catalogos" class="btn btn-light" style="border-radius: 4px; padding: 12px 32px; font-weight: 600;">Ver catalogos</a>
            </div>
          </div>
        </div>
      </section>

    </div>
  </t>
</t>"""

ok = models.execute_kw(db, uid, password, 'website.page', 'write', [[11], {'arch_db': new_arch}])
print(f"Homepage written: {ok}")
print(f"Arch length: {len(new_arch)}")
