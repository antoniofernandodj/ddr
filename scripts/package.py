#!/usr/bin/env python3
import os
import shutil
import subprocess
import sys
import xml.etree.ElementTree as ET


def strip_lines(text: str) -> str:
    """Remove indentação extra e espaços das linhas"""
    lines = [line.lstrip() for line in text.strip().splitlines()]
    return "\n".join(lines).strip()

def find_by_id(elem: ET.Element, id_value: str) -> ET.Element:
    """Percorre recursivamente todos os elementos para encontrar atributo id"""
    if elem.attrib.get("id") == id_value:
        return elem
    for child in elem.iter():
        if child.attrib.get("id") == id_value:
            return child
    raise LookupError(f"Elemento não encontrado: {id_value}")

if len(sys.argv) < 4:
    print("Uso: package.py <nome> <versão> <arquitetura>")
    sys.exit(1)

BUILD_DIR = "build"
NAME, VERSION, ARCH = sys.argv[1:4]
DEB_DIR_NAME = f"{NAME}_{VERSION}_{ARCH}"

# Lê o manifest XML
with open("scripts/manifest.xml") as f:
    xml_content = f.read()

root = ET.fromstring(
    xml_content.format(
        DEB_DIR=DEB_DIR_NAME,
        name=NAME,
        version=VERSION,
        arch=ARCH
    )
)

deb_dir_elem = root
deb_dir = os.path.join(BUILD_DIR, DEB_DIR_NAME)
if os.path.exists(deb_dir):
    shutil.rmtree(deb_dir)

print(f"==> Criando estrutura {deb_dir}")
os.makedirs(deb_dir, exist_ok=True)

# --- DEBIAN: control, postinst, prerm ---
for name in ["control", "postinst", "prerm"]:
    text = deb_dir_elem.findtext(f"DEBIAN/{name}")
    if text is None:
        continue
    debian_dir = os.path.join(deb_dir, "DEBIAN")
    os.makedirs(debian_dir, exist_ok=True)
    file_path = os.path.join(debian_dir, name)
    with open(file_path, "w") as f:
        f.write(strip_lines(text) + "\n")
    if name in ("postinst", "prerm"):
        os.chmod(file_path, 0o755)

# --- Binário ---
bin_elem = find_by_id(deb_dir_elem, "binary")
bin_dir = os.path.join(deb_dir, "usr/local/bin")
os.makedirs(bin_dir, exist_ok=True)
shutil.copy(bin_elem.attrib["src"].strip(), bin_dir)

# --- Empacotar com dpkg-deb ---
deb_file = f"{deb_dir}.deb"
print(f"==> Gerando pacote {deb_file}")
subprocess.check_call(["dpkg-deb", "--build", deb_dir])
shutil.move(f"{deb_dir}.deb", f"{DEB_DIR_NAME}.deb")

print(f"==> Pacote criado: {deb_file}")
