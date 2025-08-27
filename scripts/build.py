#!/usr/bin/env python3
import subprocess

print("==> Compilando em modo release...")
subprocess.check_call(["cargo", "build", "--release"])
