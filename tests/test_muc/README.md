<!--
Copyright 2017 The Australian National University

Licensed under the Apache License, Version 2.0 (the "License");
you may not use this file except in compliance with the License.
You may obtain a copy of the License at

    http://www.apache.org/licenses/LICENSE-2.0

Unless required by applicable law or agreed to in writing, software
distributed under the License is distributed on an "AS IS" BASIS,
WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
See the License for the specific language governing permissions and
limitations under the License.
-->

You will need [muc](https://gitlab.anu.edu.au/mu/mu-tool-compiler) and the Python 2 version of pytest.

You may find the following environment variables useful:

Variable       | default | description
--------------:|---------|-------------------------
`MUC`          | _muc_   | The command to execute muc (or just put _muc_ in your path)
`MU_LOG_LEVEL` | _none_	 | The log level used by zebu when building and running (_Zebu_ will read this variable at compile time and runtime of your bootimage)
`MU_EMIT_DIR`  | _emit_  | The directory to store the stuff _zebu_ emits
