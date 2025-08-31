#!/usr/bin/env python3
import sys
import subprocess


if len(sys.argv) < 4:
    print("Uso: package.py <nome> <versão> <arquitetura>")
    sys.exit(1)

BUILD_DIR = "build"
NAME, VERSION, ARCH = sys.argv[1:4]
DEB_DIR = f"{NAME}_{VERSION}_{ARCH}"

def uninstall():
    print(f"Desinstalando {NAME} {VERSION} {ARCH}")
    try:
        subprocess.run(["sudo", "dpkg", "-P", f"{NAME}"], check=True)
    except subprocess.CalledProcessError as e:
        print(f"Erro ao desinstalar {NAME}: {e}")
        sys.exit(1)

def main():
    uninstall()

if __name__ == "__main__":
    main()
