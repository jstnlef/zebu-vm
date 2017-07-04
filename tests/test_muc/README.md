You will need the python 2 version of pytest and (muc)[https://gitlab.anu.edu.au/mu/mu-tool-compiler].

You may find the following environemnt variables usefull:
* `MUC` (default _muc_)			Set to the path to use to execute muc (or just put muc in your path)
* `MU_LOG_LEVEL` (default _none_)	The log level used by zebu when building and running (Zebu will read this variable at compile time and runtime of your bootimage)
* `MU_EMIT_DIR` (default _emit_)	The directory to store the stuff zebu emits
