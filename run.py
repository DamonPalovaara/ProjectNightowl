#!/usr/bin/env python3
from termcolor import colored
import subprocess as sp

print(colored("Compiling client", "green"))
status = sp.run('wasm-pack build --target web',
                shell=True, cwd="./client").returncode
if status != 0:
    print(colored("Failed to compile", "red"))
else:
    print(colored("Copying package to server", "green"))
    sp.run('rm -r public/pkg', shell=True)
    sp.run('cp -r client/pkg public/pkg', shell=True)

    print(colored("Booting server", "green"))
    sp.run('cargo run', shell=True)
