#include <stdio.h>
#include <stdlib.h>
#include <stdint.h>

// a pointer to asm label of serialized vm
//extern void* vm;
// a pointer to the symbol table I generate
extern void* mu_sym_table_root;
void * mu_sym_table = NULL;

extern int32_t mu_retval;

int32_t muentry_get_retval(){
    return mu_retval;
}

unsigned long * get_sym_table_root(){
    return (unsigned long *) &mu_sym_table_root;
}

char * get_sym_table_pointer(){
    return (char *) &mu_sym_table;
}

// a function to return a pointer to serialized vm, to rust
char * get_vm_pointer(unsigned long idx){

    unsigned long * current_root = get_sym_table_root();
    if(current_root == NULL){
        printf("get_vm_pointer: SYM TABLE ROOT IS NULL\n\n");
        assert(0);
    }
    unsigned long ii;
    // get the total number of idXs
    unsigned long num_of_idXs = (unsigned long) current_root[0];

    // go to next unsigned long, which is the first idX
    current_root++;

    for (ii=0; ii<num_of_idXs; ii++){
        if( current_root[0] == idx){
            return current_root[2];
        }
        else{
            current_root += 4;
        }
    }
    return NULL;
}

// This function returns 0 if test type is integer and 1 if test type is float
// for integer tests use =>
//      get_input_value(,,)
//      get_output_value(,,)
// for float tests use =>
//      get_fp_input_value(,,)
//      get_fp_output_value(,,)
// And these functions are common for both types =>
//      get_number_of_testcases()
//      get_number_of_input()
//      get_number_of_outputs()

unsigned long get_type_of_testcases(unsigned long test_index){

    unsigned long * current_root = get_sym_table_root();
    if(current_root == NULL){
        printf("get_type_of_testcases: SYM TABLE ROOT IS NULL\n\n");
        assert(0);
    }
    unsigned long ii;
    // get the total number of idXs
    unsigned long num_of_idXs = (unsigned long) current_root[0];

    // go to next unsigned long, which is the first idX
    current_root++;

    for (ii=0; ii<num_of_idXs; ii++){
        if( current_root[0] == test_index){
            // pointer to testcase data for current test_index is found
            unsigned long * test_root = (unsigned long *) current_root[3];

            unsigned long type_of_testcases = test_root[0];
            unsigned long number_of_testcases = test_root[1];       // first cell is the number of testcases
            unsigned long number_of_inputs = test_root[2];          // second cell is the number of inputs for this test
            unsigned long number_of_outputs = test_root[3];

            return type_of_testcases;

        }
        else{
            current_root += 4;
        }
    }
    return 0;
}

unsigned long get_number_of_testcases(unsigned long test_index){

    unsigned long * current_root = get_sym_table_root();
    if(current_root == NULL){
        printf("get_number_of_testcases: SYM TABLE ROOT IS NULL\n\n");
        assert(0);
    }
    unsigned long ii;
    // get the total number of idXs
    unsigned long num_of_idXs = (unsigned long) current_root[0];

    // go to next unsigned long, which is the first idX
    current_root++;

    for (ii=0; ii<num_of_idXs; ii++){
        if( current_root[0] == test_index){
            // pointer to testcase data for current test_index is found
            unsigned long * test_root = (unsigned long *) current_root[3];

            unsigned long type_of_testcases = test_root[0];
            unsigned long number_of_testcases = test_root[1];       // first cell is the number of testcases
            unsigned long number_of_inputs = test_root[2];          // second cell is the number of inputs for this test
            unsigned long number_of_outputs = test_root[3];


            return number_of_testcases;

        }
        else{
            current_root += 4;
        }
    }
    return 0;
}


unsigned long get_number_of_inputs(unsigned long test_index){

    unsigned long * current_root = get_sym_table_root();
    if(current_root == NULL){
        printf("get_number_of_inputs: SYM TABLE ROOT IS NULL\n\n");
        assert(0);
    }
    unsigned long ii;
    // get the total number of idXs
    unsigned long num_of_idXs = (unsigned long) current_root[0];

    // go to next unsigned long, which is the first idX
    current_root++;

    for (ii=0; ii<num_of_idXs; ii++){
        if( current_root[0] == test_index){
            // pointer to testcase data for current test_index is found
            unsigned long * test_root = (unsigned long *) current_root[3];

            unsigned long number_of_testcases = test_root[1];       // first cell is the number of testcases
            unsigned long number_of_inputs = test_root[2];          // second cell is the number of inputs for this test
            unsigned long number_of_outputs = test_root[3];

            return number_of_inputs;

        }
        else{
            current_root += 4;
        }
    }
    return 0;
}

unsigned long get_number_of_outputs(unsigned long test_index){

    unsigned long * current_root = get_sym_table_root();
    if(current_root == NULL){
        printf("get_number_of_outputs: SYM TABLE ROOT IS NULL\n\n");
        assert(0);
    }
    unsigned long ii;
    // get the total number of idXs
    unsigned long num_of_idXs = (unsigned long) current_root[0];

    // go to next unsigned long, which is the first idX
    current_root++;

    for (ii=0; ii<num_of_idXs; ii++){
        if( current_root[0] == test_index){
            // pointer to testcase data for current test_index is found
            unsigned long * test_root = (unsigned long *) current_root[3];

            unsigned long number_of_testcases = test_root[0];       // first cell is the number of testcases
            unsigned long number_of_inputs = test_root[1];          // second cell is the number of inputs for this test
            unsigned long number_of_outputs = test_root[2];

            return number_of_outputs;

        }
        else{
            current_root += 4;
        }
    }
    return 0;
}

// specific to integer tests
long get_input_value(unsigned long test_index, unsigned long testcase_index, unsigned long input_index){

    unsigned long * current_root = get_sym_table_root();
    if(current_root == NULL){
        printf("get_input_value: SYM TABLE ROOT IS NULL\n\n");
        assert(0);
    }
    unsigned long ii;
    // get the total number of idXs
    unsigned long num_of_idXs = (unsigned long) current_root[0];

    // go to next unsigned long, which is the first idX
    current_root++;

    for (ii=0; ii<num_of_idXs; ii++){
        if( current_root[0] == test_index){
            // pointer to testcase data for current test_index is found
            unsigned long * test_root = (unsigned long *) current_root[3];

            unsigned long number_of_testcases = test_root[1];       // first cell is the number of testcases
            unsigned long number_of_inputs = test_root[2];          // second cell is the number of inputs for this test
            unsigned long number_of_outputs = test_root[3];

            if ( testcase_index < (number_of_testcases+1) ){
                unsigned long exact_index = 4;
                exact_index += (testcase_index) * (number_of_inputs+number_of_outputs);
                exact_index += (input_index - 1);
                return test_root[exact_index];
            }
            else{
                printf("get_input_value: testcase_index out of range!\n");
                return 0;
            }

        }
        else{
            current_root += 4;
        }
    }
    return 0;
}

int get_output_value(unsigned long test_index, unsigned long testcase_index){

    unsigned long * current_root = get_sym_table_root();
    if(current_root == NULL){
        printf("get_output_value: SYM TABLE ROOT IS NULL\n\n");
        assert(0);
    }
    unsigned long ii;
    // get the total number of idXs
    unsigned long num_of_idXs = (unsigned long) current_root[0];

    // go to next unsigned long, which is the first idX
    current_root++;

    for (ii=0; ii<num_of_idXs; ii++){
        if( current_root[0] == test_index){
            // pointer to testcase data for current test_index is found
            unsigned long * test_root = (unsigned long *) current_root[3];

            unsigned long number_of_testcases = test_root[1];       // first cell is the number of testcases
            unsigned long number_of_inputs = test_root[2];          // second cell is the number of inputs for this test
            unsigned long number_of_outputs = test_root[3];

            // for tests which don't have a return value
            if (number_of_testcases == 0)
                return 0;

            if ( testcase_index < (number_of_testcases) ){
                unsigned long exact_index = 4;
                exact_index += (testcase_index) * (number_of_inputs+number_of_outputs);
                exact_index += (number_of_inputs);
                return test_root[exact_index];
            }
            else{
                printf("get_output_value: testcase_index out of range!\n");
//                printf("%d\n", testcase_index);
                return 0;
            }

        }
        else{
            current_root += 4;
        }
    }
    return 0;
}

// specific to float tests
double get_fp_input_value(unsigned long test_index, unsigned long testcase_index, unsigned long input_index){

    unsigned long * current_root = get_sym_table_root();
    if(current_root == NULL){
        printf("get_input_value: SYM TABLE ROOT IS NULL\n\n");
        assert(0);
    }
    unsigned long ii;
    // get the total number of idXs
    unsigned long num_of_idXs = (unsigned long) current_root[0];

    // go to next unsigned long, which is the first idX
    current_root++;

    for (ii=0; ii<num_of_idXs; ii++){
        if( current_root[0] == test_index){
            // pointer to testcase data for current test_index is found
            unsigned long * test_root = (unsigned long *) current_root[3];

            unsigned long number_of_testcases = test_root[1];       // first cell is the number of testcases
            unsigned long number_of_inputs = test_root[2];          // second cell is the number of inputs for this test
            unsigned long number_of_outputs = test_root[3];

            double * fp_test_root = (double *)(&test_root[4]);

            if ( testcase_index < (number_of_testcases+1) ){
                unsigned long exact_index = 0;
                exact_index += (testcase_index) * (number_of_inputs+number_of_outputs);
                exact_index += (input_index - 1);
                return fp_test_root[exact_index];
            }
            else{
                printf("get_fp_input_value: testcase_index out of range!\n");
                return 0;
            }

        }
        else{
            current_root += 4;
        }
    }
    return 0;
}

double get_fp_output_value(unsigned long test_index, unsigned long testcase_index){

    unsigned long * current_root = get_sym_table_root();
    if(current_root == NULL){
        printf("get_output_value: SYM TABLE ROOT IS NULL\n\n");
        assert(0);
    }
    unsigned long ii;
    // get the total number of idXs
    unsigned long num_of_idXs = (unsigned long) current_root[0];

    // go to next unsigned long, which is the first idX
    current_root++;

    for (ii=0; ii<num_of_idXs; ii++){
        if( current_root[0] == test_index){
            // pointer to testcase data for current test_index is found
            unsigned long * test_root = (unsigned long *) current_root[3];

            unsigned long number_of_testcases = test_root[1];       // first cell is the number of testcases
            unsigned long number_of_inputs = test_root[2];          // second cell is the number of inputs for this test
            unsigned long number_of_outputs = test_root[3];

            double * fp_test_root = (double *)(&test_root[4]);

            // for tests which don't have a return value
            if (number_of_testcases == 0)
                return 0;

            if ( testcase_index < (number_of_testcases) ){
                unsigned long exact_index = 0;
                exact_index += (testcase_index) * (number_of_inputs+number_of_outputs);
                exact_index += (number_of_inputs);
                return fp_test_root[exact_index];
            }
            else{
                printf("get_output_value: testcase_index out of range!\n");
//                printf("%d\n", testcase_index);
                return 0;
            }

        }
        else{
            current_root += 4;
        }
    }
    return 0;
}

unsigned long get_number_of_tests(){
    unsigned long * current_root = get_sym_table_root();
    if(current_root == NULL){
        printf("get_number_of_tests: SYM TABLE ROOT IS NULL\n\n");
        assert(0);
    }
    return (unsigned long) current_root[0];
}


// a function to resolve symbol names to their address, using mu_sym_table\
// which is valued by the c_resolve_sym function, according to \
// mu_sym_table_root
// actually this function will search for the symbol in one idX_sym_table
void * resolve_symbol_in_current_context(const unsigned char * input_symbol_name){
    unsigned long i = 0, j=0;
    unsigned long num_of_syms = 0;
    unsigned long cur_len = 0;
    unsigned long input_len = 0;
    unsigned char cur_char = 0;
    unsigned long cur_add = 0;
    unsigned char is_different = 0;

    while(input_symbol_name[input_len] != 0){
        input_len++;
    }

    if( (mu_sym_table == NULL) || (input_len == 0)){
        printf("**********\n****ERROR!!!****\nC_RESOLVE_SYMBOL: extern mu sym table is null!\n*********\n");
        assert(0);
    }

    // a copy of mu_sym_table, to keep mu_sym_table from changing
//    unsigned char * sym_table_current = (unsigned char *) get_sym_table_pointer();
    unsigned char * sym_table_current = (unsigned char *) mu_sym_table;
//    printf("C_RESOLVE_SYMBOL --> Step 1\n");

//    printf("C_RESOLVE_SYMBOL --> Current mu_sym_table = %p\n", sym_table_current);
//    printf("C_RESOLVE_SYMBOL --> Current vm = %p\n", get_vm_pointer());

    // read the total number of symbols from the first line of sym table
    num_of_syms = *((unsigned long*)sym_table_current);
//    printf("RESOLVE_SYM --> number of symbols = %d\n", num_of_syms);
    // go 8 bytes forward, to skip the current .quad
    sym_table_current += 8;

    for(i=0; i<num_of_syms; i++){
        is_different = 0;
        // length of the symbol we are going to check
        cur_len = *((unsigned long*)sym_table_current);
        if(cur_len != input_len){
            sym_table_current += 8;     //cur_len
            sym_table_current += 8;     //cur_add
            sym_table_current += cur_len;     //length of cur_name
//            printf("*** Sym name doesn't match! ***\n");
            continue;
        }
        sym_table_current += 8;
//        printf("Sym_Tbl current Len = %d\n", cur_len);
        for(j=0; j<cur_len; j++){
            cur_char = *((unsigned char*)sym_table_current);
            sym_table_current += 1;
//            printf("Sym_Tbl read char = %d , input symbol current char = %d\n", cur_char, input_symbol_name[j]);
            if(cur_char == input_symbol_name[j]){
                continue;
            }
            else{
                is_different = 1;
                sym_table_current += ((cur_len-j)-1);
                break;
            }
        }
        if(is_different == 1){
            // skip the 64b address in current location and continue to the next sym in table
//            printf("*** Sym name doesn't match! ***\n");
            sym_table_current += 8;
            continue;
        }
        else{
//            printf("*** Sym name = %s! Sym Address = %llu***\n", input_symbol_name, *((long*) sym_table_current));
//            printf("*** Last char = %c\n", sym_table_current[-1]);
//            for(j=0; j<8; j++){
//                printf("*** Last char = %c\n", sym_table_current[j]);
//            }

            unsigned long result = 0;
            unsigned long shifter=1;
            unsigned int ii=0, jj=0;

            for (ii=0; ii<8; ii++){
                shifter = 1;
                for(jj=0; jj<ii; jj++)
                    shifter = shifter * 256;
                result += ((sym_table_current[ii]%256)* shifter);
//                printf("Char = %x\n", (sym_table_current[ii]%256));
//                printf("Result = %llu\n", result);
            }

//            printf("*** going to return {-%llu-}\n", result);
            return ((void*) result);
        }
    }
    return NULL;
}

void * c_resolve_symbol(const unsigned char * input_symbol_name){
    unsigned long * current_root = get_sym_table_root();
    if(current_root == NULL){
        printf("c_resolve_symbol: SYM TABLE ROOT IS NULL\n\n");
        assert(0);
    }
    unsigned long idX = 0;
    unsigned long ii;
    unsigned long * table_pointer = NULL;
    // get the total number of idXs
    unsigned long num_of_idXs = (unsigned long) current_root[0];

    // go to next unsigned long, which is the first idX
    current_root++;

    for (ii=0; ii<num_of_idXs; ii++){
        idX = current_root[0];
        table_pointer = current_root[1];
        mu_sym_table = table_pointer;
        table_pointer = resolve_symbol_in_current_context(input_symbol_name);
        if(table_pointer == NULL){
            current_root += 4;
            continue;
        }
        break;
    }
    return table_pointer;
}
