#!/usr/bin/env python3
import os
import shutil
import sys
import subprocess
import xml.etree.ElementTree as ET

if len(sys.argv) < 4:
    print("Uso: package.py <nome> <versão> <arquitetura>")
    sys.exit(1)

BUILD_DIR = "build"
NAME, VERSION, ARCH = sys.argv[1:4]
DEB_DIR = f"{NAME}_{VERSION}_{ARCH}"


def strip_lines(text: str) -> str:
    lines = [line.lstrip() for line in text.splitlines()]
    return "\n".join(lines) + "\n"  # garante a newline final

def remove_all(path):
    if os.path.exists(path):
        shutil.rmtree(path)

def create_dir(path, element):
    os.makedirs(path, exist_ok=True)
    for child in element:
        create_structure(child, path)

def create_file(path, element):
    os.makedirs(os.path.dirname(path), exist_ok=True)

    compress = element.attrib.get("compress")
    if compress:
        src_before = element.attrib["src-before"]
        src_after = element.attrib["src-after"]

        if compress == "gzip":
            subprocess.run(["gzip", "-9", "-c", src_before], check=True, stdout=open(src_after, "wb"))
        else:
            raise ValueError(f"Compressão '{compress}' não suportada")

        shutil.copy(src_after, path)

    elif (src := element.attrib.get("src")):
        shutil.copy(src, path)

    else:
        if element.text and element.text.strip():
            with open(path, "w") as f:
                f.write(strip_lines(element.text))

    if (chmod := element.attrib.get("chmod")):
        os.chmod(path, int(chmod, 8))

    print(f"Arquivo criado: {path}")

def create_structure(element: ET.Element, base_path: str):
    func = {'dir': create_dir, 'file': create_file}[element.tag]
    func(os.path.join(base_path, element.attrib["name"]), element)

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

    # Caminhos
    package_dir = os.path.join(BUILD_DIR, DEB_DIR)
    deb_file = f"{DEB_DIR}.deb"
    deb_path = os.path.join(BUILD_DIR, deb_file)

    # Gera o .deb
    subprocess.run(
        ["dpkg-deb", "--build", package_dir, deb_path],
        check=True
    )

    # Copia para current dir
    shutil.copy(deb_path, os.getcwd())
    print(f"Pacote gerado: ./{deb_file}")


if __name__ == "__main__":
    main()
