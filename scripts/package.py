#!/usr/bin/env python3
import os
import shutil
import sys
import xml.etree.ElementTree as ET

if len(sys.argv) < 4:
    print("Uso: package.py <nome> <versão> <arquitetura>")
    sys.exit(1)

BUILD_DIR = "build"
NAME, VERSION, ARCH = sys.argv[1:4]
DEB_DIR = f"{NAME}_{VERSION}_{ARCH}"



def strip_lines(text: str) -> str:
    lines = [line.lstrip() for line in text.strip().splitlines()]
    return "\n".join(lines).strip()

def remove_all(path):
    if os.path.exists(path):
        shutil.rmtree(path)

def create_dir(path, element, _):
    os.makedirs(path, exist_ok=True)
    for child in element:
        create_structure(child, path)

def create_file(path, element, name=None):
    os.makedirs(os.path.dirname(path), exist_ok=True)
    if (src := element.attrib.get("src")):
        shutil.copy(src, path)
    elif element.text:
        with open(path, "w") as f:
            f.write(strip_lines(element.text) + "\n")
    else:
        raise ValueError(f"Element {element.tag} has no content")
    if (chmod := element.attrib.get("chmod")):
        os.chmod(path, int(chmod, 8))

def create_structure(element: ET.Element, base_path: str):
    name = element.attrib["name"]
    func = {'dir': create_dir, 'file': create_file}[element.tag]
    func(os.path.join(base_path, name), element, name)

def main():
    remove_all(os.path.join(BUILD_DIR, NAME))

    with open("scripts/manifest.xml") as f:
        xml_content = f.read()

    root = ET.fromstring(
        xml_content
        .replace("__DEB_DIR__", DEB_DIR)
        .format(
            name=NAME,
            version=VERSION,
            arch=ARCH
        )
    )
    create_structure(root, BUILD_DIR)
    print(f"Estrutura criada em {BUILD_DIR}/{NAME}")

if __name__ == "__main__":
    main()
