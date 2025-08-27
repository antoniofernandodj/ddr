#!/usr/bin/env python3
import os
import shutil
import subprocess

print("==> Limpando...")
subprocess.call(["cargo", "clean"])
shutil.rmtree("build", ignore_errors=True)
shutil.rmtree("target", ignore_errors=True)
for f in os.listdir("."):
    if f.endswith(".deb"):
        os.remove(f)
