#!/usr/bin/env python3
import os
import shutil
import sys
import xml.etree.ElementTree as ET
from typing import Callable, Dict

if len(sys.argv) < 4:
    print("Uso: package.py <nome> <versão> <arquitetura>")
    sys.exit(1)

BUILD_DIR = "build"
NAME, VERSION, ARCH = sys.argv[1:4]
DEB_DIR = f"{NAME}_{VERSION}_{ARCH}"

context = {
    "name": NAME,
    "version": VERSION,
    "arch": ARCH,
    "DEB_DIR": DEB_DIR
}

def strip_lines(text: str) -> str:
    lines = [line.lstrip() for line in text.strip().splitlines()]
    return "\n".join(lines).strip()

def create_dir(path, element, _):
    os.makedirs(path, exist_ok=True)
    for child in element:
        create_structure(child, path)

def remove_all(path):
    if os.path.exists(path):
        shutil.rmtree(path)

def create_file(path, element, name=None):
    content = ""
    src = element.attrib.get("src")
    if src:
        os.makedirs(os.path.dirname(path), exist_ok=True)
        shutil.copy(src, path)
        return

    if element.text:
        content = strip_lines(element.text)
    os.makedirs(os.path.dirname(path), exist_ok=True)

    with open(path, "w") as f:
        f.write(content + "\n")

    if name in ("postinst", "prerm"):
        os.chmod(path, 0o755)

funcs: Dict[str, Callable] = {
    'dir': create_dir,
    'file': create_file
}

def create_structure(element: ET.Element, base_path: str):
    """
    Cria pastas e arquivos a partir do XML.
    - Usa 'name' se existir, senão usa tag.
    - type="dir" cria diretório
    - type="file" cria arquivo com CDATA ou src
    """
    # Nome do arquivo/pasta
    func = funcs[element.attrib.get("type", "dir")]
    name = element.attrib.get("name", element.tag)
    func(os.path.join(base_path, name), element, name)

def main():
    remove_all(os.path.join(BUILD_DIR, NAME))

    with open("scripts/manifest.xml") as f:
        xml_content = f.read()

    root = ET.fromstring(xml_content.format(**context))
    create_structure(root, BUILD_DIR)
    print(f"Estrutura criada em {BUILD_DIR}/{context['name']}")

if __name__ == "__main__":
    main()
