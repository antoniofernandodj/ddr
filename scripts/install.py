#!/usr/bin/env python3
import subprocess
import sys

if len(sys.argv) < 4:
    print("Uso: install.py <nome> <versão> <arquitetura>")
    sys.exit(1)

name, version, arch = sys.argv[1:4]
deb_file = f"{name}_{version}_{arch}.deb"
print(f"==> Instalando {deb_file}...")
subprocess.check_call(["sudo", "dpkg", "-i", deb_file])
