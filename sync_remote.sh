# Copyright 2017 The Australian National University
# 
# Licensed under the Apache License, Version 2.0 (the "License");
# you may not use this file except in compliance with the License.
# You may obtain a copy of the License at
# 
#     http://www.apache.org/licenses/LICENSE-2.0
# 
# Unless required by applicable law or agreed to in writing, software
# distributed under the License is distributed on an "AS IS" BASIS,
# WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
# See the License for the specific language governing permissions and
# limitations under the License.

#!/bin/bash

echo "rsync source to machine: $1..."
rsync -avz --exclude 'Cargo.lock' --exclude '*.log' --exclude 'emit*' --exclude 'target*' --exclude '.git' --exclude '*__pycache__*' . $1:~/mu/
