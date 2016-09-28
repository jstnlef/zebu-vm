#pragma once

#include <muapi.h>

MuVM* new_mock_micro_vm(char* name);
void free_mock_micro_vm(MuVM* mvm);
