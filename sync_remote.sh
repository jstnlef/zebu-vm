#!/bin/bash

echo "rsync source to machine: $1..."
rsync -avz --exclude 'Cargo.lock' --exclude '*.log' --exclude 'emit*' --exclude 'target*' --exclude '.git' --exclude '*__pycache__*' . $1:~/mu/