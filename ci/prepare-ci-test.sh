#!/bin/bash

set -e

sudo apt-get install -y libxkbcommon-dev

python -m venv .env
source .env/bin/activate
pip install maturin