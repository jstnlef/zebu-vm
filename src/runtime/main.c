#include <stdlib.h>
#include <stdio.h>
#include <stdint.h>
#include <string.h>

extern void* vm;
extern void mu_main(char* vm);
extern void mu_trace_level_log();

int main() {
	mu_trace_level_log();
	
	printf("main(), going to launch mu_main()\n");
	char* serialize_vm = (char*) &vm;
	
	printf("%s\n", serialize_vm);

	mu_main(serialize_vm);
}
