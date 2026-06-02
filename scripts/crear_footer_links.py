import xmlrpc.client

url = 'https://globalo.es/xmlrpc/2/common'
db = 'globalo'
username = 'clawdia@nicolasramos.es'
password = '@ClawdiA@'
uid = xmlrpc.client.ServerProxy(url).authenticate(db, username, password, {})
models = xmlrpc.client.ServerProxy('https://globalo.es/xmlrpc/2/object')

# Create Devoluciones menu
ok1 = models.execute_kw(db, uid, password, 'website.menu', 'create', [{
    'name': 'Devoluciones',
    'url': '/page/devoluciones',
    'parent_id': 76,
    'website_id': 1,
    'sequence': 200,
}])
print(f"Devoluciones menu created: id={ok1}")

# Create Privacidad menu
ok2 = models.execute_kw(db, uid, password, 'website.menu', 'create', [{
    'name': 'Politica de Privacidad',
    'url': '/page/politica-de-privacidad-y-cookies',
    'parent_id': 76,
    'website_id': 1,
    'sequence': 201,
}])
print(f"Politica de Privacidad menu created: id={ok2}")

# List top-level menus
top_menus = models.execute_kw(db, uid, password, 'website.menu', 'search_read',
    [['parent_id', '=', 76]],
    {'fields': ['id', 'name', 'url', 'sequence'], 'order': 'sequence'})
print("\nTop-level menus:")
for m in top_menus:
    print(f"  id={m['id']} seq={m['sequence']} name={m['name']!r} url={m['url']!r}")
