#!/usr/bin/env python3
from termcolor import colored
import subprocess as sp


if __name__ == "__main__":
    print(colored("Compiling client", "green"))
    status = sp.run("wasm-pack build --dev --target web", shell=True, cwd="./client")

    if status.returncode == 0:
        print(colored("\nCopying package to server\n", "green"))
        sp.run("rm -r public/pkg", shell=True)
        sp.run("cp -r client/pkg public/pkg", shell=True)

    else:
        print(colored("\nFailed to compile\n", "red"))
