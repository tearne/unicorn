#!/usr/bin/python

import subprocess
import sys
import os
import pexpect
import getpass
import importlib
import site

def main():
    # Cargo must be installed
    if not run("cargo --version"):
        input("Press enter to install cargo")
        run("curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh")
    else: print("Cargo already installed")

    if run("cargo deb -v"):
        input("Press enter to install cargo deb")
        run("cargo install cargo-deb")
    else:
        print("Cargo deb already installed")

    # Compile and build deb package
    proj_dir = os.path.dirname(os.path.realpath(__file__))
    print("Project dir is ", proj_dir)
    child = pexpect.spawnu("cargo deb -p syspixel", cwd=proj_dir)
    child.logfile_read=sys.stdout
    options = [r'target/debian/syspixel_(\S*).deb', pexpect.TIMEOUT, pexpect.EOF]
    index = child.expect(options)
    deb_path = child.match.group()
    child.expect(pexpect.EOF)
    print(f"Debian package build complete: {deb_path}")

    # Install deb package
    password = Password()
    sudo(f"apt install ../{deb_path}", Password().get(), timeout=30, cwd=proj_dir)
  

#
# Setup
#

class Password:
    def __init__(self):
        self.p = None

    def get(self):
        if self.p: return self.p
        else:
            self.p = getpass.getpass()
            return self.p


def run(cmd, cwd=None):
    print("*** running", cmd)
    return subprocess.run(
        cmd,
        shell=True,
        stdout=sys.stdout,
        stderr=sys.stderr,
        encoding='utf8',
        cwd=cwd
    )

def sudo(cmd, password, timeout = -1, cwd=None):
    child = pexpect.spawnu(f"sudo {cmd}", cwd=cwd)
    child.logfile_read=sys.stdout
    options = ['password', pexpect.TIMEOUT, pexpect.EOF]
    index = child.expect(options, timeout = 1)
    if index > 0:
        print(f"Error waiting for password prompt: {options[index]} - {child.before.decode()}")
        sys.exit(1)
    
    child.sendline(password)
    
    options = [pexpect.EOF, 'try again', pexpect.TIMEOUT]
    index = child.expect(options, timeout=timeout)
    if index == 0:
        return
    elif index == 1:
        print(f"Authentication failure: {options[index]}")
        sys.exit(1)
    else:
        print(f"Command failure: {options[index]}")
        sys.exit(1)
    

def ensure_import(package_name):
    try:
        pkg = importlib.import_module(package_name)
        print(f"Dependency '{package_name}' already installed")
        return pkg
    except ImportError as err:
        print(f"Dependency {package_name} not installed yet: {err}")
        subprocess.check_call([sys.executable, '-m', 'pip', 'install', package_name])
        importlib.reload(site)  # reloads sys.path
        importlib.invalidate_caches()
        return importlib.import_module(package_name)


if __name__ == '__main__':
    main()
