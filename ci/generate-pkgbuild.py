#!/usr/bin/env python3
from os import environ, makedirs
import hashlib


release_tag = environ.get("RELEASE_TAG")
if not release_tag:
  print("::error ::RELEASE_TAG is required but missing")
  exit(1)


def calculate_sha256(filename):
    sha256_hash = hashlib.sha256()
    with open(filename, "rb") as f:
        # read and update hash string value in blocks of 4K
        for byte_block in iter(lambda: f.read(4096), b""):
            sha256_hash.update(byte_block)
        return sha256_hash.hexdigest()


print("Generating PKGBUILD for map2...")
makedirs("./dist/aur", exist_ok=True)
with open("./dist/aur/PKGBUILD", "w") as out:
    checksum_x86_64 = calculate_sha256(f"./wheels/map2-{release_tag}-cp312-cp312-manylinux_2_17_x86_64.manylinux2014_x86_64.whl")
    checksum_i686   = calculate_sha256(f"./wheels/map2-{release_tag}-cp312-cp312-manylinux_2_17_i686.manylinux2014_i686.whl"    )

    content = open("./ci/templates/PKGBUILD").read()\
        .replace("pkgver=", f"pkgver={release_tag}")\
        .replace("sha256sums_x86_64=('')", f"sha256sums_x86_64=('{checksum_x86_64}')")\
        .replace("sha256sums_i686=('')", f"sha256sums_i686=('{checksum_i686}')")

    out.write(content)
