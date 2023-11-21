#!/bin/bash

set -e

cargo test --no-default-features --features integration
